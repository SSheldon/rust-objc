#[cfg(feature="gnustep")]
extern crate gcc;
#[cfg(feature="gnustep")]
use std::path::PathBuf;


#[cfg(not(feature="gnustep"))]
fn compile() {
}

#[cfg(feature="gnustep")]
fn compile() {
    gcc::Config::new().flag("-lobjc")
                      .flag("-fobjc-runtime=gnustep-1.8")
                      .flag("-fno-objc-legacy-dispatch")
                      .file("NSObject.m")
                      .compile("libNSObject.a");
    let path = ::std::env::var_os("OUT_DIR").map(PathBuf::from).unwrap();
    println!("cargo:rustc-link-search=native={}", path.display()); 
}
fn main() {
    compile();
}
