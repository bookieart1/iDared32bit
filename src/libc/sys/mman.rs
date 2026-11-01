/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::abi::DotDotDot;
use crate::dyld::FunctionExports;
use crate::environment::Environment;
use crate::export_c_func;
use crate::libc::errno::{set_errno, EINVAL, ENOTSUP};
use crate::libc::posix_io;
use crate::libc::posix_io::{off_t, FileDescriptor, SEEK_SET};
use crate::mem::VMAllocError;
use crate::mem::{ConstPtr, GuestUSize, MutVoidPtr, Ptr, PAGE_SIZE_ALIGN_MASK};
use std::collections::HashMap;

#[allow(dead_code)]
const MAP_FILE: i32 = 0x0000;
const MAP_FIXED: i32 = 0x0010;
const MAP_ANON: i32 = 0x1000;
const VM_FLAGS_PURGABLE: u32 = 0x0000_0002;
const VM_FLAGS_ALIAS_MASK: u32 = 0xff00_0000;
const MAP_FAILED: MutVoidPtr = Ptr::from_bits(u32::MAX);

#[derive(Default)]
pub struct State {
    /// Keeping track of `mmap` allocations
    mmap_allocations: HashMap<MutVoidPtr, GuestUSize>,
}

impl State {
    pub fn forget_overlapping(&mut self, address: MutVoidPtr, size: GuestUSize) {
        let start = address.to_bits() as u64;
        let end = start + size as u64;
        self.mmap_allocations.retain(|allocation, allocation_size| {
            let allocation_start = allocation.to_bits() as u64;
            let allocation_end = allocation_start + *allocation_size as u64;
            allocation_end <= start || end <= allocation_start
        });
    }
}

/// For files, our implementation of mmap is really simple:
/// it's just load entirety of file in memory!
fn mmap(
    env: &mut Environment,
    addr: MutVoidPtr,
    len: GuestUSize,
    prot: i32,
    flags: i32,
    fd: FileDescriptor,
    offset: off_t,
) -> MutVoidPtr {
    // TODO: handle errno properly
    set_errno(env, 0);

    log_dbg!(
        "mmap({:?}, {}, {}, {}, {}, {})",
        addr,
        len,
        prot,
        flags,
        fd,
        offset
    );

    if len == 0 {
        set_errno(env, EINVAL);
        return MAP_FAILED;
    }

    if flags & MAP_ANON != 0 && fd != -1 {
        let vm_flags = fd as u32;
        // Darwin permits an anonymous mapping's otherwise-unused descriptor
        // to carry VM_MAKE_TAG(tag), encoded in the high byte. Keep rejecting
        // every unrelated bit. Purgeable mappings need lifecycle semantics
        // that the VM allocator does not implement yet.
        if vm_flags & !(VM_FLAGS_ALIAS_MASK | VM_FLAGS_PURGABLE) != 0 {
            log!(
                "Warning: mmap anonymous fd/VM flags {vm_flags:#x} are invalid, returning MAP_FAILED"
            );
            set_errno(env, EINVAL);
            return MAP_FAILED;
        }
        if vm_flags & VM_FLAGS_PURGABLE != 0 {
            log!(
                "Warning: mmap anonymous purgeable allocation flags {vm_flags:#x} are unsupported"
            );
            set_errno(env, ENOTSUP);
            return MAP_FAILED;
        }
    }

    assert_eq!(offset, 0);
    let ptr = if addr.is_null() {
        env.mem.vm_alloc(None, len).unwrap()
    } else {
        match env.mem.vm_alloc(Some(addr.to_bits()), len) {
            // The hint may also be free but too small for len.
            Err(VMAllocError::AddressUnavailable | VMAllocError::NoSpace)
                if flags & MAP_FIXED == 0 =>
            {
                let ptr = env.mem.vm_alloc(None, len).unwrap();
                log!("Warning: mmap could not allocate at hint {addr:?}, allocated at {ptr:?}",);
                ptr
            }
            Err(VMAllocError::AddressUnavailable | VMAllocError::NoSpace) => {
                // MAP_FIXED replaces any mappings that overlap the requested
                // range. Free the range first so the allocator can install the
                // replacement at the exact address.
                env.libc_state.mman.forget_overlapping(addr, len);
                env.libc_state
                    .mach_vm
                    .forget_overlapping(addr.to_bits(), len);
                env.mem.vm_free(addr, len);
                env.mem
                    .vm_alloc(Some(addr.to_bits()), len)
                    .unwrap_or_else(|error| {
                        panic!(
                            "mmap MAP_FIXED could not replace {addr:?} ({len:#x} bytes): {error:?}"
                        )
                    })
            }
            Ok(ptr) => ptr,
        }
    };

    assert!(ptr.to_bits() & PAGE_SIZE_ALIGN_MASK == 0);

    if (flags & MAP_ANON) == 0 {
        let new_offset = posix_io::lseek(env, fd, offset, SEEK_SET);
        assert_eq!(new_offset, offset);

        let read = posix_io::read(env, fd, ptr, len);
        assert_eq!(read as u32, len);
    };

    assert!(!env.libc_state.mman.mmap_allocations.contains_key(&ptr));
    env.libc_state.mman.mmap_allocations.insert(ptr, len);

    ptr
}

