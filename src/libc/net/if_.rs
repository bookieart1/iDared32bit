/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `net/if.h`

use crate::dyld::FunctionExports;
use crate::export_c_func;
use crate::libc::errno::{set_errno, ENXIO};
use crate::mem::{ConstPtr, Ptr};
use crate::Environment;

// Guest-visible interfaces are deliberately virtual rather than aliases for
// host interfaces. The indexes only need to be stable, positive identifiers;
// link state and reachability are separate contracts.
const INTERFACES: &[(&str, u32)] = &[("lo0", 1), ("en0", 2)];

// TODO: struct definition
#[allow(non_camel_case_types)]
struct if_nameindex {}

fn if_nameindex(_env: &mut Environment) -> ConstPtr<if_nameindex> {
    // TODO: implement
    Ptr::null()
}

fn if_nametoindex(env: &mut Environment, ifname: ConstPtr<u8>) -> u32 {
    let Ok(ifname) = env.mem.cstr_at_utf8(ifname) else {
        set_errno(env, ENXIO);
        return 0;
    };
    let index = INTERFACES
        .iter()
        .find_map(|&(name, index)| (name == ifname).then_some(index))
        .unwrap_or(0);
    if index == 0 {
        set_errno(env, ENXIO);
    }
    log!("if_nametoindex({ifname:?}) -> {index}");
    index
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(if_nameindex()),
    export_c_func!(if_nametoindex(_)),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libc::errno::get_errno;

    #[test]
    fn if_nametoindex_uses_stable_virtual_interfaces() {
        let mut env = Environment::new_fake_for_test();
        let lo0 = env.mem.alloc_and_write_cstr(b"lo0").cast_const();
        let en0 = env.mem.alloc_and_write_cstr(b"en0").cast_const();

        assert_eq!(if_nametoindex(&mut env, lo0), 1);
        assert_eq!(if_nametoindex(&mut env, en0), 2);
    }

    #[test]
    fn if_nametoindex_reports_an_unknown_interface() {
        let mut env = Environment::new_fake_for_test();
        let unknown = env.mem.alloc_and_write_cstr(b"missing0").cast_const();

        assert_eq!(if_nametoindex(&mut env, unknown), 0);
        assert_eq!(get_errno(&mut env), ENXIO);
    }
}
