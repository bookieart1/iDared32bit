/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Small compatibility surface for the ICU regular-expression API.

use crate::dyld::{export_c_func, FunctionExports};
use crate::mem::{ConstPtr, MutPtr, MutVoidPtr, Ptr};
use crate::Environment;
use regex_lite::Regex;

#[allow(non_camel_case_types)]
type UChar = u16;
#[allow(non_camel_case_types)]
type UErrorCode = i32;

const U_ZERO_ERROR: UErrorCode = 0;
const NO_MATCH: u32 = u32::MAX;

// Layout of the guest-side opaque handle, in u32 words.
const PATTERN_PTR: u32 = 0;
const PATTERN_LEN: u32 = 1;
const FLAGS: u32 = 2;
const TEXT_PTR: u32 = 3;
const TEXT_LEN: u32 = 4;
const MATCH_START: u32 = 5;
const MATCH_END: u32 = 6;

fn utf16(env: &Environment, ptr: ConstPtr<UChar>, len: u32) -> String {
    if ptr.is_null() || len == 0 {
        return String::new();
    }
    let units: Vec<u16> = (0..len).map(|i| env.mem.read(ptr + i)).collect();
    String::from_utf16_lossy(&units)
}

fn regex_for_handle(env: &Environment, handle: ConstPtr<u32>) -> Option<Regex> {
    let pattern_ptr: ConstPtr<UChar> = Ptr::from_bits(env.mem.read(handle + PATTERN_PTR));
    let pattern_len = env.mem.read(handle + PATTERN_LEN);
    let flags = env.mem.read(handle + FLAGS);
    let pattern = utf16(env, pattern_ptr, pattern_len);

    // ICU and regex-lite share the syntax used by these older apps. Translate
    // the commonly used ICU flags into inline flags understood by regex-lite.
    let mut inline_flags = String::new();
    if flags & 2 != 0 {
        inline_flags.push('i'); // UREGEX_CASE_INSENSITIVE
    }
    if flags & 4 != 0 {
        inline_flags.push('x'); // UREGEX_COMMENTS
    }
    if flags & 8 != 0 {
        inline_flags.push('m'); // UREGEX_MULTILINE
    }
    if flags & 32 != 0 {
        inline_flags.push('s'); // UREGEX_DOTALL
    }
    let pattern = if inline_flags.is_empty() {
        pattern
    } else {
        format!("(?{}){}", inline_flags, pattern)
    };
    match Regex::new(&pattern) {
        Ok(regex) => Some(regex),
        Err(error) => {
            log!("Warning: couldn't compile ICU regular expression {:?}: {}", pattern, error);
            None
        }
    }
}

fn uregex_open(
    env: &mut Environment,
    pattern: ConstPtr<UChar>,
    pattern_length: i32,
    flags: u32,
    _parse_error: MutVoidPtr,
    status: MutPtr<UErrorCode>,
) -> MutVoidPtr {
    if !status.is_null() {
        env.mem.write(status, U_ZERO_ERROR);
    }

    // Preserve enough state to give the guest a stable, non-null opaque
    // handle. The pattern remains owned by the caller, as in ICU.
    let handle: MutPtr<u32> = env.mem.alloc(28).cast();
    env.mem.write(handle + PATTERN_PTR, pattern.to_bits());
    env.mem.write(handle + PATTERN_LEN, pattern_length.max(0) as u32);
    env.mem.write(handle + FLAGS, flags);
    env.mem.write(handle + TEXT_PTR, 0);
    env.mem.write(handle + TEXT_LEN, 0);
    env.mem.write(handle + MATCH_START, NO_MATCH);
    env.mem.write(handle + MATCH_END, NO_MATCH);
    handle.cast()
}

#[allow(non_snake_case)]
fn uregex_setText(
    env: &mut Environment,
    regexp: MutVoidPtr,
    text: ConstPtr<UChar>,
    text_length: i32,
    status: MutPtr<UErrorCode>,
) {
    if !status.is_null() {
        env.mem.write(status, U_ZERO_ERROR);
    }
    if regexp.is_null() {
        return;
    }

    let text_length = if text_length >= 0 {
        text_length as u32
    } else if text.is_null() {
        0
    } else {
        let mut length = 0;
        while env.mem.read(text + length) != 0 {
            length += 1;
        }
        length
    };
    let handle: MutPtr<u32> = regexp.cast();
    env.mem.write(handle + TEXT_PTR, text.to_bits());
    env.mem.write(handle + TEXT_LEN, text_length);
    env.mem.write(handle + MATCH_START, NO_MATCH);
    env.mem.write(handle + MATCH_END, NO_MATCH);
}

