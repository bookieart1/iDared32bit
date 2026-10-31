/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Opt-in, bounded flight recorder. Recording never calls guest code, formats
//! text, allocates, writes logs, or changes guest state. This records boundaries,
//! not instructions, and is not proof that a nil return is an error.

use crate::mem::{ConstVoidPtr, Mem};
use crate::Environment;
use std::time::{Duration, Instant};

const RECENT: usize = 96;
const SIGNALS: usize = 24;
const PROGRESS: usize = 24;

#[derive(Clone, Copy, Debug)]
struct Text {
    bytes: [u8; 80],
    len: usize,
}
impl Default for Text {
    fn default() -> Self {
        Self {
            bytes: [0; 80],
            len: 0,
        }
    }
}
impl Text {
    fn from_bytes(bytes: &[u8]) -> Self {
        let mut text = Self::default();
        text.len = bytes.len().min(text.bytes.len());
        text.bytes[..text.len].copy_from_slice(&bytes[..text.len]);
        text
    }
    fn guest(mem: &Mem, address: u32) -> Self {
        let mut text = Self::default();
        for i in 0..text.bytes.len() {
            let Some(address) = address.checked_add(i as u32) else {
                break;
            };
            let Some(byte) = mem.get_bytes_fallible(ConstVoidPtr::from_bits(address), 1) else {
                break;
            };
            if byte[0] == 0 {
                break;
            }
            text.bytes[i] = byte[0];
            text.len += 1;
        }
        text
    }
    fn bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}
