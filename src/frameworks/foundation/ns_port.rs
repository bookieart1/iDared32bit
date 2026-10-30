/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Minimal `NSPort` and `NSMachPort` support.
//!
//! The emulated run loop does not yet deliver Mach messages. These objects
//! nevertheless provide the lifecycle and scheduling surface used by older
//! networking and engine code to keep worker run loops alive.

use crate::objc::{autorelease, id, msg, objc_classes, ClassExports, HostObject, NSZonePtr};

#[derive(Default)]
struct NSPortHostObject {
    valid: bool,
}
impl HostObject for NSPortHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSPort: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(
        this,
        Box::new(NSPortHostObject { valid: true }),
        &mut env.mem,
    )
}

+ (id)port {
    let port: id = msg![env; this new];
    autorelease(env, port)
}

- (bool)isValid {
    env.objc.borrow::<NSPortHostObject>(this).valid
}

- (())invalidate {
    env.objc.borrow_mut::<NSPortHostObject>(this).valid = false;
}

- (())scheduleInRunLoop:(id)run_loop forMode:(id)mode {
    () = msg![env; run_loop addPort:this forMode:mode];
}

- (())removeFromRunLoop:(id)run_loop forMode:(id)mode {
    () = msg![env; run_loop removePort:this forMode:mode];
}

- (())dealloc {
    env.objc.dealloc_object(this, &mut env.mem);
}

@end

@implementation NSMachPort: NSPort

+ (id)port {
    let port: id = msg![env; this new];
    autorelease(env, port)
}

@end

};
