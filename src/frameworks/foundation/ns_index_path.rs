/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSIndexPath`.

use super::{
    NSComparisonResult, NSInteger, NSOrderedAscending, NSOrderedDescending, NSOrderedSame,
    NSUInteger,
};
use crate::mem::ConstPtr;
use crate::objc::{
    autorelease, id, msg, nil, objc_classes, retain, ClassExports, HostObject, NSZonePtr,
};

#[derive(Clone, Debug, Default)]
struct NSIndexPathHostObject {
    indexes: Vec<NSUInteger>,
}
impl HostObject for NSIndexPathHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSIndexPath: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(
        this,
        Box::new(NSIndexPathHostObject::default()),
        &mut env.mem,
    )
}

+ (id)indexPathWithIndex:(NSUInteger)index {
    let path: id = msg![env; this alloc];
    let path: id = msg![env; path initWithIndex:index];
    autorelease(env, path)
}

+ (id)indexPathWithIndexes:(ConstPtr<NSUInteger>)indexes length:(NSUInteger)length {
    let path: id = msg![env; this alloc];
    let path: id = msg![env; path initWithIndexes:indexes length:length];
    autorelease(env, path)
}

// UIKit's table-view additions to NSIndexPath. These are implemented here so
// callers see the same immutable index-path object regardless of which
// framework first causes the class to be registered.
+ (id)indexPathForRow:(NSInteger)row inSection:(NSInteger)section {
    let values = [section as NSUInteger, row as NSUInteger];
    let path: id = msg![env; this alloc];
    {
        let host = env.objc.borrow_mut::<NSIndexPathHostObject>(path);
        host.indexes.extend(values);
    }
    autorelease(env, path)
}

+ (id)indexPathForItem:(NSInteger)item inSection:(NSInteger)section {
    msg![env; this indexPathForRow:item inSection:section]
}

- (id)initWithIndex:(NSUInteger)index {
    env.objc.borrow_mut::<NSIndexPathHostObject>(this).indexes = vec![index];
    this
}

- (id)initWithIndexes:(ConstPtr<NSUInteger>)indexes length:(NSUInteger)length {
    let values = (0..length).map(|offset| env.mem.read(indexes + offset)).collect();
    env.objc.borrow_mut::<NSIndexPathHostObject>(this).indexes = values;
    this
}

- (NSUInteger)length {
    env.objc.borrow::<NSIndexPathHostObject>(this).indexes.len().try_into().unwrap()
}

- (NSUInteger)indexAtPosition:(NSUInteger)position {
    env.objc.borrow::<NSIndexPathHostObject>(this).indexes[position as usize]
}

- (id)indexPathByAddingIndex:(NSUInteger)index {
    let mut indexes = env.objc.borrow::<NSIndexPathHostObject>(this).indexes.clone();
    indexes.push(index);
    let class = env.objc.get_known_class("NSIndexPath", &mut env.mem);
    let path = env.objc.alloc_object(
        class,
        Box::new(NSIndexPathHostObject { indexes }),
        &mut env.mem,
    );
    autorelease(env, path)
}

- (id)indexPathByRemovingLastIndex {
    let mut indexes = env.objc.borrow::<NSIndexPathHostObject>(this).indexes.clone();
    indexes.pop();
    let class = env.objc.get_known_class("NSIndexPath", &mut env.mem);
    let path = env.objc.alloc_object(
        class,
        Box::new(NSIndexPathHostObject { indexes }),
        &mut env.mem,
    );
    autorelease(env, path)
}

- (NSInteger)section {
    env.objc.borrow::<NSIndexPathHostObject>(this).indexes.first().copied().unwrap_or(0) as NSInteger
}

- (NSInteger)row {
    env.objc.borrow::<NSIndexPathHostObject>(this).indexes.get(1).copied().unwrap_or(0) as NSInteger
}

- (NSInteger)item {
    msg![env; this row]
}

- (NSComparisonResult)compare:(id)other {
    let left = &env.objc.borrow::<NSIndexPathHostObject>(this).indexes;
    let right = &env.objc.borrow::<NSIndexPathHostObject>(other).indexes;
    match left.cmp(right) {
        std::cmp::Ordering::Less => NSOrderedAscending,
        std::cmp::Ordering::Equal => NSOrderedSame,
        std::cmp::Ordering::Greater => NSOrderedDescending,
    }
}

- (bool)isEqual:(id)other {
    if this == other {
        return true;
    }
    if other == nil {
        return false;
    }
    let class = env.objc.get_known_class("NSIndexPath", &mut env.mem);
    let other_class = crate::objc::ObjC::read_isa(other, &env.mem);
    if !env.objc.class_is_subclass_of(other_class, class) {
        return false;
    }
    env.objc.borrow::<NSIndexPathHostObject>(this).indexes
        == env.objc.borrow::<NSIndexPathHostObject>(other).indexes
}

- (NSUInteger)hash {
    env.objc.borrow::<NSIndexPathHostObject>(this).indexes.iter().fold(
        0u32,
        |hash, &index| hash.wrapping_mul(31).wrapping_add(index),
    )
}

- (id)copyWithZone:(NSZonePtr)_zone {
    retain(env, this)
}

- (())dealloc {
    env.objc.dealloc_object(this, &mut env.mem)
}
@end

};