impl std::fmt::Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Escape control characters so guest data cannot impersonate log lines.
        for &b in self.bytes() {
            if (32..127).contains(&b) {
                write!(f, "{}", b as char)?;
            } else {
                write!(f, "\\x{b:02x}")?;
            }
        }
        if self.len == self.bytes.len() {
            write!(f, "[truncated]")?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Call {
    sequence: u64,
    thread: usize,
    kind: &'static str,
    name: Text,
    pc: u32,
    lr: u32,
    sp: u32,
    stack: [u32; 4],
    stack_len: usize,
    args: [u32; 4],
    receiver: u32,
    isa: u32,
    detail: Text,
    mib: [u32; 8],
    mib_len: usize,
}

#[derive(Clone, Copy, Debug, Default)]
struct Event {
    sequence: u64,
    call: Call,
    result: Option<[u32; 2]>,
    note: &'static str,
}

struct Ring<T: Copy + Default, const N: usize> {
    items: [T; N],
    written: u64,
}
impl<T: Copy + Default, const N: usize> Default for Ring<T, N> {
    fn default() -> Self {
        Self {
            items: [T::default(); N],
            written: 0,
        }
    }
}
impl<T: Copy + Default, const N: usize> Ring<T, N> {
    fn push(&mut self, item: T) {
        self.items[self.written as usize % N] = item;
        self.written += 1;
    }
    fn iter(&self) -> impl Iterator<Item = &T> {
        let count = self.written.min(N as u64);
        (self.written - count..self.written).map(|i| &self.items[i as usize % N])
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Progress {
    ms: u128,
    thread: usize,
    pc: u32,
    lr: u32,
    threads: usize,
    runnable: usize,
    api: u64,
    objc: u64,
    run_loops: u64,
    presents: u64,
    yields: u64,
    sources: [usize; 4],
}

pub struct Recorder {
    recent: Ring<Event, RECENT>,
    signals: Ring<Event, SIGNALS>,
    progress: Ring<Progress, PROGRESS>,
    sequence: u64,
    api: u64,
    objc: u64,
    filtered: u64,
    pub run_loops: u64,
    pub presents: u64,
    /// Last observed main-loop source counts: timers, selectors, queues, units.
    pub sources: [usize; 4],
    yields: u64,
    started: Instant,
    next_progress: Instant,
    dumped: bool,
}
impl Recorder {
    pub fn new() -> Box<Self> {
        let now = Instant::now();
        Box::new(Self {
            recent: Ring::default(),
            signals: Ring::default(),
            progress: Ring::default(),
            sequence: 0,
            api: 0,
            objc: 0,
            filtered: 0,
            run_loops: 0,
            presents: 0,
            sources: [0; 4],
            yields: 0,
            started: now,
            next_progress: now + Duration::from_secs(5),
            dumped: false,
        })
    }
    fn begin(&mut self, mut call: Call) -> Call {
        self.sequence += 1;
        call.sequence = self.sequence;
        let event = Event {
            sequence: self.sequence,
            call,
            result: None,
            note: "enter",
        };
        self.recent.push(event);
        if call.name.bytes() == b"raise" {
            self.signals.push(event);
        }
        call
    }
    fn finish(&mut self, call: Call, result: [u32; 2], object_result: bool) {
        self.sequence += 1;
        let signal = (object_result && result[0] == 0)
            || (call.name.bytes() == b"_dlsym" && result[0] == 0)
            || (call.name.bytes() == b"_open" && result[0] == u32::MAX)
            || (call.name.bytes() == b"_fopen" && result[0] == 0);
        let event = Event {
            sequence: self.sequence,
            call,
            result: Some(result),
            note: if object_result && result[0] == 0 {
                "nil-object-return (may be normal)"
            } else if signal {
                "lookup/resource-failure"
            } else {
                "return (raw ABI)"
            },
        };
        self.recent.push(event);
        if signal {
            self.signals.push(event);
        }
    }
    pub fn checkpoint(&mut self, thread: usize, regs: &[u32], threads: usize, runnable: usize) {
        self.yields += 1;
        let now = Instant::now();
        if now < self.next_progress {
            return;
        }
        self.next_progress = now + Duration::from_secs(5);
        let sample = Progress {
            ms: now.duration_since(self.started).as_millis(),
            thread,
            pc: regs[15],
            lr: regs[14],
            threads,
            runnable,
            api: self.api,
            objc: self.objc,
            run_loops: self.run_loops,
            presents: self.presents,
            yields: self.yields,
            sources: self.sources,
        };
        self.progress.push(sample);
        // Persist limited evidence even when a host kill prevents a final dump.
        if self.progress.written <= 12 {
            Self::print_progress(&sample);
        }
    }
    fn print_progress(p: &Progress) {
        echo_no_panic!("flight progress ms={} tid={} pc={:#x} lr={:#x} thread-slots={} runnable={} api={} objc={} main-loops={} EAGL-returns={} yields={} main-sources[timer,selector,queue,unit]={:?}",
            p.ms, p.thread, p.pc, p.lr, p.threads, p.runnable, p.api, p.objc, p.run_loops, p.presents, p.yields, p.sources);
    }
    fn print_event(e: &Event) {
        let c = &e.call;
        echo_no_panic!("flight #{} call={} tid={} {} {} {} pc={:#x} lr={:#x} sp={:#x} args={:08x?} stack={:08x?} receiver={:#x} isa={:#x} result={:08x?} text=\"{}\" mib={:?}",
            e.sequence, c.sequence, c.thread, c.kind, c.name, e.note, c.pc, c.lr, c.sp,
            c.args, &c.stack[..c.stack_len], c.receiver, c.isa, e.result, c.detail, &c.mib[..c.mib_len]);
    }
    pub fn dump(&mut self, reason: &str) {
        if self.dumped {
            return;
        }
        self.dumped = true;
        echo_no_panic!("flight recorder v1: {} elapsed-ms={} api={} objc={} filtered={} main-loops={} EAGL-returns={} (raw registers; LR may be inherited for host-internal calls)",
            reason, self.started.elapsed().as_millis(), self.api, self.objc, self.filtered, self.run_loops, self.presents);
        echo_no_panic!(
            "flight progress history: newest {PROGRESS}; total={}",
            self.progress.written
        );
        for p in self.progress.iter() {
            Self::print_progress(p);
        }
        echo_no_panic!(
            "flight candidate signals: newest {SIGNALS}; total={} (not diagnoses)",
            self.signals.written
        );
        for e in self.signals.iter() {
            Self::print_event(e);
        }
        echo_no_panic!("flight recent boundaries: newest {RECENT}; total={} (older events overwritten; absent return does not prove a stall)", self.recent.written);
        for e in self.recent.iter() {
            Self::print_event(e);
        }
        echo_no_panic!("flight recorder end");
    }
    #[cfg(test)]
    pub fn last_entry(&self) -> Option<(&[u8], &[u32])> {
        self.recent
            .iter()
            .filter(|e| e.result.is_none())
            .last()
            .map(|e| (e.call.name.bytes(), &e.call.mib[..e.call.mib_len]))
    }
}

fn read_word(mem: &Mem, addr: u32) -> Option<u32> {
    let bytes = mem.get_bytes_fallible(ConstVoidPtr::from_bits(addr), 4)?;
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}

impl Environment {
    /// Observe existing guest/host boundaries without ownership changes.
    pub fn diagnostic_api_begin(&mut self, name: &'static str) -> Option<Call> {
        let recorder = self.diagnostics.as_mut()?;
        recorder.api += 1;
        // ObjC is recorded at its decoded dispatch boundary. Count high-volume
        // graphics/allocator/memory primitives without erasing other history.
        if name.starts_with("_objc_msgSend")
            || name.starts_with("_gl")
            || matches!(
                name,
                "_malloc"
                    | "_free"
                    | "_calloc"
                    | "_realloc"
                    | "_memcpy"
                    | "_memset"
                    | "_memmove"
                    | "_strlen"
            )
        {
            recorder.filtered += 1;
            return None;
        }
        let regs = self.cpu.regs();
        let mut call = Call {
            thread: self.current_thread,
            kind: "API",
            name: Text::from_bytes(name.as_bytes()),
            pc: regs[15],
            lr: regs[14],
            sp: regs[13],
            args: regs[..4].try_into().unwrap(),
            ..Call::default()
        };
        for i in 0..call.stack.len() {
            let Some(addr) = call.sp.checked_add((i * 4) as u32) else {
                break;
            };
            let Some(word) = read_word(&self.mem, addr) else {
                break;
            };
            call.stack[i] = word;
            call.stack_len += 1;
        }
        if name == "_sysctl" {
            for i in 0..(regs[1] as usize).min(call.mib.len()) {
                let Some(addr) = regs[0].checked_add((i * 4) as u32) else {
                    break;
                };
                let Some(word) = read_word(&self.mem, addr) else {
                    break;
                };
                call.mib[i] = word;
                call.mib_len += 1;
            }
        } else if matches!(name, "_dlsym" | "_open" | "_fopen") {
            call.detail = Text::guest(&self.mem, regs[if name == "_dlsym" { 1 } else { 0 }]);
        }
        Some(recorder.begin(call))
    }
    pub fn diagnostic_objc_begin(
        &mut self,
        receiver: u32,
        selector: u32,
        host_internal: bool,
    ) -> Option<Call> {
        let recorder = self.diagnostics.as_mut()?;
        recorder.objc += 1;
        let name = Text::guest(&self.mem, selector);
        if matches!(
            name.bytes(),
            b"retain" | b"release" | b"autorelease" | b"retainCount"
        ) {
            recorder.filtered += 1;
            return None;
        }
        let regs = self.cpu.regs();
        let isa = read_word(&self.mem, receiver).unwrap_or(0);
        let class_name = self
            .objc
            .try_get_class_name(crate::objc::id::from_bits(isa))
            .unwrap_or("<unknown-class>");
        Some(recorder.begin(Call {
            thread: self.current_thread,
            kind: if host_internal { "ObjC(host)" } else { "ObjC" },
            name,
            receiver,
            isa,
            detail: Text::from_bytes(class_name.as_bytes()),
            pc: regs[15],
            lr: regs[14],
            sp: regs[13],
            args: regs[..4].try_into().unwrap(),
            ..Call::default()
        }))
    }
    pub fn diagnostic_end(&mut self, call: Option<Call>, object_result: bool) {
        if let (Some(recorder), Some(call)) = (&mut self.diagnostics, call) {
            let regs = self.cpu.regs();
            if call.name.bytes() == b"presentRenderbuffer:" && regs[0] != 0 {
                recorder.presents += 1;
            }
            recorder.finish(call, [regs[0], regs[1]], object_result);
        }
    }
    pub fn diagnostic_dump(&mut self, reason: &str) {
        if let Some(recorder) = &mut self.diagnostics {
            recorder.dump(reason);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ring_retains_order_after_wrap() {
        let mut ring = Ring::<u32, 3>::default();
        for i in 0..10 {
            ring.push(i);
        }
        assert_eq!(ring.iter().copied().collect::<Vec<_>>(), [7, 8, 9]);
    }
    #[test]
    fn invalid_pointers_and_interleaved_returns_are_observational() {
        let mut env = Environment::new_fake_for_test();
        env.cpu.regs_mut()[0..4].copy_from_slice(&[0, 6, 0, 4]);
        let before = *env.cpu.regs();
        assert!(env.diagnostic_api_begin("_sysctl").is_none());
        env.diagnostics = Some(Recorder::new());
        let a = env.diagnostic_api_begin("_sysctl").unwrap();
        env.current_thread = 2;
        let b = env.diagnostic_objc_begin(0, u32::MAX, false).unwrap();
        env.diagnostic_end(Some(b), true);
        env.current_thread = 0;
        env.diagnostic_end(Some(a), false);
        assert_eq!(before, *env.cpu.regs());
        let recorder = env.diagnostics.as_ref().unwrap();
        let events: Vec<_> = recorder.recent.iter().collect();
        assert_eq!(events.len(), 4);
        assert_eq!(events[2].call.thread, 2);
        assert_eq!(events[3].call.sequence, a.sequence);
        assert_eq!(recorder.signals.written, 1);
        assert_eq!(a.mib_len, 0);
    }
    #[test]
    fn objc_dispatch_distinguishes_nil_objects_from_numeric_zero() {
        use crate::objc::{id, msg, msg_class, nil, release};
        let mut env = Environment::new_fake_for_test();
        let env = &mut env;
        let array: id = msg_class![env; NSArray new];
        env.diagnostics = Some(Recorder::new());
        let count: u32 = msg![env; array count];
        assert_eq!(count, 0);
        assert_eq!(env.diagnostics.as_ref().unwrap().signals.written, 0);
        let last: id = msg![env; array lastObject];
        assert_eq!(last, nil);
        let recorder = env.diagnostics.as_ref().unwrap();
        assert_eq!(recorder.signals.written, 1);
        assert_eq!(
            recorder.signals.iter().last().unwrap().call.name.bytes(),
            b"lastObject"
        );
        release(env, array);
    }
    #[test]
    fn progress_and_output_are_bounded() {
        let mut recorder = Recorder::new();
        for _ in 0..40 {
            recorder.next_progress = Instant::now();
            recorder.checkpoint(0, &[0; 16], 1, 1);
        }
        assert_eq!(recorder.progress.iter().count(), PROGRESS);
        assert_eq!(recorder.progress.written, 40);
        assert!(std::mem::size_of::<Recorder>() < 80 * 1024);
        let text = Text::from_bytes(b"name\nforged line");
        assert_eq!(format!("{text}"), "name\\x0aforged line");
    }
}
