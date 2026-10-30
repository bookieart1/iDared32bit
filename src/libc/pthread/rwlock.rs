/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Guest read/write-lock interface.
//!
//! The emulator currently serializes readers as well as writers. This is more
//! restrictive than a native read/write lock, but preserves mutual exclusion
//! and is safe for the common compatibility case.

use super::mutex::{
    pthread_mutex_destroy, pthread_mutex_init, pthread_mutex_lock, pthread_mutex_t,
    pthread_mutex_trylock, pthread_mutex_unlock,
};
use crate::dyld::{export_c_func, FunctionExports};
use crate::mem::{ConstVoidPtr, MutPtr, Ptr};
use crate::Environment;

pub type pthread_rwlock_t = pthread_mutex_t;

fn pthread_rwlock_init(
    env: &mut Environment,
    rwlock: MutPtr<pthread_rwlock_t>,
    _attr: ConstVoidPtr,
) -> i32 {
    pthread_mutex_init(env, rwlock.cast(), Ptr::null())
}

fn pthread_rwlock_destroy(env: &mut Environment, rwlock: MutPtr<pthread_rwlock_t>) -> i32 {
    pthread_mutex_destroy(env, rwlock.cast())
}

fn pthread_rwlock_rdlock(env: &mut Environment, rwlock: MutPtr<pthread_rwlock_t>) -> i32 {
    pthread_mutex_lock(env, rwlock.cast())
}

fn pthread_rwlock_wrlock(env: &mut Environment, rwlock: MutPtr<pthread_rwlock_t>) -> i32 {
    pthread_mutex_lock(env, rwlock.cast())
}

fn pthread_rwlock_tryrdlock(env: &mut Environment, rwlock: MutPtr<pthread_rwlock_t>) -> i32 {
    pthread_mutex_trylock(env, rwlock.cast())
}

fn pthread_rwlock_trywrlock(env: &mut Environment, rwlock: MutPtr<pthread_rwlock_t>) -> i32 {
    pthread_mutex_trylock(env, rwlock.cast())
}

fn pthread_rwlock_unlock(env: &mut Environment, rwlock: MutPtr<pthread_rwlock_t>) -> i32 {
    pthread_mutex_unlock(env, rwlock.cast())
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(pthread_rwlock_init(_, _)),
    export_c_func!(pthread_rwlock_destroy(_)),
    export_c_func!(pthread_rwlock_rdlock(_)),
    export_c_func!(pthread_rwlock_wrlock(_)),
    export_c_func!(pthread_rwlock_tryrdlock(_)),
    export_c_func!(pthread_rwlock_trywrlock(_)),
    export_c_func!(pthread_rwlock_unlock(_)),
];
