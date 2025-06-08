use anyhow::bail;
use bytes::Buf;

pub mod ffi;
pub mod metadata;

mod converter_impls;
mod converter_traits;
pub use converter_traits::{FFIConverter, Lift, Lower, TypeId};

pub type FFIResult<T> = anyhow::Result<T>;

pub fn check_remaining(buf: &[u8], num_bytes: usize) -> FFIResult<()> {
    if buf.remaining() < num_bytes {
        bail!(
            "not enough bytes remaining in buffer ({} < {num_bytes})",
            buf.remaining(),
        );
    }
    Ok(())
}
