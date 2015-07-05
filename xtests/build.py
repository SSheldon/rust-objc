import os
import re

TEST_DIR = os.path.dirname(__file__)
SRC_DIR = os.path.join(TEST_DIR, os.pardir, 'src')
TEST_REGEX = '#\[test\]\n(    fn ([^{]*)\(\) {(?:(?!#\[test\]).)*\n    }\n)'

TEMPLATE = """
#[macro_use]
extern crate objc;

use objc::*;
use objc::declare::*;
use objc::runtime::*;

mod id {{
    use objc::runtime::*;

    {0}
}}

mod test_utils {{
    use objc::*;
    use objc::declare::*;
    use objc::runtime::*;

    {1}
}}

{2}

pub const TESTS: &'static [(&'static str, fn())] = &[
    {3}
];
"""

def read_module(filename):
    internal_use = ('use {', 'use runtime::', 'use declare::')
    with open(filename) as f:
        return ''.join(l for l in f if not l.startswith(internal_use))

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
    id_mod = read_module(os.path.join(SRC_DIR, 'id.rs'))
    test_utils_mod = read_module(os.path.join(SRC_DIR, 'test_utils.rs'))
    tests = read_all_tests(SRC_DIR)
    test_fns = '\n'.join(tests.itervalues())
    test_names = ',\n'.join('("{0}", {0})'.format(n) for n in tests.iterkeys())
    output = TEMPLATE.format(id_mod, test_utils_mod, test_fns, test_names)

    with open(os.path.join(TEST_DIR, 'lib.rs'), 'w') as f:
        f.write(output)
