#include <objc/runtime.h>
#include <stdint.h>
/**
 * This is a mock implementation of NSObject, which will be linked against
 * the tests in order to provide a superclass for them.
 */
__attribute__((objc_root_class))
@interface NSObject
{
  Class isa;
}
@end

@implementation NSObject 

+ (id)alloc
{
  return class_createInstance(self, 0);	
}

- (id)init
{
  return self;
}

- (id)self
{
  return self;
}

- (uintptr_t)hash
{
  return (uintptr_t)(void*)self;
}

- (void)dealloc
{
  object_dispose(self);
}

- (NSObject*)description
{
  return nil;
}
@end