fn munmap(env: &mut Environment, addr: MutVoidPtr, len: GuestUSize) -> i32 {
    // TODO: handle errno properly
    set_errno(env, 0);

    log_dbg!("munmap({:?}, {})", addr, len);

    if len == 0 {
        set_errno(env, EINVAL);
        // TODO: should we clear allocations for `addr` here too?
        log!("Warning: munmap({:?}, {}) failed, returning -1", addr, len);
        return -1;
    }
    env.libc_state.mman.forget_overlapping(addr, len);
    env.libc_state
        .mach_vm
        .forget_overlapping(addr.to_bits(), len);
    env.mem.vm_free(addr, len);
    0 // success
}

fn madvise(env: &mut Environment, addr: MutVoidPtr, len: GuestUSize, advice: i32) -> i32 {
    log!("TODO: madvise({:?}, {}, {}) -> -1", addr, len, advice);
    set_errno(env, ENOTSUP);
    -1
}

fn shm_open(env: &mut Environment, name: ConstPtr<u8>, oflag: i32, _dots: DotDotDot) -> i32 {
    log!(
        "TODO: shm_open({:?} '{:?}', {}, ...) -> -1",
        name,
        env.mem.cstr_at_utf8(name),
        oflag
    );
    set_errno(env, EINVAL);
    -1
}

fn mprotect(env: &mut Environment, addr: MutVoidPtr, len: GuestUSize, prot: i32) -> i32 {
    // Guest memory does not currently enforce per-page protection flags, so
    // every mapped page already permits the accesses mprotect can request.
    // Report success rather than making runtimes such as Mono treat a harmless
    // protection change as a fatal error.
    log_dbg!("mprotect({:?}, {}, {}) -> 0", addr, len, prot);
    set_errno(env, 0);
    0
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(mmap(_, _, _, _, _, _)),
    export_c_func!(munmap(_, _)),
    export_c_func!(madvise(_, _, _)),
    export_c_func!(shm_open(_, _, _)),
    export_c_func!(mprotect(_, _, _)),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libc::errno::get_errno;

    #[test]
    fn anonymous_mmap_accepts_darwin_vm_tags() {
        let mut env = Environment::new_fake_for_test();
        let ptr = mmap(
            &mut env,
            MutVoidPtr::null(),
            0x1000,
            3,
            MAP_ANON | 0x0002,
            0xf000_0000u32 as i32,
            0,
        );

        assert_ne!(ptr, MAP_FAILED);
        assert!(env.mem.bytes_at(ptr.cast(), 0x1000).iter().all(|&b| b == 0));
        assert_eq!(munmap(&mut env, ptr, 0x1000), 0);
    }

    #[test]
    fn anonymous_mmap_rejects_non_tag_descriptors_before_allocating() {
        let mut env = Environment::new_fake_for_test();
        let ptr = mmap(
            &mut env,
            MutVoidPtr::null(),
            0x1000,
            3,
            MAP_ANON | 0x0002,
            3,
            0,
        );

        assert_eq!(ptr, MAP_FAILED);
        assert_eq!(get_errno(&mut env), EINVAL);
    }
}
