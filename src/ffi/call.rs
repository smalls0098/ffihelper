use std::mem::ManuallyDrop;

use crate::ffi::buffer::FFIBuffer;

#[repr(C)]
pub struct FFIErrStatus {
    pub code: i32,
    pub error: ManuallyDrop<FFIBuffer>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FFIStatusCode {
    Success,
    Error,
    UnexpectedError,
    Cancelled,
}

impl TryFrom<i32> for FFIStatusCode {
    type Error = i32;

    fn try_from(value: i32) -> Result<Self, i32> {
        match value {
            0 => Ok(Self::Success),
            1 => Ok(Self::Error),
            2 => Ok(Self::UnexpectedError),
            3 => Ok(Self::Cancelled),
            n => Err(n),
        }
    }
}

impl Into<i32> for FFIStatusCode {
    fn into(self) -> i32 {
        self as i32
    }
}
