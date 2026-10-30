/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Minimal Grand Central Dispatch queue objects.

use crate::dyld::{export_c_func, ConstantExports, FunctionExports, HostConstant};
use crate::mem::{ConstPtr, ConstVoidPtr, MutVoidPtr};
use crate::Environment;

fn new_queue_object(env: &mut Environment) -> MutVoidPtr {
    // Queue objects are opaque to callers. Scheduling is intentionally added
    // separately when a game demonstrates which dispatch operations it needs.
    env.mem.alloc_and_write(0x7175_6575_u32).cast()
}

fn dispatch_queue_create(
    env: &mut Environment,
    label: ConstPtr<u8>,
    _attr: ConstVoidPtr,
) -> MutVoidPtr {
    if !label.is_null() {
        log_dbg!(
            "dispatch_queue_create('{}')",
            env.mem.cstr_at_utf8(label).unwrap_or("<invalid UTF-8>")
        );
    }
    new_queue_object(env)
}

fn dispatch_group_create(env: &mut Environment) -> MutVoidPtr {
    // Dispatch objects are opaque. Group operations will be implemented when
    // an app demonstrates which scheduling behaviour it needs.
    new_queue_object(env)
}

pub const CONSTANTS: ConstantExports = &[(
    "__dispatch_main_q",
    HostConstant::Custom(|env| new_queue_object(env).cast_const()),
)];

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(dispatch_queue_create(_, _)),
    export_c_func!(dispatch_group_create()),
];
