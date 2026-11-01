/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIPasteboard`.

use crate::dyld::HostConstant;
use crate::frameworks::foundation::{ns_string, NSInteger, NSUInteger};
use crate::objc::{
    id, msg, msg_class, nil, objc_classes, release, retain, ClassExports, HostObject, NSZonePtr,
};
use std::collections::HashMap;

const UIPASTEBOARD_NAME_GENERAL: &str = "com.apple.UIKit.pboard.general";
const UIPASTEBOARD_NAME_FIND: &str = "com.apple.UIKit.pboard.find";

pub const CONSTANTS: &[(&str, HostConstant)] = &[
    (
        "_UIPasteboardNameGeneral",
        HostConstant::NSString(UIPASTEBOARD_NAME_GENERAL),
    ),
    (
        "_UIPasteboardNameFind",
        HostConstant::NSString(UIPASTEBOARD_NAME_FIND),
    ),
];

#[derive(Default)]
pub struct State {
    named: HashMap<String, id>,
    unique_counter: u64,
}

#[derive(Debug)]
struct PasteboardHostObject {
    name: id,
    string: id,
    persistent: bool,
    change_count: NSUInteger,
}
impl HostObject for PasteboardHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UIPasteboard: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(
        this,
        Box::new(PasteboardHostObject {
            name: nil,
            string: nil,
            persistent: false,
            change_count: 0,
        }),
        &mut env.mem,
    )
}

+ (id)pasteboardWithName:(id)name create:(bool)create {
    if name == nil {
        return nil;
    }
    let key = ns_string::to_rust_string(env, name);
    if let Some(&pasteboard) = env.framework_state.uikit.ui_pasteboard.named.get(&key) {
        return pasteboard;
    }
    if !create {
        return nil;
    }

    let pasteboard: id = msg![env; this alloc];
    let pasteboard: id = msg![env; pasteboard initWithName:name];
    // The registry owns the allocation. Callers receive a non-owning,
    // singleton-style reference, as they do on iPhone OS.
    env.framework_state
        .uikit
        .ui_pasteboard
        .named
        .insert(key, pasteboard);
    pasteboard
}

+ (id)generalPasteboard {
    let name = ns_string::get_static_str(env, UIPASTEBOARD_NAME_GENERAL);
    let pasteboard: id = msg![env; this pasteboardWithName:name create:true];
    env.objc.borrow_mut::<PasteboardHostObject>(pasteboard).persistent = true;
    pasteboard
}

+ (id)pasteboardWithUniqueName {
    let counter = env.framework_state.uikit.ui_pasteboard.unique_counter;
    env.framework_state.uikit.ui_pasteboard.unique_counter = counter.wrapping_add(1);
    let name = ns_string::from_rust_string(env, format!("com.touchHLE.pboard.{counter}"));
    let pasteboard: id = msg![env; this pasteboardWithName:name create:true];
    release(env, name);
    pasteboard
}

+ (())removePasteboardWithName:(id)name {
    if name == nil {
        return;
    }
    let key = ns_string::to_rust_string(env, name);
    if let Some(pasteboard) = env.framework_state.uikit.ui_pasteboard.named.remove(&key) {
        release(env, pasteboard);
    }
}

- (id)initWithName:(id)name {
    retain(env, name);
    env.objc.borrow_mut::<PasteboardHostObject>(this).name = name;
    this
}

- (id)name {
    env.objc.borrow::<PasteboardHostObject>(this).name
}

- (bool)isPersistent {
    env.objc.borrow::<PasteboardHostObject>(this).persistent
}

- (())setPersistent:(bool)persistent {
    env.objc.borrow_mut::<PasteboardHostObject>(this).persistent = persistent;
}

- (NSInteger)changeCount {
    env.objc.borrow::<PasteboardHostObject>(this).change_count as NSInteger
}

- (id)string {
    env.objc.borrow::<PasteboardHostObject>(this).string
}

- (())setString:(id)string {
    if string != nil {
        retain(env, string);
    }
    let old = {
        let host = env.objc.borrow_mut::<PasteboardHostObject>(this);
        let old = std::mem::replace(&mut host.string, string);
        host.change_count = host.change_count.wrapping_add(1);
        old
    };
    if old != nil {
        release(env, old);
    }
}

- (id)strings {
    let string = env.objc.borrow::<PasteboardHostObject>(this).string;
    if string == nil {
        nil
    } else {
        msg_class![env; NSArray arrayWithObject:string]
    }
}

- (())setStrings:(id)strings {
    let count: NSUInteger = if strings == nil { 0 } else { msg![env; strings count] };
    let first: id = if count == 0 { nil } else { msg![env; strings objectAtIndex:0u32] };
    () = msg![env; this setString:first];
}

- (())dealloc {
    let (name, string) = {
        let host = env.objc.borrow::<PasteboardHostObject>(this);
        (host.name, host.string)
    };
    if name != nil {
        release(env, name);
    }
    if string != nil {
        release(env, string);
    }
    env.objc.dealloc_object(this, &mut env.mem)
}

@end

};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Environment;

    #[test]
    fn named_pasteboards_obey_create_identity_and_string_state() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let name = ns_string::from_rust_string(env, "test.board".to_owned());
        let missing: id = msg_class![env; UIPasteboard pasteboardWithName:name create:false];
        assert_eq!(missing, nil);
        let created: id = msg_class![env; UIPasteboard pasteboardWithName:name create:true];
        let found: id = msg_class![env; UIPasteboard pasteboardWithName:name create:false];
        assert_eq!(created, found);

        let value = ns_string::from_rust_string(env, "hello".to_owned());
        () = msg![env; created setString:value];
        let actual: id = msg![env; found string];
        assert_eq!(ns_string::to_rust_string(env, actual), "hello");
        let change_count: NSInteger = msg![env; found changeCount];
        assert_eq!(change_count, 1);

        () = msg_class![env; UIPasteboard removePasteboardWithName:name];
        let missing_again: id = msg_class![env; UIPasteboard pasteboardWithName:name create:false];
        assert_eq!(missing_again, nil);
        release(env, value);
        release(env, name);
    }
}
