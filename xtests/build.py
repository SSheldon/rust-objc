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

def read_all_tests(src_files):
    for filename in src_files:
        for name, test in read_tests(filename):
            yield name, test

if __name__ == '__main__':
    src_files = [os.path.join(SRC_DIR, f) for f in os.listdir(SRC_DIR)]
    # The ol' zip* trick to unzip an iterator of pairs
    test_names, test_fns = zip(*read_all_tests(src_files))
    output = TEMPLATE.format(
        '\n'.join(test_fns),
        ',\n'.join('("{0}", {0})'.format(n) for n in test_names),
    )

    with open(os.path.join(TEST_DIR, 'tests.rs'), 'w') as f:
        f.write(output)
