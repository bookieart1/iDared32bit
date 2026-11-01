/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSIndexSet` and `NSMutableIndexSet`.

use super::{NSNotFound, NSRange, NSUInteger};
use crate::objc::{
    autorelease, id, msg, msg_class, nil, objc_classes, retain, ClassExports, HostObject, NSZonePtr,
};

/// Sorted, non-overlapping half-open ranges. This reflects the real class's
/// compact representation and avoids expanding a large range into individual
/// indexes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct IndexSetHostObject {
    ranges: Vec<(NSUInteger, NSUInteger)>,
}
impl HostObject for IndexSetHostObject {}

impl IndexSetHostObject {
    fn bounds(range: NSRange) -> (NSUInteger, NSUInteger) {
        let start = range.location;
        let end = start
            .checked_add(range.length)
            .expect("NSIndexSet range overflow");
        assert!(
            end <= NSNotFound as NSUInteger,
            "NSIndexSet index out of range"
        );
        (start, end)
    }

    fn add_range(&mut self, start: NSUInteger, end: NSUInteger) {
        if start == end {
            return;
        }
        let mut merged = (start, end);
        let mut out = Vec::with_capacity(self.ranges.len() + 1);
        let mut inserted = false;
        for &(old_start, old_end) in &self.ranges {
            if old_end < merged.0 {
                out.push((old_start, old_end));
            } else if merged.1 < old_start {
                if !inserted {
                    out.push(merged);
                    inserted = true;
                }
                out.push((old_start, old_end));
            } else {
                merged.0 = merged.0.min(old_start);
                merged.1 = merged.1.max(old_end);
            }
        }
        if !inserted {
            out.push(merged);
        }
        self.ranges = out;
    }

    fn remove_range(&mut self, start: NSUInteger, end: NSUInteger) {
        if start == end {
            return;
        }
        let mut out = Vec::with_capacity(self.ranges.len());
        for &(old_start, old_end) in &self.ranges {
            if old_end <= start || old_start >= end {
                out.push((old_start, old_end));
                continue;
            }
            if old_start < start {
                out.push((old_start, start));
            }
            if old_end > end {
                out.push((end, old_end));
            }
        }
        self.ranges = out;
    }

    fn contains(&self, index: NSUInteger) -> bool {
        self.ranges
            .iter()
            .any(|&(start, end)| start <= index && index < end)
    }

    fn count(&self) -> NSUInteger {
        self.ranges.iter().map(|&(start, end)| end - start).sum()
    }
}

fn new_with_ranges(env: &mut crate::Environment, class_name: &str, ranges: Vec<(u32, u32)>) -> id {
    let class = env.objc.get_known_class(class_name, &mut env.mem);
    env.objc
        .alloc_object(class, Box::new(IndexSetHostObject { ranges }), &mut env.mem)
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSIndexSet: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(this, Box::new(IndexSetHostObject::default()), &mut env.mem)
}

+ (id)indexSet {
    let set: id = msg![env; this new];
    autorelease(env, set)
}

+ (id)indexSetWithIndex:(NSUInteger)index {
    let set: id = msg![env; this alloc];
    let set: id = msg![env; set initWithIndex:index];
    autorelease(env, set)
}

+ (id)indexSetWithIndexesInRange:(NSRange)range {
    let set: id = msg![env; this alloc];
    let set: id = msg![env; set initWithIndexesInRange:range];
    autorelease(env, set)
}

- (id)initWithIndex:(NSUInteger)index {
    assert!(index < NSNotFound as NSUInteger, "NSIndexSet index out of range");
    env.objc.borrow_mut::<IndexSetHostObject>(this).ranges = vec![(index, index + 1)];
    this
}

- (id)initWithIndexesInRange:(NSRange)range {
    let (start, end) = IndexSetHostObject::bounds(range);
    env.objc.borrow_mut::<IndexSetHostObject>(this).ranges =
        if start == end { Vec::new() } else { vec![(start, end)] };
    this
}

- (id)initWithIndexSet:(id)other {
    let ranges = env.objc.borrow::<IndexSetHostObject>(other).ranges.clone();
    env.objc.borrow_mut::<IndexSetHostObject>(this).ranges = ranges;
    this
}

- (NSUInteger)count {
    env.objc.borrow::<IndexSetHostObject>(this).count()
}

- (bool)containsIndex:(NSUInteger)index {
    env.objc.borrow::<IndexSetHostObject>(this).contains(index)
}

- (bool)containsIndexesInRange:(NSRange)range {
    let (start, end) = IndexSetHostObject::bounds(range);
    if start == end {
        return true;
    }
    env.objc
        .borrow::<IndexSetHostObject>(this)
        .ranges
        .iter()
        .any(|&(old_start, old_end)| old_start <= start && old_end >= end)
}

- (bool)intersectsIndexesInRange:(NSRange)range {
    let (start, end) = IndexSetHostObject::bounds(range);
    env.objc
        .borrow::<IndexSetHostObject>(this)
        .ranges
        .iter()
        .any(|&(old_start, old_end)| old_start < end && old_end > start)
}

- (NSUInteger)firstIndex {
    env.objc
        .borrow::<IndexSetHostObject>(this)
        .ranges
        .first()
        .map_or(NSNotFound as NSUInteger, |range| range.0)
}

- (NSUInteger)lastIndex {
    env.objc
        .borrow::<IndexSetHostObject>(this)
        .ranges
        .last()
        .map_or(NSNotFound as NSUInteger, |range| range.1 - 1)
}

- (NSUInteger)indexGreaterThanIndex:(NSUInteger)index {
    if index >= (NSNotFound as NSUInteger - 1) {
        return NSNotFound as NSUInteger;
    }
    msg![env; this indexGreaterThanOrEqualToIndex:(index + 1)]
}

- (NSUInteger)indexGreaterThanOrEqualToIndex:(NSUInteger)index {
    for &(start, end) in &env.objc.borrow::<IndexSetHostObject>(this).ranges {
        if index < start {
            return start;
        }
        if index < end {
            return index;
        }
    }
    NSNotFound as NSUInteger
}

- (NSUInteger)indexLessThanIndex:(NSUInteger)index {
    if index == 0 {
        return NSNotFound as NSUInteger;
    }
    msg![env; this indexLessThanOrEqualToIndex:(index - 1)]
}

- (NSUInteger)indexLessThanOrEqualToIndex:(NSUInteger)index {
    for &(start, end) in env.objc.borrow::<IndexSetHostObject>(this).ranges.iter().rev() {
        if index >= end {
            return end - 1;
        }
        if index >= start {
            return index;
        }
    }
    NSNotFound as NSUInteger
}

- (bool)isEqualToIndexSet:(id)other {
    other != nil
        && env.objc.borrow::<IndexSetHostObject>(this).ranges
            == env.objc.borrow::<IndexSetHostObject>(other).ranges
}

- (bool)isEqual:(id)other {
    if this == other {
        return true;
    }
    if other == nil {
        return false;
    }
    let class = env.objc.get_known_class("NSIndexSet", &mut env.mem);
    let other_class = crate::objc::ObjC::read_isa(other, &env.mem);
    env.objc.class_is_subclass_of(other_class, class)
        && msg![env; this isEqualToIndexSet:other]
}

- (NSUInteger)hash {
    env.objc
        .borrow::<IndexSetHostObject>(this)
        .ranges
        .iter()
        .fold(0u32, |hash, &(start, end)| {
            hash.wrapping_mul(31).wrapping_add(start).wrapping_mul(31).wrapping_add(end)
        })
}

- (id)copyWithZone:(NSZonePtr)_zone {
    retain(env, this)
}

- (id)mutableCopyWithZone:(NSZonePtr)_zone {
    let ranges = env.objc.borrow::<IndexSetHostObject>(this).ranges.clone();
    new_with_ranges(env, "NSMutableIndexSet", ranges)
}

- (())dealloc {
    env.objc.dealloc_object(this, &mut env.mem)
}

@end

@implementation NSMutableIndexSet: NSIndexSet

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(this, Box::new(IndexSetHostObject::default()), &mut env.mem)
}

- (id)copyWithZone:(NSZonePtr)_zone {
    let ranges = env.objc.borrow::<IndexSetHostObject>(this).ranges.clone();
    new_with_ranges(env, "NSIndexSet", ranges)
}

- (())addIndex:(NSUInteger)index {
    assert!(index < NSNotFound as NSUInteger, "NSMutableIndexSet index out of range");
    env.objc.borrow_mut::<IndexSetHostObject>(this).add_range(index, index + 1);
}

- (())addIndexesInRange:(NSRange)range {
    let (start, end) = IndexSetHostObject::bounds(range);
    env.objc.borrow_mut::<IndexSetHostObject>(this).add_range(start, end);
}

- (())addIndexes:(id)other {
    let ranges = env.objc.borrow::<IndexSetHostObject>(other).ranges.clone();
    let set = env.objc.borrow_mut::<IndexSetHostObject>(this);
    for (start, end) in ranges {
        set.add_range(start, end);
    }
}

- (())removeIndex:(NSUInteger)index {
    if index < NSNotFound as NSUInteger {
        env.objc.borrow_mut::<IndexSetHostObject>(this).remove_range(index, index + 1);
    }
}

- (())removeIndexesInRange:(NSRange)range {
    let (start, end) = IndexSetHostObject::bounds(range);
    env.objc.borrow_mut::<IndexSetHostObject>(this).remove_range(start, end);
}

- (())removeIndexes:(id)other {
    let ranges = env.objc.borrow::<IndexSetHostObject>(other).ranges.clone();
    let set = env.objc.borrow_mut::<IndexSetHostObject>(this);
    for (start, end) in ranges {
        set.remove_range(start, end);
    }
}

- (())removeAllIndexes {
    env.objc.borrow_mut::<IndexSetHostObject>(this).ranges.clear();
}

@end

};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objc::release;
    use crate::Environment;

    #[test]
    fn mutable_ranges_merge_split_and_copy_independently() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let set: id = msg_class![env; NSMutableIndexSet indexSet];
        retain(env, set);
        let first_range = NSRange {
            location: 10,
            length: 5,
        };
        () = msg![env; set addIndexesInRange:first_range];
        () = msg![env; set addIndex:15u32];
        let second_range = NSRange {
            location: 2,
            length: 3,
        };
        () = msg![env; set addIndexesInRange:second_range];
        let count: NSUInteger = msg![env; set count];
        assert_eq!(count, 9);
        assert!(msg![env; set containsIndex:12u32]);
        let removed_range = NSRange {
            location: 11,
            length: 3,
        };
        () = msg![env; set removeIndexesInRange:removed_range];
        let first: NSUInteger = msg![env; set firstIndex];
        let last: NSUInteger = msg![env; set lastIndex];
        assert_eq!((first, last), (2, 15));
        assert!(!msg![env; set containsIndex:12u32]);

        let immutable: id = msg![env; set copy];
        () = msg![env; set removeAllIndexes];
        let immutable_count: NSUInteger = msg![env; immutable count];
        assert_eq!(immutable_count, 6);
        release(env, immutable);
        release(env, set);
    }
}
