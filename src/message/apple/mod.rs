use super::Super;

#[cfg(target_arch = "x86")]
#[path = "x86.rs"]
mod arch;
#[cfg(target_arch = "x86_64")]
#[path = "x86_64.rs"]
mod arch;
#[cfg(target_arch = "arm")]
#[path = "arm.rs"]
mod arch;
#[cfg(target_arch = "aarch64")]
#[path = "arm64.rs"]
mod arch;

pub use self::arch::{msg_send_fn, msg_send_super_fn};
