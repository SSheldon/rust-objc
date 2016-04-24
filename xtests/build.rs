extern crate regex;
extern crate walkdir;

use std::env;
use std::fs::{File, Metadata, self};
use std::io::{ErrorKind, Read, Write};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use regex::Regex;
use walkdir::{DirEntry, WalkDir};

static TEST_REGEX: &'static str =
    "#\\[test\\]\n(    fn ([^\\{]*)\\(\\) \\{(?s:.)*?\n    \\}\n)";

static TEMPLATE: &'static str = r"
use objc::*;
use objc::declare::*;
use objc::runtime::*;

use test_utils;
";

fn has_rs_ext(path: &Path) -> bool {
    path.extension().and_then(|x| x.to_str()).map_or(false, |x| x == "rs")
}

fn modified_more_recently(m1: &Metadata, m2: &Metadata) -> bool {
    m1.mtime() > m2.mtime() ||
        (m1.mtime() == m2.mtime() && m1.mtime_nsec() > m2.mtime_nsec())
}

fn should_build(output: &Path, src_files: &[DirEntry]) -> bool {
    let output_metadata = match fs::metadata(output) {
        Ok(m) => m,
        Err(ref e) if e.kind() == ErrorKind::NotFound => return true,
        Err(e) => panic!("Error getting output file metadata: {:?}", e),
    };
    src_files.iter()
        .map(|e| e.metadata().unwrap())
        .any(|m| modified_more_recently(&m, &output_metadata))
}

fn build_test_module<I: Iterator<Item=String>>(src_contents: I) -> String {
    use std::fmt::Write;

    let mut output = TEMPLATE.to_owned();
    let mut test_names = Vec::new();

    let re = Regex::new(TEST_REGEX).unwrap();
    for buf in src_contents {
        for capture in re.captures_iter(&buf) {
            output.push_str("\n");
            output.push_str(&capture[1]);

            test_names.push(capture[2].to_owned());
        }
    }

    output.push_str("\npub static TESTS: &'static [(&'static str, fn())] = &[\n");
    for test_name in &test_names {
        write!(&mut output, "(\"{0}\", {0}),\n", test_name).unwrap();
    }
    output.push_str("];\n");
    output
}

fn main() {
    let cwd = env::current_dir().unwrap();
    let output_path = cwd.join("tests.rs");
    let src_dir = cwd.parent().unwrap().join("src");

    let src_files: Vec<DirEntry> = WalkDir::new(src_dir).into_iter()
        .map(|e| e.unwrap())
        .filter(|e| e.file_type().is_file() && has_rs_ext(e.path()))
        .collect();

    if !should_build(&output_path, &src_files) {
        return;
    }

    let src_contents = src_files.iter().map(|entry| {
        let mut f = File::open(entry.path()).unwrap();
        let mut buf = String::new();
        f.read_to_string(&mut buf).unwrap();
        buf
    });
    let output = build_test_module(src_contents);

    let mut output_file = File::create(output_path).unwrap();
    output_file.write(output.as_bytes()).unwrap();
}
