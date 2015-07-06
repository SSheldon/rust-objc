//
//  AppDelegate.m
//  RustObjCTests
//
//  Created by Steven Sheldon on 7/5/15.
//
//

#import "AppDelegate.h"
#import "objc_tests.h"

@implementation AppDelegate

- (BOOL)application:(UIApplication *)application didFinishLaunchingWithOptions:(NSDictionary *)launchOptions {
  for (size_t i = 0; i < tests_count(); i++) {
    size_t name_len;
    const char *c_name = test_name(i, &name_len);
    NSString *name = [[NSString alloc] initWithBytes:c_name length:name_len encoding:NSUTF8StringEncoding];

    NSLog(@"Running test: %@", name);
    run_test(i);
  }
  return YES;
}

@end
