//
//  RustObjCTests.m
//  RustObjCTests
//
//  Created by Steven Sheldon on 2/6/16.
//
//

#import <XCTest/XCTest.h>
#import "objc_tests.h"

@interface RustObjCTests : XCTestCase
@end

@implementation RustObjCTests

- (void)test {
  for (size_t i = 0; i < tests_count(); i++) {
    size_t name_len;
    const char *c_name = test_name(i, &name_len);
    NSString *name = [[NSString alloc] initWithBytes:c_name length:name_len encoding:NSUTF8StringEncoding];

    NSLog(@"Running test: %@", name);
    run_test(i);
  }
}

@end
