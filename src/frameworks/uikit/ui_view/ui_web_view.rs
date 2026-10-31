/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIWebView`.

use crate::frameworks::core_graphics::CGRect;
use crate::frameworks::foundation::ns_string::to_rust_string;
use crate::frameworks::foundation::NSUInteger;
use crate::frameworks::uikit::ui_view::UIViewHostObject;
use crate::msg;
use crate::objc::{id, msg_class, msg_super, nil, objc_classes, release, ClassExports};
use crate::Environment;
use std::borrow::Cow;

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UIWebView: UIView

- (id)initWithFrame:(CGRect)frame {
    let this: id = msg_super![env; this initWithFrame:frame];
    add_scroll_view(env, this);
    this
}

// NSCoding implementation
- (id)initWithCoder:(id)coder {
    let this: id = msg_super![env; this initWithCoder:coder];
    add_scroll_view(env, this);
    this
}

- (())setScalesPageToFit:(bool)_scales {
    // TODO
}
- (())setDelegate:(id)delegate {
    // UIWebView's delegate property was assign (non-retaining) on old iOS.
    env.objc.borrow_mut::<UIViewHostObject>(this).ui_web_view_delegate = delegate;
}
- (id)delegate {
    env.objc.borrow::<UIViewHostObject>(this).ui_web_view_delegate
}
// Pages are never loaded (see loadRequest:), so nothing is ever loading.
- (bool)isLoading {
    false
}
- (())stopLoading {}

- (())loadRequest:(id)request { // NSURLRequest*
    let url_string = if request != nil {
        let url = msg![env; request URL];
        let url_desc = msg![env; url description];
        to_rust_string(env, url_desc)
    } else {
        Cow::default()
    };
    log!("TODO: [(UIWebView*) {:?} loadRequest:{:?} ({})]", this, request, url_string);
}

- (())loadHTMLString:(id)html baseURL:(id)base_url { // NSString*, NSURL*
    // UIWebView is intentionally still a no-renderer implementation: touchHLE
    // does not embed WebKit, so it cannot build a DOM, run JavaScript, or claim
    // that a frame finished loading. Exporting the real selector is nevertheless
    // important. Framework wrappers commonly obtain its method signature and
    // invoke it through NSInvocation before any rendering is relevant.
    let length: NSUInteger = if html == nil {
        0
    } else {
        msg![env; html length]
    };
    let base_url = if base_url == nil {
        Cow::default()
    } else {
        let description: id = msg![env; base_url description];
        to_rust_string(env, description)
    };
    log!(
        "TODO: [(UIWebView*) {:?} loadHTMLString:<{} UTF-16 units> baseURL:{}] (call accepted; no WebKit renderer)",
        this,
        length,
        base_url
    );
}

- (id)stringByEvaluatingJavaScriptFromString:(id)script { // NSString*
    let script = if script == nil {
        Cow::default()
    } else {
        to_rust_string(env, script)
    };
    // touchHLE does not currently embed a web engine, and loadRequest: does
    // not create a document. UIWebView returns nil when a script cannot be
    // evaluated, which is the truthful result for the current no-page model.
    log!(
        "TODO: [(UIWebView*) {:?} stringByEvaluatingJavaScriptFromString:{}] (no loaded document)",
        this,
        script
    );
    nil
}

@end

};

/// The page is shown in a scroll view, the web view's first subview.
/// Some games reach it through -subviews to hide its shadows.
fn add_scroll_view(env: &mut Environment, web_view: id) {
    let bounds: CGRect = msg![env; web_view bounds];
    let scroll_view: id = msg_class![env; UIScrollView alloc];
    let scroll_view: id = msg![env; scroll_view initWithFrame:bounds];
    () = msg![env; web_view addSubview:scroll_view];
    release(env, scroll_view);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_html_string_is_an_exported_instance_method() {
        let mut env = Environment::new_fake_for_test();
        let env = &mut env;
        let class = env.objc.get_known_class("UIWebView", &mut env.mem);
        let selector = env
            .objc
            .register_host_selector("loadHTMLString:baseURL:".to_string(), &mut env.mem);

        assert!(env.objc.class_has_method(class, selector));
        assert_eq!(
            env.objc.class_get_method_types(class, selector),
            Some("v@:@@")
        );
    }
}
