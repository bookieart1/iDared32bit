/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSDateFormatter`.
//!
//! Resources:
//! - Apple's [Introduction to Data Formatting Programming Guide For Cocoa](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/DataFormatting/DataFormatting.html)
//! - [Unicode Technical Standard #35](https://unicode.org/reports/tr35/tr35-10.html#Date_Format_Patterns)

use crate::frameworks::core_foundation::time::{
    absolute_time_from_gregorian, CFAbsoluteTimeGetGregorianDate, CFGregorianDate,
};
use crate::frameworks::foundation::{ns_string, NSInteger, NSTimeInterval, NSUInteger};
use crate::objc::{
    autorelease, id, msg, msg_class, nil, objc_classes, release, retain, ClassExports,
    HostObject, NSZonePtr,
};
use crate::Environment;

type NSDateFormatterStyle = NSUInteger;

struct NSDateFormatterHostObject {
    date_format: Option<id>,
    date_style: NSDateFormatterStyle,
    time_style: NSDateFormatterStyle,
    formatter_behavior: NSUInteger,
    locale: id,
    time_zone: id,
}
impl HostObject for NSDateFormatterHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSDateFormatter: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(NSDateFormatterHostObject {
        date_format: None,
        date_style: 0,
        time_style: 0,
        formatter_behavior: 0,
        locale: nil,
        time_zone: nil,
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (())setDateFormat:(id)format { // NSString *
    let date_format: id = msg![env; format copy];
    let old = env
        .objc
        .borrow_mut::<NSDateFormatterHostObject>(this)
        .date_format
        .replace(date_format);
    if let Some(old) = old {
        release(env, old);
    }
}

- (())setTimeZone:(id)time_zone {
    retain(env, time_zone);
    let old = env.objc.borrow::<NSDateFormatterHostObject>(this).time_zone;
    env.objc.borrow_mut::<NSDateFormatterHostObject>(this).time_zone = time_zone;
    release(env, old);
}

- (id)timeZone {
    env.objc.borrow::<NSDateFormatterHostObject>(this).time_zone
}

- (())setLocale:(id)locale {
    retain(env, locale);
    let old = env.objc.borrow::<NSDateFormatterHostObject>(this).locale;
    env.objc.borrow_mut::<NSDateFormatterHostObject>(this).locale = locale;
    release(env, old);
}

- (id)locale {
    env.objc.borrow::<NSDateFormatterHostObject>(this).locale
}

// Setting a style replaces the format, as in Cocoa.
- (())setDateStyle:(NSDateFormatterStyle)style {
    env.objc.borrow_mut::<NSDateFormatterHostObject>(this).date_style = style;
    set_format_from_styles(env, this);
}
- (())setTimeStyle:(NSDateFormatterStyle)style {
    env.objc.borrow_mut::<NSDateFormatterHostObject>(this).time_style = style;
    set_format_from_styles(env, this);
}

- (())setFormatterBehavior:(NSUInteger)behavior {
    env.objc.borrow_mut::<NSDateFormatterHostObject>(this).formatter_behavior = behavior;
}

- (NSUInteger)formatterBehavior {
    env.objc.borrow::<NSDateFormatterHostObject>(this).formatter_behavior
}

- (())dealloc {
    let host_object = env.objc.borrow::<NSDateFormatterHostObject>(this);
    let (date_format, locale, time_zone) = (
        host_object.date_format,
        host_object.locale,
        host_object.time_zone,
    );
    if let Some(date_format) = date_format {
        release(env, date_format);
    }
    release(env, locale);
    release(env, time_zone);
    env.objc.dealloc_object(this, &mut env.mem)
}

- (id)stringFromDate:(id)date {
    let &NSDateFormatterHostObject {
        date_format, time_zone, ..
    } = env.objc.borrow(this);
    let format = ns_string::to_rust_string(env, date_format.unwrap()).to_string();
    log_dbg!("date_format before: {:?}", format);

    let ti: NSTimeInterval = msg![env; date timeIntervalSinceReferenceDate];
    let time_zone = if time_zone == nil {
        msg_class![env; NSTimeZone defaultTimeZone]
    } else {
        time_zone
    };
    // NSTimeZone is currently a fixed-offset GMT model, but honoring its
    // offset here keeps formatter behavior correct as that model grows.
    let seconds_from_gmt: NSInteger = msg![env; time_zone secondsFromGMT];
    let local_ti = ti + seconds_from_gmt as NSTimeInterval;
    let greg_date = CFAbsoluteTimeGetGregorianDate(env, local_ti, nil);
    // The reference date (2001-01-01) was a Monday.
    let weekday = ((local_ti / 86400.0).floor() as i64 + 1).rem_euclid(7) as usize;
    let res = format_date(&format, &greg_date, weekday, seconds_from_gmt);
    log_dbg!("date_format after: {:?}", res);

    let res = ns_string::from_rust_string(env, res);
    autorelease(env, res)
}

- (id)dateFromString:(id)string {
    let &NSDateFormatterHostObject {
        date_format, time_zone, ..
    } = env.objc.borrow(this);
    let Some(date_format) = date_format else {
        return nil;
    };
    let format = ns_string::to_rust_string(env, date_format);
    let string = ns_string::to_rust_string(env, string);
    let Some(parsed) = parse_date(&format, &string) else {
        return nil;
    };

    let seconds_from_gmt = if let Some(seconds) = parsed.seconds_from_gmt {
        seconds
    } else {
        let time_zone = if time_zone == nil {
            msg_class![env; NSTimeZone defaultTimeZone]
        } else {
            time_zone
        };
        let seconds: NSInteger = msg![env; time_zone secondsFromGMT];
        i64::from(seconds)
    };
    let time_interval = absolute_time_from_gregorian(
        parsed.year,
        parsed.month,
        parsed.day,
        parsed.hour,
        parsed.minute,
        parsed.second,
    ) - seconds_from_gmt as NSTimeInterval;
    msg_class![env; NSDate dateWithTimeIntervalSinceReferenceDate:time_interval]
}

@end

};

/// The en_US patterns for each style (none, short, medium, long, full).
fn set_format_from_styles(env: &mut Environment, this: id) {
    const DATE: [&str; 5] = [
        "",
        "M/d/yy",
        "MMM d, yyyy",
        "MMMM d, yyyy",
        "EEEE, MMMM d, yyyy",
    ];
    const TIME: [&str; 5] = ["", "h:mm a", "h:mm:ss a", "h:mm:ss a z", "h:mm:ss a zzzz"];
    let host_object = env.objc.borrow::<NSDateFormatterHostObject>(this);
    let date = DATE[host_object.date_style as usize];
    let time = TIME[host_object.time_style as usize];
    let format = [date, time]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let format = ns_string::from_rust_string(env, format);
    let old = env
        .objc
        .borrow_mut::<NSDateFormatterHostObject>(this)
        .date_format
        .replace(format);
    if let Some(old) = old {
        release(env, old);
    }
}

/// Formats a date with a Unicode date pattern (en_US).
fn format_date(
    format: &str,
    date: &CFGregorianDate,
    weekday: usize,
    seconds_from_gmt: NSInteger,
) -> String {
    const MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    const DAYS: [&str; 7] = [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ];
    let month = MONTHS[date.month as usize - 1];
    let hour12 = (date.hours as i32 + 11) % 12 + 1;
    let mut out = String::new();
    let mut chars = format.chars().peekable();
    while let Some(c) = chars.next() {
        if !c.is_ascii_alphabetic() {
            if c == '\'' {
                // Quoted text, where '' is a quote.
                while let Some(c) = chars.next() {
                    if c == '\'' && chars.next_if_eq(&'\'').is_none() {
                        break;
                    }
                    out.push(c);
                }
            } else {
                out.push(c);
            }
            continue;
        }
        let mut n = 1;
        while chars.next_if_eq(&c).is_some() {
            n += 1;
        }
        let num = |v: i32| format!("{v:0n$}");
        out += &match c {
            'y' | 'Y' if n == 2 => format!("{:02}", date.year % 100),
            'y' | 'Y' => num(date.year),
            'M' | 'L' if n == 5 => month[..1].to_string(),
            'M' | 'L' if n == 4 => month.to_string(),
            'M' | 'L' if n == 3 => month[..3].to_string(),
            'M' | 'L' if n <= 2 => num(date.month.into()),
            'd' => num(date.day.into()),
            'H' => num(date.hours.into()),
            'h' => num(hour12),
            'm' => num(date.minutes.into()),
            's' => num(date.seconds as i32),
            'a' => (if date.hours < 12 { "AM" } else { "PM" }).to_string(),
            'E' if n == 6 => DAYS[weekday][..2].to_string(),
            'E' if n == 5 => DAYS[weekday][..1].to_string(),
            'E' if n == 4 => DAYS[weekday].to_string(),
            'E' if n <= 3 => DAYS[weekday][..3].to_string(),
            // Stand-alone local day of week. The formatter currently models
            // en_US, whose local numeric week starts on Sunday.
            'c' if n == 6 => DAYS[weekday][..2].to_string(),
            'c' if n == 5 => DAYS[weekday][..1].to_string(),
            'c' if n == 4 => DAYS[weekday].to_string(),
            'c' if n == 3 => DAYS[weekday][..3].to_string(),
            'c' => num(weekday as i32 + 1),
            // Dates are worked out in UTC (see CFAbsoluteTimeGetGregorianDate).
            'z' if n >= 4 => "Greenwich Mean Time".to_string(),
            'z' => "GMT".to_string(),
            // RFC 822/ISO 8601 zone forms from Unicode TR35. Widths 1-3 use
            // the basic numeric form, width 4 uses localized GMT, and width 5
            // uses the extended ISO form (with Z for UTC).
            'Z' if n <= 3 => format_time_zone_offset(seconds_from_gmt, false),
            'Z' if n == 4 => {
                if seconds_from_gmt == 0 {
                    "GMT".to_string()
                } else {
                    format!("GMT{}", format_time_zone_offset(seconds_from_gmt, true))
                }
            }
            'Z' if n == 5 => {
                if seconds_from_gmt == 0 {
                    "Z".to_string()
                } else {
                    format_time_zone_offset(seconds_from_gmt, true)
                }
            }
            _ => unimplemented!("date format pattern {}", c.to_string().repeat(n)),
        };
    }
    out
}

fn format_time_zone_offset(seconds_from_gmt: NSInteger, colon: bool) -> String {
    let seconds = i64::from(seconds_from_gmt);
    let sign = if seconds < 0 { '-' } else { '+' };
    let absolute = seconds.abs();
    let hours = absolute / 3600;
    let minutes = (absolute % 3600) / 60;
    if colon {
        format!("{sign}{hours:02}:{minutes:02}")
    } else {
        format!("{sign}{hours:02}{minutes:02}")
    }
}

#[derive(Debug, PartialEq)]
struct ParsedDate {
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: f64,
    seconds_from_gmt: Option<i64>,
}

/// Parse the same bounded en_US Unicode-pattern subset that `format_date`
/// emits. NSDateFormatter returns nil for an invalid string, so unsupported or
/// inconsistent input is rejected rather than partially inventing a date.
fn parse_date(format: &str, input: &str) -> Option<ParsedDate> {
    let mut input_pos = 0;
    let mut year = 2001;
    let mut month = 1;
    let mut day = 1;
    let mut hour = 0;
    let mut minute = 0;
    let mut second = 0.0;
    let mut hour_is_12 = false;
    let mut is_pm = None;
    let mut seconds_from_gmt = None;
    let mut chars = format.chars().peekable();

    while let Some(c) = chars.next() {
        if !c.is_ascii_alphabetic() {
            if c == '\'' {
                let mut literal = String::new();
                while let Some(c) = chars.next() {
                    if c == '\'' {
                        if chars.next_if_eq(&'\'').is_some() {
                            literal.push('\'');
                            continue;
                        }
                        break;
                    }
                    literal.push(c);
                }
                take_literal(input, &mut input_pos, &literal)?;
            } else {
                take_literal(input, &mut input_pos, &c.to_string())?;
            }
            continue;
        }

        let mut width = 1;
        while chars.next_if_eq(&c).is_some() {
            width += 1;
        }
        match c {
            'y' | 'Y' => {
                let value = if width == 2 {
                    take_number(input, &mut input_pos, 2, 2)?
                } else {
                    take_number(input, &mut input_pos, width.min(4), width.max(4))?
                };
                year = if width == 2 {
                    if value >= 50 { 1900 + value } else { 2000 + value }
                } else {
                    value
                };
            }
            'M' | 'L' if width <= 2 => {
                month = take_number(input, &mut input_pos, width, 2)?;
            }
            'M' | 'L' if width == 3 => {
                month = take_named(input, &mut input_pos, &MONTH_NAMES, true)? as i64 + 1;
            }
            'M' | 'L' if width == 4 => {
                month = take_named(input, &mut input_pos, &MONTH_NAMES, false)? as i64 + 1;
            }
            'd' => day = take_number(input, &mut input_pos, width.min(2), 2)?,
            'H' => hour = take_number(input, &mut input_pos, width.min(2), 2)?,
            'h' => {
                hour = take_number(input, &mut input_pos, width.min(2), 2)?;
                hour_is_12 = true;
            }
            'm' => minute = take_number(input, &mut input_pos, width.min(2), 2)?,
            's' => second = take_number(input, &mut input_pos, width.min(2), 2)? as f64,
            'S' => {
                if width > 9 {
                    return None;
                }
                let fraction = take_number(input, &mut input_pos, width, width)?;
                second += fraction as f64 / 10_i64.pow(width as u32) as f64;
            }
            'a' => {
                is_pm = if take_literal_case_insensitive(input, &mut input_pos, "AM").is_some() {
                    Some(false)
                } else {
                    take_literal_case_insensitive(input, &mut input_pos, "PM")?;
                    Some(true)
                };
            }
            'E' if width <= 3 => {
                take_named(input, &mut input_pos, &DAY_NAMES, true)?;
            }
            'E' => {
                take_named(input, &mut input_pos, &DAY_NAMES, false)?;
            }
            'c' if width <= 2 => {
                take_number(input, &mut input_pos, width, 2)?;
            }
            'c' if width == 3 => {
                take_named(input, &mut input_pos, &DAY_NAMES, true)?;
            }
            'c' => {
                take_named(input, &mut input_pos, &DAY_NAMES, false)?;
            }
            'z' | 'Z' => {
                seconds_from_gmt = Some(take_time_zone(input, &mut input_pos)?);
            }
            _ => return None,
        }
    }

    if input_pos != input.len() || !(1..=12).contains(&month) {
        return None;
    }
    let max_day = days_in_month(year, month);
    if !(1..=max_day).contains(&day)
        || !(0..=59).contains(&minute)
        || !(0.0..60.0).contains(&second)
    {
        return None;
    }
    if hour_is_12 {
        if !(1..=12).contains(&hour) {
            return None;
        }
        hour = hour % 12 + if is_pm.unwrap_or(false) { 12 } else { 0 };
    } else if !(0..=23).contains(&hour) {
        return None;
    }

    Some(ParsedDate {
        year,
        month,
        day,
        hour,
        minute,
        second,
        seconds_from_gmt,
    })
}

const MONTH_NAMES: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August",
    "September", "October", "November", "December",
];
const DAY_NAMES: [&str; 7] = [
    "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday",
];

fn take_literal(input: &str, pos: &mut usize, literal: &str) -> Option<()> {
    input.get(*pos..)?.starts_with(literal).then(|| *pos += literal.len())
}

fn take_literal_case_insensitive(input: &str, pos: &mut usize, literal: &str) -> Option<()> {
    let candidate = input.get(*pos..(*pos + literal.len()))?;
    candidate.eq_ignore_ascii_case(literal).then(|| *pos += literal.len())
}

fn take_number(
    input: &str,
    pos: &mut usize,
    min_digits: usize,
    max_digits: usize,
) -> Option<i64> {
    let bytes = input.as_bytes();
    let start = *pos;
    while *pos < bytes.len()
        && *pos - start < max_digits
        && bytes[*pos].is_ascii_digit()
    {
        *pos += 1;
    }
    if *pos - start < min_digits {
        *pos = start;
        return None;
    }
    input.get(start..*pos)?.parse().ok()
}

fn take_named(input: &str, pos: &mut usize, names: &[&str], abbreviated: bool) -> Option<usize> {
    names.iter().enumerate().find_map(|(index, name)| {
        let name = if abbreviated { name.get(..3)? } else { name };
        let mut candidate_pos = *pos;
        take_literal_case_insensitive(input, &mut candidate_pos, name)?;
        *pos = candidate_pos;
        Some(index)
    })
}

fn take_time_zone(input: &str, pos: &mut usize) -> Option<i64> {
    for prefix in ["Greenwich Mean Time", "GMT", "UTC"] {
        let mut candidate_pos = *pos;
        if take_literal_case_insensitive(input, &mut candidate_pos, prefix).is_some() {
            *pos = candidate_pos;
            if matches!(input.as_bytes().get(*pos), Some(&b'+') | Some(&b'-')) {
                return take_numeric_time_zone(input, pos);
            }
            return Some(0);
        }
    }
    if input.as_bytes().get(*pos) == Some(&b'Z') {
        *pos += 1;
        Some(0)
    } else {
        take_numeric_time_zone(input, pos)
    }
}

fn take_numeric_time_zone(input: &str, pos: &mut usize) -> Option<i64> {
    let sign = match *input.as_bytes().get(*pos)? {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    *pos += 1;
    let hours = take_number(input, pos, 2, 2)?;
    if input.as_bytes().get(*pos) == Some(&b':') {
        *pos += 1;
    }
    let minutes = take_number(input, pos, 2, 2)?;
    (hours <= 23 && minutes <= 59).then_some(sign * (hours * 3600 + minutes * 60))
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        4 | 6 | 9 | 11 => 30,
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        _ => 31,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_date() {
        // 2009-07-04 15:05:09, a Saturday.
        let date = CFGregorianDate {
            year: 2009,
            month: 7,
            day: 4,
            hours: 15,
            minutes: 5,
            seconds: 9.0,
        };
        let f = |format| format_date(format, &date, 6, 0);
        assert_eq!(f("yyyy-MM-dd HH:mm:ss"), "2009-07-04 15:05:09");
        assert_eq!(f("M/d/yy h:mm a"), "7/4/09 3:05 PM");
        assert_eq!(f("MMM d, yyyy h:mm:ss a"), "Jul 4, 2009 3:05:09 PM");
        assert_eq!(f("EEEE, MMMM d, yyyy"), "Saturday, July 4, 2009");
        assert_eq!(f("L LL LLL LLLL LLLLL"), "7 07 Jul July J");
        assert_eq!(f("c cc ccc cccc ccccc cccccc"), "7 07 Sat Saturday S Sa");
        assert_eq!(f("'at' HH 'o''clock'"), "at 15 o'clock");
        assert_eq!(f("Z ZZ ZZZ ZZZZ ZZZZZ"), "+0000 +0000 +0000 GMT Z");
        assert_eq!(
            format_date("Z ZZZZ ZZZZZ", &date, 6, -19_800),
            "-0530 GMT-05:30 -05:30"
        );
    }

    #[test]
    fn test_parse_date() {
        assert_eq!(
            parse_date("yyyy-MM-dd HH:mm:ss", "2009-07-04 15:05:09"),
            Some(ParsedDate {
                year: 2009,
                month: 7,
                day: 4,
                hour: 15,
                minute: 5,
                second: 9.0,
                seconds_from_gmt: None,
            })
        );
        assert_eq!(
            parse_date("MMM d, yyyy h:mm:ss a Z", "Jul 4, 2009 3:05:09 PM -0530"),
            Some(ParsedDate {
                year: 2009,
                month: 7,
                day: 4,
                hour: 15,
                minute: 5,
                second: 9.0,
                seconds_from_gmt: Some(-19_800),
            })
        );
        assert_eq!(
            parse_date("yyyy-MM-dd'T'HH:mm:ss.SSSZZZZZ", "2009-07-04T15:05:09.125Z")
                .unwrap()
                .second,
            9.125
        );
        assert!(parse_date("yyyy-MM-dd", "2009-02-29").is_none());
        assert!(parse_date("yyyy-MM-dd", "not a date").is_none());
    }
}
