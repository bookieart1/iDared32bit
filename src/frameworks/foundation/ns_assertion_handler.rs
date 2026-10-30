/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSAssertionHandler`.

use super::NSUInteger;
use crate::objc::{id, msg, objc_classes, ClassExports, SEL};

#[derive(Default)]
pub struct State {
    handler: Option<id>,
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSAssertionHandler: NSObject

+ (id)currentHandler {
    if let Some(handler) = env.framework_state.foundation.ns_assertion_handler.handler {
        handler
    } else {
        // Apple's implementation stores one handler per thread. The handler is
        // stateless here, so a process-wide singleton is sufficient and avoids
        // making assertion handling depend on NSThread's dictionary support.
        let handler: id = msg![env; this new];
        env.framework_state.foundation.ns_assertion_handler.handler = Some(handler);
        handler
    }
}

- (())handleFailureInMethod:(SEL)selector
                     object:(id)object
                       file:(id)file
                 lineNumber:(NSUInteger)line_number
                description:(id)description {
    // Assertions inside legacy SDKs often guard optional functionality that
    // touchHLE does not expose yet. Logging and continuing is more useful than
    // aborting the entire guest process, and matches the emulator's treatment
    // of many other recoverable compatibility gaps.
    log!(
        "Ignoring Objective-C assertion in method {} on {:?} at {:?}:{} (description {:?})",
        selector.as_str(&env.mem),
        object,
        file,
        line_number,
        description,
    );
}

- (())handleFailureInFunction:(id)function
                          file:(id)file
                    lineNumber:(NSUInteger)line_number
                   description:(id)description {
    log!(
        "Ignoring Objective-C assertion in function {:?} at {:?}:{} (description {:?})",
        function,
        file,
        line_number,
        description,
    );
}

@end

};
