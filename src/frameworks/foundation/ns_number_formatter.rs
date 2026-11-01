/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Bounded historical `NSNumberFormatter` support.

use super::{ns_string, NSUInteger};
use crate::objc::{
    autorelease, id, msg, msg_class, nil, objc_classes, ClassExports, HostObject, NSZonePtr,
};
use crate::Environment;

const NSNumberFormatterNoStyle: NSUInteger = 0;
const NSNumberFormatterDecimalStyle: NSUInteger = 1;
const NSNumberFormatterCurrencyStyle: NSUInteger = 2;
const NSNumberFormatterPercentStyle: NSUInteger = 3;
const NSNumberFormatterScientificStyle: NSUInteger = 4;

struct NSNumberFormatterHostObject {
    style: NSUInteger,
    minimum_fraction_digits: NSUInteger,
    maximum_fraction_digits: NSUInteger,
    minimum_integer_digits: NSUInteger,
    uses_grouping_separator: bool,
    grouping_size: NSUInteger,
    lenient: bool,
}
impl HostObject for NSNumberFormatterHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSNumberFormatter: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(NSNumberFormatterHostObject {
        style: NSNumberFormatterNoStyle,
        minimum_fraction_digits: 0,
        maximum_fraction_digits: 3,
        minimum_integer_digits: 1,
        uses_grouping_separator: false,
        grouping_size: 3,
        lenient: false,
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (())dealloc {
    env.objc.dealloc_object(this, &mut env.mem)
}

- (NSUInteger)numberStyle {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).style
}
- (())setNumberStyle:(NSUInteger)style {
    env.objc.borrow_mut::<NSNumberFormatterHostObject>(this).style = style;
}

- (NSUInteger)minimumFractionDigits {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).minimum_fraction_digits
}
- (())setMinimumFractionDigits:(NSUInteger)digits {
    env.objc.borrow_mut::<NSNumberFormatterHostObject>(this).minimum_fraction_digits = digits;
}
- (NSUInteger)maximumFractionDigits {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).maximum_fraction_digits
}
- (())setMaximumFractionDigits:(NSUInteger)digits {
    env.objc.borrow_mut::<NSNumberFormatterHostObject>(this).maximum_fraction_digits = digits;
}
- (NSUInteger)minimumIntegerDigits {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).minimum_integer_digits
}
- (())setMinimumIntegerDigits:(NSUInteger)digits {
    env.objc.borrow_mut::<NSNumberFormatterHostObject>(this).minimum_integer_digits = digits;
}
- (bool)usesGroupingSeparator {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).uses_grouping_separator
}
- (())setUsesGroupingSeparator:(bool)uses {
    env.objc.borrow_mut::<NSNumberFormatterHostObject>(this).uses_grouping_separator = uses;
}
- (NSUInteger)groupingSize {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).grouping_size
}
- (())setGroupingSize:(NSUInteger)size {
    env.objc.borrow_mut::<NSNumberFormatterHostObject>(this).grouping_size = size;
}
- (bool)isLenient {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).lenient
}
- (())setLenient:(bool)lenient {
    env.objc.borrow_mut::<NSNumberFormatterHostObject>(this).lenient = lenient;
}

- (id)stringFromNumber:(id)number {
    if number == nil {
        return nil;
    }
    let state = env.objc.borrow::<NSNumberFormatterHostObject>(this);
    let style = state.style;
    let min_fraction = state.minimum_fraction_digits as usize;
    let max_fraction = state.maximum_fraction_digits.max(state.minimum_fraction_digits) as usize;
    let min_integer = state.minimum_integer_digits as usize;
    let grouping_size = state.grouping_size as usize;
    let grouping = state.uses_grouping_separator;

    if style == NSNumberFormatterNoStyle {
        return msg![env; number stringValue];
    }

    let mut value: f64 = msg![env; number doubleValue];
    if style == NSNumberFormatterPercentStyle {
        value *= 100.0;
    }
    let mut output = if style == NSNumberFormatterScientificStyle {
        format!("{value:.max_fraction$E}")
    } else {
        let mut fixed = format!("{value:.max_fraction$}");
        trim_fraction(&mut fixed, min_fraction);
        pad_integer(&mut fixed, min_integer);
        if grouping {
            fixed = add_grouping(&fixed, grouping_size.max(1));
        }
        fixed
    };
    if style == NSNumberFormatterCurrencyStyle {
        output.insert(0, '$');
    } else if style == NSNumberFormatterPercentStyle {
        output.push('%');
    }
    let string = ns_string::from_rust_string(env, output);
    autorelease(env, string)
}

- (id)stringForObjectValue:(id)object {
    msg![env; this stringFromNumber:object]
}

- (id)numberFromString:(id)string {
    if string == nil {
        return nil;
    }
    let state = env.objc.borrow::<NSNumberFormatterHostObject>(this);
    let style = state.style;
    let lenient = state.lenient;
    let mut value = ns_string::to_rust_string(env, string).trim().to_owned();
    value.retain(|character| character != ',' && character != '$' && character != '%');
    let parsed = if lenient {
        value.split_whitespace().next().unwrap_or("").parse::<f64>()
    } else {
        value.parse::<f64>()
    };
    let Ok(mut parsed) = parsed else {
        return nil;
    };
    if style == NSNumberFormatterPercentStyle {
        parsed /= 100.0;
    }
    msg_class![env; NSNumber numberWithDouble:parsed]
}

@end

};

fn trim_fraction(value: &mut String, minimum_digits: usize) {
    let Some(decimal) = value.find('.') else {
        return;
    };
    while value.ends_with('0') && value.len() - decimal - 1 > minimum_digits {
        value.pop();
    }
    if value.ends_with('.') && minimum_digits == 0 {
        value.pop();
    }
}

fn pad_integer(value: &mut String, minimum_digits: usize) {
    let negative = value.starts_with('-');
    let start = usize::from(negative);
    let integer_end = value.find('.').unwrap_or(value.len());
    let integer_digits = integer_end.saturating_sub(start);
    if integer_digits < minimum_digits {
        value.insert_str(start, &"0".repeat(minimum_digits - integer_digits));
    }
}

fn add_grouping(value: &str, grouping_size: usize) -> String {
    let negative = value.starts_with('-');
    let start = usize::from(negative);
    let integer_end = value.find('.').unwrap_or(value.len());
    let integer = &value[start..integer_end];
    let mut grouped = String::new();
    if negative {
        grouped.push('-');
    }
    for (idx, character) in integer.chars().enumerate() {
        if idx != 0 && (integer.len() - idx) % grouping_size == 0 {
            grouped.push(',');
        }
        grouped.push(character);
    }
    grouped.push_str(&value[integer_end..]);
    grouped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objc::release;

    #[test]
    fn decimal_formatter_round_trips_bounded_number_state() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let formatter: id = msg_class![env; NSNumberFormatter new];
        () = msg![env; formatter setNumberStyle:NSNumberFormatterDecimalStyle];
        () = msg![env; formatter setMaximumFractionDigits:2];
        () = msg![env; formatter setUsesGroupingSeparator:true];
        let number: id = msg_class![env; NSNumber numberWithDouble:1234.5f64];
        let string: id = msg![env; formatter stringFromNumber:number];
        assert_eq!(ns_string::to_rust_string(env, string), "1,234.5");
        let parsed: id = msg![env; formatter numberFromString:string];
        let value: f64 = msg![env; parsed doubleValue];
        assert_eq!(value, 1234.5);
        release(env, formatter);
    }
}
