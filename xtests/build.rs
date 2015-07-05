use std::process::Command;

fn main() {
    Command::new("python").arg("build.py").status().unwrap();
}
