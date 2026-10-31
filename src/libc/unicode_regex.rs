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
use crate::mem::{ConstPtr, MutPtr, MutVoidPtr, Ptr};
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
    let handle: MutPtr<u32> = env.mem.alloc(20).cast();
    env.mem.write(handle, pattern.to_bits());
    env.mem.write(handle + 1, pattern_length as u32);
    env.mem.write(handle + 2, flags);
    env.mem.write(handle + 3, 0);
    env.mem.write(handle + 4, 0);
    log_once!(
        "Warning: uregex_open provides an opaque compatibility handle; ICU matching is not implemented yet"
    );
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
    env.mem.write(handle + 3, text.to_bits());
    env.mem.write(handle + 4, text_length);
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
    export_c_func!(uregex_groupCount(_, _)),
    export_c_func!(uregex_close(_)),
];
