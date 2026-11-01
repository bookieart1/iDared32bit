/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
// Native Foundation reference fixture, not a historical iPhone OS runtime.
#import <Foundation/Foundation.h>
#include <assert.h>
#include <stdio.h>

static unsigned liveTokens;
@interface CopyToken : NSObject
@end
@implementation CopyToken
- (id)init {
    self = [super init];
    if (self) liveTokens++;
    return self;
}
- (void)dealloc {
    liveTokens--;
    [super dealloc];
}
@end

@interface DefaultsObserver : NSObject {
@public unsigned changes;
}
@end
@implementation DefaultsObserver
- (void)observeValueForKeyPath:(NSString *)keyPath ofObject:(id)object
                       change:(NSDictionary *)change context:(void *)context {
    changes++;
}
@end

int main(void) {
    NSAutoreleasePool *pool = [NSAutoreleasePool new];
    NSString *suite = [@"org.touchhle.foundation-fixture."
        stringByAppendingString:[[NSUUID UUID] UUIDString]];
    NSUserDefaults *defaults = [[NSUserDefaults alloc] initWithSuiteName:suite];
    [defaults registerDefaults:@{@"setting": @"fallback"}];
    [defaults setObject:@"saved" forKey:@"setting"];
    [defaults setObject:@"preserve" forKey:@"unrelated"];
    DefaultsObserver *observer = [DefaultsObserver new];
    [defaults addObserver:observer forKeyPath:@"setting" options:0 context:NULL];
    [defaults setObject:nil forKey:@"setting"];
    assert(observer->changes == 1);
    assert([[defaults objectForKey:@"setting"] isEqual:@"fallback"]);
    [defaults setObject:nil forKey:@"absent"];
    assert([defaults objectForKey:@"absent"] == nil);
    assert([[defaults objectForKey:@"unrelated"] isEqual:@"preserve"]);
    [defaults synchronize];
    NSDictionary *persisted = [defaults persistentDomainForName:suite];
    assert([persisted objectForKey:@"setting"] == nil);
    assert([[persisted objectForKey:@"unrelated"] isEqual:@"preserve"]);
    [defaults removeObserver:observer forKeyPath:@"setting"];
    [observer release];
    [defaults removePersistentDomainForName:suite];
    [defaults release];

    BOOL rejected = NO;
    @try {
        [[NSMutableDictionary dictionary] setObject:nil forKey:@"invalid"];
    } @catch (NSException *exception) {
        rejected = [exception.name isEqual:NSInvalidArgumentException];
    }
    assert(rejected);

    CopyToken *element = [CopyToken new];
    NSMutableSet *source = [[NSMutableSet alloc] initWithObjects:element, nil];
    NSMutableSet *mutable = [source mutableCopy];
    NSSet *immutable = [source copy];
    NSMutableSet *another = [immutable mutableCopy];
    assert([mutable isKindOfClass:[NSMutableSet class]]);
    assert(![immutable isKindOfClass:[NSMutableSet class]]);
    assert(mutable != source && another != immutable);
    assert([immutable anyObject] == element);
    [source removeAllObjects];
    assert(mutable.count == 1 && immutable.count == 1);
    [mutable removeObject:element];
    assert(immutable.count == 1);
    [source release];
    [mutable release];
    [another release];
    [element release];
    [pool drain];
    assert(liveTokens == 1 && [immutable anyObject] == element);
    [immutable release];
    assert(liveTokens == 0);
    puts("Foundation defaults/removal and set-copy reference contracts passed.");
    return 0;
}
