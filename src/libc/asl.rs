/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Minimal Apple System Log (ASL) compatibility.
//!
//! ASL was primarily a logging facility. It was deprecated by Apple long ago,
//! but a number of older apps and bundled SDKs still initialise it during
//! startup. Keeping lightweight guest-side handles and accepting messages is
//! sufficient for those callers; ASL persistence and querying are deliberately
//! not emulated.

use crate::abi::{DotDotDot, VaList};
use crate::dyld::{export_c_func, FunctionExports};
use crate::mem::{ConstPtr, MutPtr, Ptr, SafeRead};
use crate::Environment;

#[repr(C, packed)]
struct OpaqueAslObject {
    _filler: u8,
}
unsafe impl SafeRead for OpaqueAslObject {}

type AslObjectRef = MutPtr<OpaqueAslObject>;

fn new_object(env: &mut Environment) -> AslObjectRef {
    env.mem.alloc_and_write(OpaqueAslObject { _filler: 0 })
}

fn asl_open(
    env: &mut Environment,
    ident: ConstPtr<u8>,
    facility: ConstPtr<u8>,
    opts: u32,
) -> AslObjectRef {
    log_dbg!("asl_open({:?}, {:?}, {:#x})", ident, facility, opts);
    new_object(env)
}

fn asl_close(env: &mut Environment, client: AslObjectRef) {
    if !client.is_null() {
        env.mem.free(client.cast());
    }
}

fn asl_new(env: &mut Environment, _type: u32) -> AslObjectRef {
    new_object(env)
}

fn asl_free(env: &mut Environment, object: AslObjectRef) {
    if !object.is_null() {
        env.mem.free(object.cast());
    }
}

fn asl_set(
    _env: &mut Environment,
    _message: AslObjectRef,
    _key: ConstPtr<u8>,
    _value: ConstPtr<u8>,
) -> i32 {
    0
}

fn asl_unset(_env: &mut Environment, _message: AslObjectRef, _key: ConstPtr<u8>) -> i32 {
    0
}

fn asl_get(
    _env: &mut Environment,
    _message: AslObjectRef,
    _key: ConstPtr<u8>,
) -> ConstPtr<u8> {
    Ptr::null()
}

fn asl_send(
    _env: &mut Environment,
    _client: AslObjectRef,
    _message: AslObjectRef,
) -> i32 {
    0
}

fn asl_log(
    _env: &mut Environment,
    _client: AslObjectRef,
    _message: AslObjectRef,
    _level: i32,
    _format: ConstPtr<u8>,
    _args: DotDotDot,
) -> i32 {
    0
}

fn asl_vlog(
    _env: &mut Environment,
    _client: AslObjectRef,
    _message: AslObjectRef,
    _level: i32,
    _format: ConstPtr<u8>,
    _args: VaList,
) -> i32 {
    0
}

fn asl_set_filter(_env: &mut Environment, _client: AslObjectRef, filter: i32) -> i32 {
    filter
}

fn asl_add_log_file(_env: &mut Environment, _client: AslObjectRef, _fd: i32) -> i32 {
    0
}

fn asl_remove_log_file(_env: &mut Environment, _client: AslObjectRef, _fd: i32) -> i32 {
    0
}

fn asl_search(
    _env: &mut Environment,
    _client: AslObjectRef,
    _query: AslObjectRef,
) -> AslObjectRef {
    Ptr::null()
}

fn aslresponse_next(_env: &mut Environment, _response: AslObjectRef) -> AslObjectRef {
    Ptr::null()
}

fn aslresponse_free(_env: &mut Environment, _response: AslObjectRef) {}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(asl_open(_, _, _)),
    export_c_func!(asl_close(_)),
    export_c_func!(asl_new(_)),
    export_c_func!(asl_free(_)),
    export_c_func!(asl_set(_, _, _)),
    export_c_func!(asl_unset(_, _)),
    export_c_func!(asl_get(_, _)),
    export_c_func!(asl_send(_, _)),
    export_c_func!(asl_log(_, _, _, _, _)),
    export_c_func!(asl_vlog(_, _, _, _, _)),
    export_c_func!(asl_set_filter(_, _)),
    export_c_func!(asl_add_log_file(_, _)),
    export_c_func!(asl_remove_log_file(_, _)),
    export_c_func!(asl_search(_, _)),
    export_c_func!(aslresponse_next(_)),
    export_c_func!(aslresponse_free(_)),
];