fn uregex_find(
    env: &mut Environment,
    regexp: MutVoidPtr,
    start_index: i32,
    status: MutPtr<UErrorCode>,
) -> bool {
    if !status.is_null() {
        env.mem.write(status, U_ZERO_ERROR);
    }
    if regexp.is_null() {
        return false;
    }
    let handle: MutPtr<u32> = regexp.cast();
    let text_ptr: ConstPtr<UChar> = Ptr::from_bits(env.mem.read(handle + TEXT_PTR));
    let text_len = env.mem.read(handle + TEXT_LEN);
    let text = utf16(env, text_ptr, text_len);

    let mut utf16_start = if start_index >= 0 {
        start_index as u32
    } else {
        let last_start = env.mem.read(handle + MATCH_START);
        let last_end = env.mem.read(handle + MATCH_END);
        if last_end == NO_MATCH {
            0
        } else if last_end == last_start {
            last_end.saturating_add(1)
        } else {
            last_end
        }
    };
    utf16_start = utf16_start.min(text_len);
    let byte_start = text
        .char_indices()
        .map(|(idx, _)| idx)
        .chain(std::iter::once(text.len()))
        .find(|&idx| text[..idx].encode_utf16().count() as u32 >= utf16_start)
        .unwrap_or(text.len());

    let Some(regex) = regex_for_handle(env, handle.cast_const()) else {
        env.mem.write(handle + MATCH_START, NO_MATCH);
        env.mem.write(handle + MATCH_END, NO_MATCH);
        return false;
    };
    let Some(found) = regex.find(&text[byte_start..]) else {
        env.mem.write(handle + MATCH_START, NO_MATCH);
        env.mem.write(handle + MATCH_END, NO_MATCH);
        return false;
    };
    let absolute_start = byte_start + found.start();
    let absolute_end = byte_start + found.end();
    let match_start = text[..absolute_start].encode_utf16().count() as u32;
    let match_end = text[..absolute_end].encode_utf16().count() as u32;
    env.mem.write(handle + MATCH_START, match_start);
    env.mem.write(handle + MATCH_END, match_end);
    true
}

fn uregex_findNext(
    env: &mut Environment,
    regexp: MutVoidPtr,
    status: MutPtr<UErrorCode>,
) -> bool {
    uregex_find(env, regexp, -1, status)
}

fn uregex_reset(
    env: &mut Environment,
    regexp: MutVoidPtr,
    index: i32,
    status: MutPtr<UErrorCode>,
) {
    if !status.is_null() {
        env.mem.write(status, U_ZERO_ERROR);
    }
    if regexp.is_null() {
        return;
    }
    let handle: MutPtr<u32> = regexp.cast();
    let index = index.max(0) as u32;
    env.mem.write(handle + MATCH_START, index);
    env.mem.write(handle + MATCH_END, index);
}

fn uregex_start(
    env: &mut Environment,
    regexp: MutVoidPtr,
    group: i32,
    status: MutPtr<UErrorCode>,
) -> i32 {
    if !status.is_null() {
        env.mem.write(status, U_ZERO_ERROR);
    }
    if regexp.is_null() || group != 0 {
        return -1;
    }
    let handle: ConstPtr<u32> = regexp.cast_const().cast();
    let value = env.mem.read(handle + MATCH_START);
    if value == NO_MATCH { -1 } else { value as i32 }
}

fn uregex_end(
    env: &mut Environment,
    regexp: MutVoidPtr,
    group: i32,
    status: MutPtr<UErrorCode>,
) -> i32 {
    if !status.is_null() {
        env.mem.write(status, U_ZERO_ERROR);
    }
    if regexp.is_null() || group != 0 {
        return -1;
    }
    let handle: ConstPtr<u32> = regexp.cast_const().cast();
    let value = env.mem.read(handle + MATCH_END);
    if value == NO_MATCH { -1 } else { value as i32 }
}

fn uregex_close(env: &mut Environment, regexp: MutVoidPtr) {
    if !regexp.is_null() {
        env.mem.free(regexp);
    }
}

#[allow(non_snake_case)]
fn uregex_groupCount(
    env: &mut Environment,
    regexp: MutVoidPtr,
    status: MutPtr<UErrorCode>,
) -> i32 {
    if !status.is_null() {
        env.mem.write(status, U_ZERO_ERROR);
    }
    if regexp.is_null() {
        return 0;
    }

    let handle: ConstPtr<u32> = regexp.cast_const().cast();
    let pattern: ConstPtr<UChar> = Ptr::from_bits(env.mem.read(handle));
    let pattern_length = env.mem.read(handle + 1) as i32;
    if pattern.is_null() || pattern_length <= 0 {
        return 0;
    }

    // Count capture groups without implementing a matcher. This covers the
    // ICU syntax used by the legacy apps while excluding escaped parentheses,
    // character classes, lookarounds, and non-capturing groups.
    let mut count = 0i32;
    let mut escaped = false;
    let mut in_class = false;
    let len = pattern_length as u32;
    for i in 0..len {
        let ch = env.mem.read(pattern + i);
        if escaped {
            escaped = false;
            continue;
        }
        if ch == b'\\' as u16 {
            escaped = true;
            continue;
        }
        if ch == b'[' as u16 {
            in_class = true;
            continue;
        }
        if ch == b']' as u16 && in_class {
            in_class = false;
            continue;
        }
        if in_class || ch != b'(' as u16 {
            continue;
        }

        let next = (i + 1 < len).then(|| env.mem.read(pattern + i + 1));
        if next != Some(b'?' as u16) {
            count += 1;
            continue;
        }

        // (?<name>...) is capturing; (?<=...) and (?<!...) are lookbehind.
        if i + 3 < len && env.mem.read(pattern + i + 2) == b'<' as u16 {
            let marker = env.mem.read(pattern + i + 3);
            if marker != b'=' as u16 && marker != b'!' as u16 {
                count += 1;
            }
        }
    }
    count
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(uregex_open(_, _, _, _, _)),
    export_c_func!(uregex_setText(_, _, _, _)),
    export_c_func!(uregex_find(_, _, _)),
    export_c_func!(uregex_findNext(_, _)),
    export_c_func!(uregex_reset(_, _, _)),
    export_c_func!(uregex_start(_, _, _)),
    export_c_func!(uregex_end(_, _, _)),
    export_c_func!(uregex_groupCount(_, _)),
    export_c_func!(uregex_close(_)),
];
