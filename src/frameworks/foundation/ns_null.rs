/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSNull`.

use crate::frameworks::foundation::ns_string;
use crate::objc::{id, objc_classes, ClassExports, TrivialHostObject};

#[derive(Default)]
pub struct State {
    null: Option<id>,
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

// This is a singleton that takes the place of nil in collections which don't
// allow that value.
@implementation NSNull: NSObject

+ (id)null {
    if let Some(null) = env.framework_state.foundation.ns_null.null {
        null
    } else {
        let new = env.objc.alloc_static_object(
            this,
            Box::new(TrivialHostObject),
            &mut env.mem
        );
        env.framework_state.foundation.ns_null.null = Some(new);
        new
   }
}

- (id)retain { this }
- (())release {}
- (id)autorelease { this }

- (id)description {
    // This is the historical Foundation representation of the singleton.
    // It is also what collection and property-list diagnostics expect when
    // they encounter an explicit null placeholder.
    ns_string::get_static_str(env, "<null>")
}

@end

};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objc::{msg, msg_class};
    use crate::Environment;

    #[test]
    fn description_uses_foundation_null_representation() {
        let mut env = Environment::new_fake_for_test();
        let env = &mut env;
        let null: id = msg_class![env; NSNull null];
        let description: id = msg![env; null description];

        assert_eq!(ns_string::to_rust_string(env, description), "<null>");
        assert_eq!(null, msg_class![env; NSNull null]);
    }
}
