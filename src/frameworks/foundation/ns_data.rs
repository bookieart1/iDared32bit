/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSData` and `NSMutableData`.

use super::ns_error::{NSCocoaErrorDomain, NSFileReadNoSuchFileError};
use super::ns_string::{from_rust_string, get_static_str, to_rust_string};
use super::{NSRange, NSUInteger};
use crate::frameworks::foundation::ns_keyed_unarchiver::decode_current_data;
use crate::fs::GuestPath;
use crate::mem::{ConstPtr, ConstVoidPtr, MutPtr, MutVoidPtr, Ptr};
use crate::objc::{
    autorelease, id, msg, nil, objc_classes, release, retain, ClassExports, HostObject, NSZonePtr,
};
use crate::{msg_class, Environment};

pub(super) struct NSDataHostObject {
    pub(super) bytes: MutVoidPtr,
    pub(super) length: NSUInteger,
    free_when_done: bool,
}
impl HostObject for NSDataHostObject {}

const NS_DATA_READING_MAPPED_IF_SAFE: NSUInteger = 1 << 0;
const NS_DATA_READING_UNCACHED: NSUInteger = 1 << 1;
const NS_DATA_READING_MAPPED_ALWAYS: NSUInteger = 1 << 3;
const NS_DATA_READING_KNOWN_OPTIONS: NSUInteger = NS_DATA_READING_MAPPED_IF_SAFE
    | NS_DATA_READING_UNCACHED
    | NS_DATA_READING_MAPPED_ALWAYS;

fn populate_file_read_error(env: &mut Environment, out_error: MutPtr<id>) {
    if out_error.is_null() {
        return;
    }
    let domain = get_static_str(env, NSCocoaErrorDomain);
    let error: id = msg_class![env; NSError errorWithDomain:domain
                                                 code:NSFileReadNoSuchFileError
                                             userInfo:nil];
    env.mem.write(out_error, error);
}

fn init_with_contents_of_file(
    env: &mut Environment,
    this: id,
    path: id,
    options: NSUInteger,
    out_error: MutPtr<id>,
) -> id {
    let unknown_options = options & !NS_DATA_READING_KNOWN_OPTIONS;
    if unknown_options != 0 {
        log!(
            "Warning: NSData read uses unknown option bits {unknown_options:#x}; ignoring them"
        );
    }
    // The defined options are caching/mapping hints. The guest filesystem
    // currently provides owned bytes, so an eager read has the same visible
    // contents and lifetime while deliberately ignoring those hints.
    if path == nil {
        populate_file_read_error(env, out_error);
        release(env, this);
        return nil;
    }
    let path_string = to_rust_string(env, path);
    log_dbg!(
        "[(NSData*){:?} initWithContentsOfFile:{:?} options:{:#x} error:{:?}]",
        this,
        path_string,
        options,
        out_error
    );
    let Ok(bytes) = env.fs.read(GuestPath::new(&path_string)) else {
        populate_file_read_error(env, out_error);
        release(env, this);
        return nil;
    };
    let size = bytes.len().try_into().unwrap();
    let alloc = env.mem.alloc(size);
    let slice = env.mem.bytes_at_mut(alloc.cast(), size);
    slice.copy_from_slice(&bytes);

    let host_object = env.objc.borrow_mut::<NSDataHostObject>(this);
    host_object.bytes = alloc;
    host_object.length = size;
    this
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

// NSData doesn't seem to be an abstract class?
@implementation NSData: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(NSDataHostObject {
        bytes: Ptr::null(),
        length: 0,
        free_when_done: true,
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (id)data {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new init];
    autorelease(env, new)
}

+ (id)dataWithBytesNoCopy:(MutVoidPtr)bytes
                   length:(NSUInteger)length {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithBytesNoCopy:bytes length:length];
    autorelease(env, new)
}

+ (id)dataWithBytesNoCopy:(MutVoidPtr)bytes
                   length:(NSUInteger)length
             freeWhenDone:(bool)free_when_done {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithBytesNoCopy:bytes length:length freeWhenDone:free_when_done];
    autorelease(env, new)
}

+ (id)dataWithBytes:(ConstVoidPtr)bytes
             length:(NSUInteger)length {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithBytes:bytes length:length];
    autorelease(env, new)
}

+ (id)dataWithContentsOfFile:(id)path {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithContentsOfFile:path];
    autorelease(env, new)
}

+ (id)dataWithContentsOfMappedFile:(id)path {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithContentsOfMappedFile:path];
    autorelease(env, new)
}

+ (id)dataWithContentsOfURL:(id)url {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithContentsOfURL:url];
    autorelease(env, new)
}

+ (id)dataWithContentsOfURL:(id)url
                    options:(NSUInteger)options
                      error:(MutPtr<id>)out_error {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithContentsOfURL:url options:options error:out_error];
    autorelease(env, new)
}

+ (id)dataWithContentsOfFile:(id)path
                     options:(NSUInteger)options
                       error:(MutPtr<id>)out_error {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithContentsOfFile:path options:options error:out_error];
    autorelease(env, new)
}

+ (id)dataWithData:(id)data {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithData:data];
    autorelease(env, new)
}

// Calling the standard `init` is also allowed, in which case we just get data
// of size 0.

- (id)initWithBytesNoCopy:(MutVoidPtr)bytes
                   length:(NSUInteger)length {
    msg![env; this initWithBytesNoCopy:bytes length:length freeWhenDone:true]
}

- (id)initWithBytesNoCopy:(MutVoidPtr)bytes
                   length:(NSUInteger)length
             freeWhenDone:(bool)free_when_done {
    let host_object = env.objc.borrow_mut::<NSDataHostObject>(this);
    assert!(host_object.bytes.is_null() && host_object.length == 0);
    host_object.bytes = bytes;
    host_object.length = length;
    host_object.free_when_done = free_when_done;
    this
}

- (id)initWithBytes:(ConstVoidPtr)bytes
              length:(NSUInteger)length {
    let host_object = env.objc.borrow_mut::<NSDataHostObject>(this);
    assert!(host_object.bytes.is_null() && host_object.length == 0);
    let alloc = env.mem.alloc(length);
    env.mem.memmove(alloc, bytes, length);
    host_object.bytes = alloc;
    host_object.length = length;
    this
}

- (id)initWithData:(id)data {
    let bytes: ConstVoidPtr = msg![env; data bytes];
    let length: NSUInteger = msg![env; data length];
    msg![env; this initWithBytes:bytes length:length]
}

- (id)initWithContentsOfURL:(id)url { // NSURL *
    if msg![env; url isFileURL] {
        let ns_path: id = msg![env; url path];
        let path = to_rust_string(env, ns_path);
        assert!(path.starts_with("/")); // TODO
        msg![env; this initWithContentsOfFile:ns_path]
    } else {
        let absolute_str: id = msg![env; url absoluteString];
        let path = to_rust_string(env, absolute_str);
        if path.starts_with("http://") || path.starts_with("https://") {
            log!("TODO: ignoring [(NSData*){:?} initWithContentsOfURL:{:?}]", this, path);
        } else {
            // Foundation reports a failed load for unsupported URL schemes;
            // it does not abort the process. This is commonly used as a
            // capability probe by older apps.
            log!("Warning: [(NSData*){:?} initWithContentsOfURL:{:?}] uses an unsupported URL scheme", this, path);
        }
        release(env, this);
        nil
    }
}

- (id)initWithContentsOfURL:(id)url // NSURL *
                    options:(NSUInteger)options
                      error:(MutPtr<id>)out_error {
    if url != nil && msg![env; url isFileURL] {
        let path: id = msg![env; url path];
        msg![env; this initWithContentsOfFile:path options:options error:out_error]
    } else {
        // Preserve the existing URL-loader boundary for non-file schemes.
        // Network loading is a separate unimplemented contract.
        msg![env; this initWithContentsOfURL:url]
    }
}

- (id)initWithContentsOfFile:(id)path {
    init_with_contents_of_file(env, this, path, 0, MutPtr::null())
}

- (id)initWithContentsOfFile:(id)path
                     options:(NSUInteger)options
                       error:(MutPtr<id>)out_error {
    init_with_contents_of_file(env, this, path, options, out_error)
}

- (id)initWithContentsOfMappedFile:(id)path {
    log_dbg!("[NSData initWithContentsOfMappedFile:] not using memory mapping");
    msg![env; this initWithContentsOfFile:path]
}

// FIXME: writes should be atomic
- (bool)writeToFile:(id)path // NSString*
         atomically:(bool)_use_aux_file {
    let file = to_rust_string(env, path);
    log_dbg!("[(NSData*){:?} writeToFile:{:?} atomically:_]", this, file);
    let host_object = env.objc.borrow::<NSDataHostObject>(this);
    // Mem::bytes_at() panics when the pointer is NULL, but NSData's pointer can
    // be NULL if the length is 0.
    let slice = if host_object.length == 0 {
        &[]
    } else {
        env.mem.bytes_at(host_object.bytes.cast(), host_object.length)
    };
    env.fs.write(GuestPath::new(&file), slice).is_ok()
}

- (())dealloc {
    let &NSDataHostObject { bytes, free_when_done, .. } = env.objc.borrow(this);
    if !bytes.is_null() && free_when_done {
        env.mem.free(bytes);
    }
    env.objc.dealloc_object(this, &mut env.mem)
}

// NSCopying implementation
- (id)copyWithZone:(NSZonePtr)_zone {
    retain(env, this)
}

// NSCoding implementation
- (id)initWithCoder:(id)coder {
    release(env, this);
    // Note: Assuming NSKeyedUnarchiver as coder here
    decode_current_data(env, coder, /* is_mutable: */ true)
}

- (id)mutableCopyWithZone:(NSZonePtr)_zone {
    let bytes: ConstVoidPtr = msg![env; this bytes];
    let length: NSUInteger = msg![env; this length];
    let new = msg_class![env; NSMutableData alloc];
    msg![env; new initWithBytes:(bytes.cast_mut()) length:length]
}

- (ConstVoidPtr)bytes {
    env.objc.borrow::<NSDataHostObject>(this).bytes.cast_const()
}
- (NSUInteger)length {
    env.objc.borrow::<NSDataHostObject>(this).length
}

- (bool)isEqualToData:(id)other {
    // FIXME: Avoid allocation
    let a = to_rust_slice(env, this).to_owned();
    let b = to_rust_slice(env, other);
    a == b
}

- (id)description {
    let host = env.objc.borrow::<NSDataHostObject>(this);
    let bytes = if host.length == 0 {
        &[]
    } else {
        env.mem.bytes_at(host.bytes.cast(), host.length)
    };

    // Match Foundation's useful, stable shape closely enough for logging and
    // SDKs which interpolate NSData into diagnostic strings.
    let mut result = String::with_capacity(bytes.len().saturating_mul(2) + bytes.len() / 4 + 2);
    result.push('<');
    for (idx, byte) in bytes.iter().enumerate() {
        if idx != 0 && idx % 4 == 0 {
            result.push(' ');
        }
        use std::fmt::Write;
        write!(&mut result, "{byte:02x}").unwrap();
    }
    result.push('>');
    let string = from_rust_string(env, result);
    autorelease(env, string)
}

- (())getBytes:(MutPtr<u8>)buffer length:(NSUInteger)length {
    let length = length.min(env.objc.borrow::<NSDataHostObject>(this).length);
    let range = NSRange { location: 0, length };
    msg![env; this getBytes:buffer range:range]
}

- (())getBytes:(MutPtr<u8>)buffer range:(NSRange)range {
    if range.length == 0 {
        return;
    }
    let &NSDataHostObject { bytes, length, .. } = env.objc.borrow(this);
    // TODO: throw NSRangeException if out-of-range instead of panic?
    assert!(range.location < length && range.location + range.length <= length);
    env.mem.memmove(
        buffer.cast(),
        bytes.cast_const() + range.location,
        range.length,
    );
}

- (())getBytes:(MutPtr<u8>)buffer {
    let &NSDataHostObject { bytes, length, .. } = env.objc.borrow(this);
    env.mem.memmove(
        buffer.cast(),
        bytes.cast_const(),
        length,
    );
}

@end

@implementation NSMutableData: NSData

+ (id)data {
    msg![env; this dataWithCapacity:0u32]
}

+ (id)dataWithCapacity:(NSUInteger)capacity {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithCapacity:capacity];
    autorelease(env, new)
}

+ (id)dataWithLength:(NSUInteger)length {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithLength:length];
    autorelease(env, new)
}

- (id)initWithCapacity:(NSUInteger)_capacity {
    msg![env; this init]
}

- (id)initWithLength:(NSUInteger)length {
    let host_object = env.objc.borrow_mut::<NSDataHostObject>(this);
    assert!(host_object.bytes.is_null() && host_object.length == 0);
    let alloc = env.mem.calloc(length);
    host_object.bytes = alloc;
    host_object.length = length;
    this
}

- (id)copyWithZone:(NSZonePtr)_zone {
    let bytes: ConstVoidPtr = msg![env; this bytes];
    let length: NSUInteger = msg![env; this length];
    let new = msg_class![env; NSData alloc];
    msg![env; new initWithBytes:bytes length:length]
}

- (())increaseLengthBy:(NSUInteger)add_len {
    let &NSDataHostObject { bytes, length, .. } = env.objc.borrow(this);
    let new_len = length + add_len;
    let new_bytes = env.mem.realloc(bytes, new_len);
    let host = env.objc.borrow_mut::<NSDataHostObject>(this);
    host.length = new_len;
    host.bytes = new_bytes;
    log_dbg!("increaseLengthBy bytes {:?}, new_bytes {:?}; length {}, new_len {}", bytes, new_bytes, length, new_len);
}

- (())appendData:(id)other_data { // NSData *
    let other_bytes: ConstVoidPtr = msg![env; other_data bytes];
    let other_bytes: ConstPtr<u8> = other_bytes.cast();
    let other_length: NSUInteger = msg![env; other_data length];
    log_dbg!("appendData other_data {:?}, other_bytes {:?}, other_length {}", other_data, other_bytes, other_length);
    msg![env; this appendBytes:other_bytes length:other_length]
}

- (())appendBytes:(ConstPtr<u8>)append_bytes length:(NSUInteger)append_length {
    let old_len = env.objc.borrow::<NSDataHostObject>(this).length;
    let old_bytes = env.objc.borrow::<NSDataHostObject>(this).bytes;
    () = msg![env; this increaseLengthBy:append_length];
    let &NSDataHostObject { bytes, length, .. } = env.objc.borrow(this);
    log_dbg!("appendBytes old_len {}, append_length {}, length {}", old_len, append_length, length);
    log_dbg!("appendBytes old_bytes {:?}, append_bytes {:?}, bytes {:?}", old_bytes, append_bytes, bytes);
    env.mem.memmove(bytes + old_len, append_bytes.cast(), append_length);
}

- (MutVoidPtr)mutableBytes {
    let host_obj = env.objc.borrow_mut::<NSDataHostObject>(this);
    // Empty NSMutableData is valid. Cocoa permits asking it for mutableBytes;
    // callers must still respect its zero length, and the pointer may be null.
    host_obj.bytes
}

- (())setLength:(NSUInteger)new_length {
    let &NSDataHostObject {bytes, length, .. } = env.objc.borrow(this);
    let new_bytes = env.mem.realloc(bytes, new_length);
    if new_length > length {
        env.mem.bytes_at_mut(new_bytes.cast(), new_length)[length as usize..].fill(0);
    }
    let host = env.objc.borrow_mut::<NSDataHostObject>(this);
    host.length = new_length;
    host.bytes = new_bytes;
    log_dbg!("setLength bytes {:?}, new_bytes {:?}; length {}, new_len {}", bytes, new_bytes, length, new_length);
}

@end

};

pub fn to_rust_slice(env: &mut Environment, data: id) -> &[u8] {
    let borrowed_data = env.objc.borrow::<NSDataHostObject>(data);
    assert!(!borrowed_data.bytes.is_null() && borrowed_data.length != 0);
    env.mem
        .bytes_at(borrowed_data.bytes.cast(), borrowed_data.length)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_mutable_data_allows_mutable_bytes_query() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let data: id = msg_class![env; NSMutableData dataWithLength:0u32];
        let length: NSUInteger = msg![env; data length];
        let _bytes: MutVoidPtr = msg![env; data mutableBytes];
        assert_eq!(length, 0);
    }

    #[test]
    fn options_url_reader_reports_a_missing_file() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        // Both the API under test and the NSError factory use a fifth ARM
        // argument, so host-to-guest dispatch needs synthetic stack space.
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        // Dirt reaches this API with +URLWithString: after prefixing its
        // bundle path with file:// and percent-escaping it, rather than with
        // +fileURLWithPath:. Exercise that complete shared contract here.
        let url_string =
            from_rust_string(env, "file:///missing%20dirt%20resource".to_string());
        let url: id = msg_class![env; NSURL alloc];
        let url: id = msg![env; url initWithString:url_string];
        assert!(msg![env; url isFileURL]);
        let out_error = env.mem.alloc_and_write(nil);
        let data: id = msg_class![env; NSData alloc];

        let data: id = msg![env; data initWithContentsOfURL:url
                                         options:NS_DATA_READING_UNCACHED
                                           error:out_error];

        assert_eq!(data, nil);
        let error = env.mem.read(out_error);
        assert_ne!(error, nil);
        let code: i32 = msg![env; error code];
        assert_eq!(code, NSFileReadNoSuchFileError);

        env.mem.free(out_error.cast());
        release(env, url);
        release(env, url_string);
        release(env, pool);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }
}
