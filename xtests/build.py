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

def should_build(output_file, src_files):
    # Reimplement make!
    # Only build if the src files were modified after the output file
    try:
        output_mtime = os.path.getmtime(output_file)
    except os.error:
        return True
    return output_mtime < max(os.path.getmtime(f) for f in src_files)

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

def list_src_files(src_dir):
    for root, _, files in os.walk(src_dir):
        for filename in files:
            yield os.path.join(root, filename)

if __name__ == '__main__':
    src_files = list(list_src_files(SRC_DIR))
    output_file = os.path.join(TEST_DIR, 'tests.rs')

    if should_build(output_file, src_files):
        # The ol' zip* trick to unzip an iterator of pairs
        test_names, test_fns = zip(*read_all_tests(src_files))
        output = TEMPLATE.format(
            '\n'.join(test_fns),
            ',\n'.join('("{0}", {0})'.format(n) for n in test_names),
        )

        with open(output_file, 'w') as f:
            f.write(output)
