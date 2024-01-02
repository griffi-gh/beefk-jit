#[cfg_attr(target_arch="x86_64", path="./compiler_impl/x86_64.rs")]
mod _impl;
pub use _impl::*;
