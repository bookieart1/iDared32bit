/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Minimal Core Foundation stream compatibility.

use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::foundation::ns_string::to_rust_string;
use crate::mem::MutPtr;
use crate::objc::{id, nil};
use crate::Environment;

type CFStringRef = id;
type CFReadStreamRef = id;
type CFWriteStreamRef = id;

/// Network-backed NSStream objects are not implemented yet. Returning null
/// stream references matches a failed stream-pair creation and lets callers
/// gracefully disable optional services such as analytics and cloud profiles.
fn CFStreamCreatePairWithSocketToHost(
    env: &mut Environment,
    _allocator: id,
    host: CFStringRef,
    port: u32,
    read_stream: MutPtr<CFReadStreamRef>,
    write_stream: MutPtr<CFWriteStreamRef>,
) {
    log!(
        "Warning: CFStreamCreatePairWithSocketToHost({}, {}) is unavailable; returning NULL streams",
        to_rust_string(env, host),
        port
    );
    if !read_stream.is_null() {
        env.mem.write(read_stream, nil);
    }
    if !write_stream.is_null() {
        env.mem.write(write_stream, nil);
    }
}

pub const FUNCTIONS: FunctionExports = &[export_c_func!(CFStreamCreatePairWithSocketToHost(
    _, _, _, _, _
))];
