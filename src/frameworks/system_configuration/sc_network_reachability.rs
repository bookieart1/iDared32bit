/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! SCNetworkReachability

use crate::abi::GuestFunction;
use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::core_foundation::cf_allocator::{kCFAllocatorDefault, CFAllocatorRef};
use crate::frameworks::core_foundation::CFTypeRef;
use crate::libc::sys::socket::sockaddr;
use crate::mem::{ConstPtr, MutPtr, MutVoidPtr, Ptr};
use crate::objc::{msg, objc_classes, Class, ClassExports, HostObject};
use crate::Environment;
use std::net::SocketAddrV4;

type SCNetworkReachabilityFlags = u32;
const kSCNetworkReachabilityFlagsReachable: SCNetworkReachabilityFlags = 1 << 1;
const kSCNetworkReachabilityFlagsIsDirect: SCNetworkReachabilityFlags = 1 << 17;

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

// SCNetworkReachabilityRef is not explicitly stated to be CFType-based type,
// but the result of "Create" or "Copy" functions here is expected to be
// released with CFRelease().
@implementation _touchHLE_SCNetworkReachability: NSObject
@end

};

struct SCNetworkReachabilityHostObject {
    address: Option<SocketAddrV4>,
    callback: Option<(GuestFunction, MutVoidPtr)>,
}
impl HostObject for SCNetworkReachabilityHostObject {}

// See comment for `_touchHLE_SCNetworkReachability` class
type SCNetworkReachabilityRef = CFTypeRef;

fn SCNetworkReachabilityCreateWithName(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    name: ConstPtr<u8>,
) -> SCNetworkReachabilityRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default()); // unimplemented
    if env
        .bundle
        .bundle_identifier()
        .starts_with("com.chillingo.cuttherope")
        && env.mem.cstr_at_utf8(name).unwrap() == "chillingo-crystal.appspot.com"
    {
        log!("Applying game-specific hack for Cut the Rope: SCNetworkReachabilityCreateWithName(\"chillingo-crystal.appspot.com\") returns NULL");
        return Ptr::null();
    }
    let isa = env
        .objc
        .get_known_class("_touchHLE_SCNetworkReachability", &mut env.mem);
    let res = env.objc.alloc_object(
        isa,
        Box::new(SCNetworkReachabilityHostObject {
            address: None,
            callback: None,
        }), // TODO
        &mut env.mem,
    );
    log!(
        "TODO: SCNetworkReachabilityCreateWithName({:?}, {:?} {:?}) -> {:?}",
        allocator,
        name,
        env.mem.cstr_at_utf8(name),
        res
    );
    res
}

fn SCNetworkReachabilityCreateWithAddress(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    address: ConstPtr<sockaddr>,
) -> SCNetworkReachabilityRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default()); // unimplemented
    let isa = env
        .objc
        .get_known_class("_touchHLE_SCNetworkReachability", &mut env.mem);
    let address_val = env.mem.read(address);
    let res = env.objc.alloc_object(
        isa,
        Box::new(SCNetworkReachabilityHostObject {
            address: Some(address_val.to_sockaddr_v4()),
            callback: None,
        }),
        &mut env.mem,
    );
    log_dbg!(
        "SCNetworkReachabilityCreateWithAddress({:?}, {:?} ({})) -> {:?}",
        allocator,
        address,
        address_val.to_sockaddr_v4(),
        res
    );
    res
}

fn SCNetworkReachabilityGetFlags(
    env: &mut Environment,
    target: SCNetworkReachabilityRef,
    flags: MutPtr<SCNetworkReachabilityFlags>,
) -> bool {
    if !env.options.network_access {
        log_dbg!(
            "Network access is disabled, SCNetworkReachabilityGetFlags({:?}, {:?}) -> false",
            target,
            flags
        );
        // The query itself succeeded; an empty flag set means offline. Some
        // older reachability wrappers do not safely handle a failed query.
        if !flags.is_null() {
            env.mem.write(flags, 0);
        }
        return true;
    }

    let target_class: Class = msg![env; target class];
    assert_eq!(
        target_class,
        env.objc
            .get_known_class("_touchHLE_SCNetworkReachability", &mut env.mem)
    );
    let host_object = env.objc.borrow::<SCNetworkReachabilityHostObject>(target);
    if let Some(addr) = host_object.address {
        if addr.ip().is_link_local() {
            log_dbg!(
                "SCNetworkReachabilityGetFlags({:?}, {:?}) -> true",
                target,
                flags
            );
            // Those corresponds to local WiFi connection on a real iOS device
            // TODO: actually check for the connectivity
            // (but do we _really_ need it?)
            let out_flags =
                kSCNetworkReachabilityFlagsReachable | kSCNetworkReachabilityFlagsIsDirect;
            env.mem.write(flags, out_flags);
            return true;
        }
    }
    if !flags.is_null() {
        env.mem.write(flags, 0);
    }
    log!(
        "SCNetworkReachabilityGetFlags({:?}, {:?}) -> true (not reachable)",
        target,
        flags
    );
    true
}

fn SCNetworkReachabilitySetCallback(
    env: &mut Environment,
    target: SCNetworkReachabilityRef,
    callout: GuestFunction, // SCNetworkReachabilityCallBack
    context: MutVoidPtr,    // SCNetworkReachabilityContext *
) -> bool {
    let target_class: Class = msg![env; target class];
    assert_eq!(
        target_class,
        env.objc
            .get_known_class("_touchHLE_SCNetworkReachability", &mut env.mem)
    );
    env.objc
        .borrow_mut::<SCNetworkReachabilityHostObject>(target)
        .callback = if callout.addr_without_thumb_bit() == 0 {
        None
    } else {
        Some((callout, context))
    };
    log!(
        "SCNetworkReachabilitySetCallback({:?}, {:?}, {:?}) -> TRUE",
        target,
        callout,
        context
    );
    true
}

fn SCNetworkReachabilityScheduleWithRunLoop(
    env: &mut Environment,
    target: SCNetworkReachabilityRef,
    _run_loop: CFTypeRef,
    _run_loop_mode: CFTypeRef,
) -> bool {
    let target_class: Class = msg![env; target class];
    assert_eq!(
        target_class,
        env.objc
            .get_known_class("_touchHLE_SCNetworkReachability", &mut env.mem)
    );
    // Registration succeeds, but no callback is fired: the object reports a
    // stable offline state through GetFlags. This prevents retired ad/network
    // SDKs from spinning or dereferencing half-initialized reachability state.
    true
}

fn SCNetworkReachabilityUnscheduleFromRunLoop(
    env: &mut Environment,
    target: SCNetworkReachabilityRef,
    _run_loop: CFTypeRef,
    _run_loop_mode: CFTypeRef,
) -> bool {
    let target_class: Class = msg![env; target class];
    assert_eq!(
        target_class,
        env.objc
            .get_known_class("_touchHLE_SCNetworkReachability", &mut env.mem)
    );
    true
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(SCNetworkReachabilityCreateWithName(_, _)),
    export_c_func!(SCNetworkReachabilityCreateWithAddress(_, _)),
    export_c_func!(SCNetworkReachabilityGetFlags(_, _)),
    export_c_func!(SCNetworkReachabilitySetCallback(_, _, _)),
    export_c_func!(SCNetworkReachabilityScheduleWithRunLoop(_, _, _)),
    export_c_func!(SCNetworkReachabilityUnscheduleFromRunLoop(_, _, _)),
];
