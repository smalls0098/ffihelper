use std::mem::ManuallyDrop;

use crate::ffi::buffer::FFIBuffer;

#[repr(C)]
pub struct FFIErrStatus {
    pub code: FFIStatusCode,
    pub error: ManuallyDrop<FFIBuffer>,
}

#[repr(u32)]
#[derive(Debug, PartialEq, Eq)]
pub enum FFIStatusCode {
    Success,
    Error,
    UnexpectedError,
    Cancelled,
}

impl TryFrom<u32> for FFIStatusCode {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, u32> {
        match value {
            0 => Ok(Self::Success),
            1 => Ok(Self::Error),
            2 => Ok(Self::UnexpectedError),
            3 => Ok(Self::Cancelled),
            n => Err(n),
        }
    }
}

impl Into<u32> for FFIStatusCode {
    fn into(self) -> u32 {
        self as u32
    }
}
