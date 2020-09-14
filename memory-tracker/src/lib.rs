#![feature(thread_id_value)]
/*
#[cfg(all(not(target_env = "msvc"), not(target_os = "macos"), feature = "profiling"))]
mod jemalloc;
#[cfg(not(all(not(target_env = "msvc"), not(target_os = "macos"), feature = "profiling")))]
mod jemalloc {
    pub fn jemalloc_profiling_dump(_: &str) -> Result<(), String> {
        Err("jemalloc profiling dump: unsupported".to_string())
    }
}

#[cfg(all(not(target_env = "msvc"), not(target_os = "macos")))]
mod process;
#[cfg(not(all(not(target_env = "msvc"), not(target_os = "macos"))))]
mod process {
    use log::info;
    use std::sync;

    pub fn track_current_process(_: u64) {
        info!("track current process: unsupported");
    }
}     */

mod jemalloc;
mod process;

use log::info;
pub fn test() {
    info!("PIOTR test");
}
pub mod allocator;
pub mod utils;

// pub use jemalloc::jemalloc_profiling_dump;
pub use process::track_current_process;
