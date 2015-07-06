import os
import re

TEST_DIR = os.path.dirname(__file__)
SRC_DIR = os.path.join(TEST_DIR, os.pardir, 'src')
TEST_REGEX = '#\[test\]\n(    fn ([^{]*)\(\) {(?:(?!#\[test\]).)*\n    }\n)'

TEMPLATE = """
use objc::*;
use objc::declare::*;
use objc::runtime::*;

use test_utils;

{0}

pub static TESTS: &'static [(&'static str, fn())] = &[
    {1}
];
"""

def read_tests(filename):
    with open(filename) as f:
        code = f.read()
        matches = re.findall(TEST_REGEX, code, flags=re.DOTALL)
        for test, name in matches:
            yield name, test

def read_all_tests(src_dir):
    tests = {}
    for filename in os.listdir(src_dir):
        tests.update(read_tests(os.path.join(src_dir, filename)))
    return tests

if __name__ == '__main__':
    tests = read_all_tests(SRC_DIR)
    test_fns = '\n'.join(tests.itervalues())
    test_names = ',\n'.join('("{0}", {0})'.format(n) for n in tests.iterkeys())
    output = TEMPLATE.format(test_fns, test_names)

    with open(os.path.join(TEST_DIR, 'tests.rs'), 'w') as f:
        f.write(output)
