use anyhow::bail;
use bytes::Buf;

use crate::{
    ffi::{buffer::FFIBuffer, default::FFIDefault},
    metadata::MetadataBuffer,
    FFIResult,
};

pub unsafe trait FFIConverter<UT>: Sized {
    type FFIType: FFIDefault;

    fn lower(obj: Self) -> Self::FFIType;
    fn write(obj: Self, buf: &mut Vec<u8>);

    fn try_lift(v: Self::FFIType) -> FFIResult<Self>;
    fn try_read(buf: &mut &[u8]) -> FFIResult<Self>;

    const TYPE_ID_META: MetadataBuffer;
}

pub unsafe trait Lift<UT>: Sized {
    type FFIType;

    fn try_lift(v: Self::FFIType) -> FFIResult<Self>;
    fn try_read(buf: &mut &[u8]) -> FFIResult<Self>;

    /// Convenience method
    fn try_lift_from_buffer(v: FFIBuffer) -> FFIResult<Self> {
        let vec = v.destroy_into_vec();
        let mut buf = vec.as_slice();
        let value = Self::try_read(&mut buf)?;
        match Buf::remaining(&buf) {
            0 => Ok(value),
            n => bail!("junk data left in buffer after lifting (count: {n})",),
        }
    }
}

pub unsafe trait Lower<UT>: Sized {
    type FFIType: FFIDefault;

    fn lower(obj: Self) -> Self::FFIType;
    fn write(obj: Self, buf: &mut Vec<u8>);

    /// Convenience method
    fn lower_into_buffer(obj: Self) -> FFIBuffer {
        let mut buf = Vec::new();
        Self::write(obj, &mut buf);
        FFIBuffer::from_vec(buf)
    }
}

pub trait TypeId<UT> {
    const TYPE_ID_META: MetadataBuffer;
}

#[macro_export]
#[allow(clippy::crate_in_macro_def)]
macro_rules! derive_ffi_traits {
    (blanket $ty:ty) => {
        $crate::derive_ffi_traits!(impl<UT> Lower<UT> for $ty);
        $crate::derive_ffi_traits!(impl<UT> Lift<UT> for $ty);
        $crate::derive_ffi_traits!(impl<UT> TypeId<UT> for $ty);
    };

    (impl $(<$($generic:ident),*>)? Lower<$ut:path> for $ty:ty $(where $($where:tt)*)?) => {
        unsafe impl $(<$($generic),*>)* $crate::converter_traits::Lower<$ut> for $ty $(where $($where)*)*
        {
            type FFIType = <Self as $crate::converter_traits::FFIConverter<$ut>>::FFIType;
            fn lower(obj: Self) -> Self::FFIType {
                <Self as $crate::converter_traits::FFIConverter<$ut>>::lower(obj)
            }
            fn write(obj: Self, buf: &mut ::std::vec::Vec<u8>) {
                <Self as $crate::converter_traits::FFIConverter<$ut>>::write(obj, buf)
            }
        }
    };

    (impl $(<$($generic:ident),*>)? Lift<$ut:path> for $ty:ty $(where $($where:tt)*)?) => {
        unsafe impl $(<$($generic),*>)* $crate::converter_traits::Lift<$ut> for $ty $(where $($where)*)*
        {
            type FFIType = <Self as $crate::converter_traits::FFIConverter<$ut>>::FFIType;

            fn try_lift(v: Self::FFIType) -> $crate::FFIResult<Self> {
                <Self as $crate::converter_traits::FFIConverter<$ut>>::try_lift(v)
            }

            fn try_read(buf: &mut &[u8]) -> $crate::FFIResult<Self> {
                <Self as $crate::converter_traits::FFIConverter<$ut>>::try_read(buf)
            }
        }
    };

    (impl $(<$($generic:ident),*>)? TypeId<$ut:path> for $ty:ty $(where $($where:tt)*)?) => {
        impl $(<$($generic),*>)* $crate::converter_traits::TypeId<$ut> for $ty $(where $($where)*)*
        {
            const TYPE_ID_META: $crate::metadata::MetadataBuffer = <Self as $crate::converter_traits::FFIConverter<$ut>>::TYPE_ID_META;
        }
    };
}
