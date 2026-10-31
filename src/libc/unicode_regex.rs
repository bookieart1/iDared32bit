/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Small compatibility surface for the ICU regular-expression API.
//!
//! Older apps sometimes use `uregex_open` only to validate a pattern before
//! falling back to Foundation's regular-expression helpers. Keep an opaque
//! guest-side handle so those probes can complete without pretending that the
//! rest of ICU's matcher is implemented.

use crate::dyld::{export_c_func, FunctionExports};
use crate::mem::{ConstPtr, MutPtr, MutVoidPtr};
use crate::Environment;

#[allow(non_camel_case_types)]
type UChar = u16;
#[allow(non_camel_case_types)]
type UErrorCode = i32;

const U_ZERO_ERROR: UErrorCode = 0;

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
    let handle: MutPtr<u32> = env.mem.alloc(12).cast();
    env.mem.write(handle, pattern.to_bits());
    env.mem.write(handle + 1, pattern_length as u32);
    env.mem.write(handle + 2, flags);
    log_once!(
        "Warning: uregex_open provides an opaque compatibility handle; ICU matching is not implemented yet"
    );
    handle.cast()
}

fn uregex_close(env: &mut Environment, regexp: MutVoidPtr) {
    if !regexp.is_null() {
        env.mem.free(regexp);
    }
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(uregex_open(_, _, _, _, _)),
    export_c_func!(uregex_close(_)),
];
