/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSUbiquitousKeyValueStore`.
//!
//! touchHLE cannot access the host user's iCloud account, so this provides the
//! local in-memory part of the API only. `synchronize` reports failure, just as
//! it does on iOS when the app lacks the required iCloud entitlement.

use crate::dyld::{ConstantExports, HostConstant};
use crate::objc::{
    autorelease, id, msg, msg_class, nil, objc_classes, release, ClassExports, HostObject,
    NSZonePtr,
};

pub const CONSTANTS: ConstantExports = &[
    (
        "_NSUbiquitousKeyValueStoreDidChangeExternallyNotification",
        HostConstant::NSString("NSUbiquitousKeyValueStoreDidChangeExternallyNotification"),
    ),
    (
        "_NSUbiquitousKeyValueStoreChangeReasonKey",
        HostConstant::NSString("NSUbiquitousKeyValueStoreChangeReasonKey"),
    ),
    (
        "_NSUbiquitousKeyValueStoreChangedKeysKey",
        HostConstant::NSString("NSUbiquitousKeyValueStoreChangedKeysKey"),
    ),
];

#[derive(Default)]
pub struct State {
    /// `NSUbiquitousKeyValueStore*`
    default_store: Option<id>,
}

struct NSUbiquitousKeyValueStoreHostObject {
    /// `NSMutableDictionary*`, retained.
    values: id,
}
impl HostObject for NSUbiquitousKeyValueStoreHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSUbiquitousKeyValueStore: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let values: id = msg_class![env; NSMutableDictionary new];
    let host_object = Box::new(NSUbiquitousKeyValueStoreHostObject { values });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (id)defaultStore {
    if let Some(store) = env.framework_state.foundation.ns_ubiquitous_key_value_store.default_store {
        store
    } else {
        let store: id = msg![env; this new];
        env.framework_state.foundation.ns_ubiquitous_key_value_store.default_store = Some(store);
        store
    }
}

- (())dealloc {
    let values = env
        .objc
        .borrow::<NSUbiquitousKeyValueStoreHostObject>(this)
        .values;
    release(env, values);
    env.objc.dealloc_object(this, &mut env.mem)
}

- (id)objectForKey:(id)key {
    let values = env
        .objc
        .borrow::<NSUbiquitousKeyValueStoreHostObject>(this)
        .values;
    msg![env; values objectForKey:key]
}

- (id)stringForKey:(id)key {
    msg![env; this objectForKey:key]
}

- (id)dataForKey:(id)key {
    msg![env; this objectForKey:key]
}

- (id)arrayForKey:(id)key {
    msg![env; this objectForKey:key]
}

- (id)dictionaryForKey:(id)key {
    msg![env; this objectForKey:key]
}

- (bool)boolForKey:(id)key {
    let value: id = msg![env; this objectForKey:key];
    msg![env; value boolValue]
}

- (f64)doubleForKey:(id)key {
    let value: id = msg![env; this objectForKey:key];
    msg![env; value doubleValue]
}

- (i64)longLongForKey:(id)key {
    let value: id = msg![env; this objectForKey:key];
    msg![env; value longLongValue]
}

- (id)dictionaryRepresentation {
    let values = env
        .objc
        .borrow::<NSUbiquitousKeyValueStoreHostObject>(this)
        .values;
    let copy: id = msg![env; values copy];
    autorelease(env, copy)
}

- (())setObject:(id)object forKey:(id)key {
    let values = env
        .objc
        .borrow::<NSUbiquitousKeyValueStoreHostObject>(this)
        .values;
    if object == nil {
        msg![env; values removeObjectForKey:key]
    } else {
        msg![env; values setObject:object forKey:key]
    }
}

- (())setString:(id)value forKey:(id)key {
    msg![env; this setObject:value forKey:key]
}

- (())setData:(id)value forKey:(id)key {
    msg![env; this setObject:value forKey:key]
}

- (())setArray:(id)value forKey:(id)key {
    msg![env; this setObject:value forKey:key]
}

- (())setDictionary:(id)value forKey:(id)key {
    msg![env; this setObject:value forKey:key]
}

- (())setBool:(bool)value forKey:(id)key {
    let number: id = msg_class![env; NSNumber numberWithBool:value];
    msg![env; this setObject:number forKey:key]
}

- (())setDouble:(f64)value forKey:(id)key {
    let number: id = msg_class![env; NSNumber numberWithDouble:value];
    msg![env; this setObject:number forKey:key]
}

- (())setLongLong:(i64)value forKey:(id)key {
    let number: id = msg_class![env; NSNumber numberWithLongLong:value];
    msg![env; this setObject:number forKey:key]
}

- (())removeObjectForKey:(id)key {
    let values = env
        .objc
        .borrow::<NSUbiquitousKeyValueStoreHostObject>(this)
        .values;
    msg![env; values removeObjectForKey:key]
}

- (bool)synchronize {
    // No iCloud entitlement or account is available to the emulator.
    false
}

@end

};
