/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSHTTPCookieStorage`.
//!
//! Cookies are currently kept in memory for the lifetime of the app. URL
//! filtering is not implemented yet, so `cookiesForURL:` returns the same
//! collection as `cookies`.

use super::{ns_array, NSUInteger};
use crate::dyld::{ConstantExports, HostConstant};
use crate::objc::{
    autorelease, id, msg, nil, objc_classes, release, retain, ClassExports, HostObject, NSZonePtr,
};

type NSHTTPCookieAcceptPolicy = NSUInteger;
const NSHTTPCookieAcceptPolicyOnlyFromMainDocumentDomain: NSHTTPCookieAcceptPolicy = 2;

pub const CONSTANTS: ConstantExports = &[
    (
        "_NSHTTPCookieManagerAcceptPolicyChangedNotification",
        HostConstant::NSString("NSHTTPCookieManagerAcceptPolicyChangedNotification"),
    ),
    (
        "_NSHTTPCookieManagerCookiesChangedNotification",
        HostConstant::NSString("NSHTTPCookieManagerCookiesChangedNotification"),
    ),
];

#[derive(Default)]
pub struct State {
    /// `NSHTTPCookieStorage*`
    shared_storage: Option<id>,
}

struct NSHTTPCookieStorageHostObject {
    /// Retained `NSHTTPCookie*` objects.
    cookies: Vec<id>,
    accept_policy: NSHTTPCookieAcceptPolicy,
}
impl HostObject for NSHTTPCookieStorageHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSHTTPCookieStorage: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(NSHTTPCookieStorageHostObject {
        cookies: Vec::new(),
        accept_policy: NSHTTPCookieAcceptPolicyOnlyFromMainDocumentDomain,
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (id)sharedHTTPCookieStorage {
    if let Some(storage) = env.framework_state.foundation.ns_http_cookie_storage.shared_storage {
        storage
    } else {
        let storage: id = msg![env; this new];
        env.framework_state.foundation.ns_http_cookie_storage.shared_storage = Some(storage);
        storage
    }
}

- (())dealloc {
    let host_object = env.objc.borrow_mut::<NSHTTPCookieStorageHostObject>(this);
    let cookies = std::mem::take(&mut host_object.cookies);
    for cookie in cookies {
        release(env, cookie);
    }
    env.objc.dealloc_object(this, &mut env.mem)
}

- (id)cookies {
    let cookies = env
        .objc
        .borrow::<NSHTTPCookieStorageHostObject>(this)
        .cookies
        .clone();
    for &cookie in &cookies {
        retain(env, cookie);
    }
    let array = ns_array::from_vec(env, cookies);
    autorelease(env, array)
}

- (id)cookiesForURL:(id)_url { // NSURL*
    msg![env; this cookies]
}

- (())setCookie:(id)cookie { // NSHTTPCookie*
    if cookie == nil {
        return;
    }
    let already_present = env
        .objc
        .borrow::<NSHTTPCookieStorageHostObject>(this)
        .cookies
        .contains(&cookie);
    if already_present {
        return;
    }
    retain(env, cookie);
    env.objc
        .borrow_mut::<NSHTTPCookieStorageHostObject>(this)
        .cookies
        .push(cookie);
}

- (())deleteCookie:(id)cookie { // NSHTTPCookie*
    let host_object = env.objc.borrow_mut::<NSHTTPCookieStorageHostObject>(this);
    let Some(index) = host_object.cookies.iter().position(|&candidate| candidate == cookie) else {
        return;
    };
    let removed = host_object.cookies.remove(index);
    release(env, removed);
}

- (())setCookies:(id)cookies // NSArray*
             forURL:(id)_url // NSURL*
     mainDocumentURL:(id)_main_document_url { // NSURL*
    if cookies == nil {
        return;
    }
    let count: NSUInteger = msg![env; cookies count];
    for index in 0..count {
        let cookie: id = msg![env; cookies objectAtIndex:index];
        let _: () = msg![env; this setCookie:cookie];
    }
}

- (NSHTTPCookieAcceptPolicy)cookieAcceptPolicy {
    env.objc
        .borrow::<NSHTTPCookieStorageHostObject>(this)
        .accept_policy
}

- (())setCookieAcceptPolicy:(NSHTTPCookieAcceptPolicy)policy {
    env.objc
        .borrow_mut::<NSHTTPCookieStorageHostObject>(this)
        .accept_policy = policy;
}

- (id)sortedCookiesUsingDescriptors:(id)descriptors { // NSArray<NSSortDescriptor*>*
    let cookies: id = msg![env; this cookies];
    msg![env; cookies sortedArrayUsingDescriptors:descriptors]
}

@end

};
