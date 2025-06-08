use std::mem::ManuallyDrop;

use crate::ffi::buffer::FFIBuffer;

pub trait FFIDefault {
    fn ffi_default() -> Self;
}

macro_rules! impl_ffi_default_with_default {
    ($($T:ty,)+) => { impl_ffi_default_with_default!($($T),+); };
    ($($T:ty),*) => {
            $(
                paste::paste! {
                    impl FFIDefault for $T {
                        fn ffi_default() -> Self {
                            $T::default()
                        }
                    }
                }
            )*
    };
}

impl_ffi_default_with_default! {
    bool, i8, u8, i16, u16, i32, u32, i64, u64, f32, f64
}

impl FFIDefault for () {
    fn ffi_default() {}
}

impl FFIDefault for *const std::ffi::c_void {
    fn ffi_default() -> Self {
        std::ptr::null()
    }
}

impl FFIDefault for FFIBuffer {
    fn ffi_default() -> Self {
        unsafe { Self::from_raw_parts(std::ptr::null_mut(), 0, 0) }
    }
}

impl<T> FFIDefault for Option<T> {
    fn ffi_default() -> Self {
        None
    }
}

impl FFIDefault for String {
    fn ffi_default() -> Self {
        String::new()
    }
}

impl<T> FFIDefault for Vec<T> {
    fn ffi_default() -> Self {
        vec![]
    }
}

impl<T: FFIDefault> FFIDefault for ManuallyDrop<T> {
    fn ffi_default() -> Self {
        ManuallyDrop::new(T::ffi_default())
    }
}
