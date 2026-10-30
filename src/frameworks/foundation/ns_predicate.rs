/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSPredicate`.

use crate::objc::{
    autorelease, id, msg, nil, objc_classes, release, ClassExports, HostObject, NSZonePtr,
};

struct NSPredicateHostObject {
    format: id,
}
impl HostObject for NSPredicateHostObject {}

fn init_with_format(env: &mut crate::Environment, this: id, format: id) -> id {
    let format = if format == nil {
        nil
    } else {
        msg![env; format copy]
    };
    *env.objc.borrow_mut(this) = NSPredicateHostObject { format };
    this
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSPredicate: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(NSPredicateHostObject { format: nil });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (id)predicateWithFormat:(id)format, ..._args {
    let predicate: id = msg![env; this alloc];
    let predicate = init_with_format(env, predicate, format);
    autorelease(env, predicate)
}

+ (id)predicateWithFormat:(id)format argumentArray:(id)_arguments {
    let predicate: id = msg![env; this alloc];
    let predicate = init_with_format(env, predicate, format);
    autorelease(env, predicate)
}

- (id)initWithFormat:(id)format, ..._args {
    init_with_format(env, this, format)
}

- (id)initWithFormat:(id)format argumentArray:(id)_arguments {
    init_with_format(env, this, format)
}

- (id)predicateFormat {
    env.objc.borrow::<NSPredicateHostObject>(this).format
}

- (bool)evaluateWithObject:(id)_object {
    // This is a compatibility fallback until predicate parsing is implemented.
    // Returning true preserves the candidate set instead of discarding data.
    log_once!("TODO: NSPredicate evaluation is not implemented; accepting the object");
    true
}

- (())dealloc {
    let format = env.objc.borrow::<NSPredicateHostObject>(this).format;
    release(env, format);
    env.objc.dealloc_object(this, &mut env.mem)
}

@end

};
