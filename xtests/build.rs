use std::process::Command;

fn main() {
    let status = Command::new("python").arg("build.py").status().unwrap();
    assert!(status.success());
}
