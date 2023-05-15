#[cfg(not(any(
        target_os = "android",
        target_arch = "i686",
        target_arch = "aarch64",
        target_arch = "armv7",
        target_arch = "thumbv7neon",
        target_arch = "x86_64")))]
include!(concat!(env!("OUT_DIR"), "/skeptic-tests.rs"));
