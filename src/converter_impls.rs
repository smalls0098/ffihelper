use std::collections::HashMap;

use anyhow::bail;
use bytes::{Buf, BufMut};

use crate::{
    check_remaining,
    converter_traits::{FFIConverter, Lift, Lower, TypeId},
    derive_ffi_traits,
    ffi::buffer::FFIBuffer,
    metadata,
    metadata::MetadataBuffer,
    FFIResult,
};

macro_rules! impl_ffi_converter_for_num_primitive {
    ($T:ty, $type_code:expr) => {
        paste::paste! {
            unsafe impl<UT> FFIConverter<UT> for $T {
                type FFIType = $T;

                fn lower(obj: $T) -> Self::FFIType {
                    obj
                }

                fn try_lift(v: Self::FFIType) -> FFIResult<$T> {
                    Ok(v)
                }

                fn write(obj: $T, buf: &mut Vec<u8>) {
                    buf.[<put_ $T>](obj);
                }

                fn try_read(buf: &mut &[u8]) -> FFIResult<$T> {
                    check_remaining(buf, std::mem::size_of::<$T>())?;
                    Ok(buf.[<get_ $T>]())
                }

                const TYPE_ID_META: MetadataBuffer = MetadataBuffer::from_code($type_code);
            }
        }
    };
}

impl_ffi_converter_for_num_primitive!(u8, metadata::codes::TYPE_U8);
impl_ffi_converter_for_num_primitive!(i8, metadata::codes::TYPE_I8);
impl_ffi_converter_for_num_primitive!(u16, metadata::codes::TYPE_U16);
impl_ffi_converter_for_num_primitive!(i16, metadata::codes::TYPE_I16);
impl_ffi_converter_for_num_primitive!(u32, metadata::codes::TYPE_U32);
impl_ffi_converter_for_num_primitive!(i32, metadata::codes::TYPE_I32);
impl_ffi_converter_for_num_primitive!(u64, metadata::codes::TYPE_U64);
impl_ffi_converter_for_num_primitive!(i64, metadata::codes::TYPE_I64);
impl_ffi_converter_for_num_primitive!(f32, metadata::codes::TYPE_F32);
impl_ffi_converter_for_num_primitive!(f64, metadata::codes::TYPE_F64);

unsafe impl<UT> FFIConverter<UT> for bool {
    type FFIType = i8;
    fn lower(obj: bool) -> Self::FFIType {
        i8::from(obj)
    }
    fn write(obj: bool, buf: &mut Vec<u8>) {
        buf.put_i8(<Self as FFIConverter<UT>>::lower(obj));
    }
    fn try_lift(v: Self::FFIType) -> FFIResult<bool> {
        Ok(match v {
            0 => false,
            1 => true,
            _ => bail!("unexpected byte for Boolean"),
        })
    }
    fn try_read(buf: &mut &[u8]) -> FFIResult<bool> {
        check_remaining(buf, 1)?;
        <Self as FFIConverter<UT>>::try_lift(buf.get_i8())
    }
    const TYPE_ID_META: MetadataBuffer = MetadataBuffer::from_code(metadata::codes::TYPE_BOOL);
}

unsafe impl<UT> FFIConverter<UT> for String {
    type FFIType = FFIBuffer;

    fn lower(obj: String) -> Self::FFIType {
        FFIBuffer::from_vec(obj.into_bytes())
    }

    fn write(obj: String, buf: &mut Vec<u8>) {
        let len = i32::try_from(obj.len()).unwrap();
        buf.put_i32(len);
        buf.put(obj.as_bytes());
    }

    fn try_lift(v: Self::FFIType) -> FFIResult<String> {
        let v = v.destroy_into_vec();
        Ok(unsafe { String::from_utf8_unchecked(v) })
    }

    fn try_read(buf: &mut &[u8]) -> FFIResult<String> {
        check_remaining(buf, 4)?;
        let len = usize::try_from(buf.get_i32())?;
        check_remaining(buf, len)?;
        let bytes = &buf.chunk()[..len];
        let res = String::from_utf8(bytes.to_vec())?;
        buf.advance(len);
        Ok(res)
    }

    const TYPE_ID_META: MetadataBuffer = MetadataBuffer::from_code(metadata::codes::TYPE_STRING);
}

unsafe impl<UT, T: Lower<UT>> Lower<UT> for Option<T> {
    type FFIType = FFIBuffer;
    fn lower(obj: Option<T>) -> FFIBuffer {
        Self::lower_into_buffer(obj)
    }
    fn write(obj: Option<T>, buf: &mut Vec<u8>) {
        match obj {
            None => buf.put_i8(0),
            Some(v) => {
                buf.put_i8(1);
                T::write(v, buf);
            }
        }
    }
}

unsafe impl<UT, T: Lift<UT>> Lift<UT> for Option<T> {
    type FFIType = FFIBuffer;
    fn try_lift(buf: FFIBuffer) -> FFIResult<Option<T>> {
        Self::try_lift_from_buffer(buf)
    }
    fn try_read(buf: &mut &[u8]) -> FFIResult<Option<T>> {
        check_remaining(buf, 1)?;
        Ok(match buf.get_i8() {
            0 => None,
            1 => Some(T::try_read(buf)?),
            _ => bail!("unexpected tag byte for Option"),
        })
    }
}

impl<UT, T: TypeId<UT>> TypeId<UT> for Option<T> {
    const TYPE_ID_META: MetadataBuffer =
        MetadataBuffer::from_code(metadata::codes::TYPE_OPTION).concat(T::TYPE_ID_META);
}

unsafe impl<UT, T: Lower<UT>> Lower<UT> for Vec<T> {
    type FFIType = FFIBuffer;
    fn lower(obj: Vec<T>) -> FFIBuffer {
        Self::lower_into_buffer(obj)
    }
    fn write(obj: Vec<T>, buf: &mut Vec<u8>) {
        let len = i32::try_from(obj.len()).unwrap();
        buf.put_i32(len);
        for item in obj {
            <T as Lower<UT>>::write(item, buf);
        }
    }
}

unsafe impl<UT, T: Lift<UT>> Lift<UT> for Vec<T> {
    type FFIType = FFIBuffer;
    fn try_lift(buf: FFIBuffer) -> FFIResult<Vec<T>> {
        Self::try_lift_from_buffer(buf)
    }
    fn try_read(buf: &mut &[u8]) -> FFIResult<Vec<T>> {
        check_remaining(buf, 4)?;
        let len = usize::try_from(buf.get_i32())?;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(<T as Lift<UT>>::try_read(buf)?)
        }
        Ok(vec)
    }
}

impl<UT, T: TypeId<UT>> TypeId<UT> for Vec<T> {
    const TYPE_ID_META: MetadataBuffer =
        MetadataBuffer::from_code(metadata::codes::TYPE_VEC).concat(T::TYPE_ID_META);
}

unsafe impl<K, V, UT> Lower<UT> for HashMap<K, V>
where
    K: Lower<UT> + std::hash::Hash + Eq,
    V: Lower<UT>,
{
    type FFIType = FFIBuffer;
    fn lower(obj: HashMap<K, V>) -> FFIBuffer {
        Self::lower_into_buffer(obj)
    }
    fn write(obj: HashMap<K, V>, buf: &mut Vec<u8>) {
        let len = i32::try_from(obj.len()).unwrap();
        buf.put_i32(len);
        for (key, value) in obj {
            <K as Lower<UT>>::write(key, buf);
            <V as Lower<UT>>::write(value, buf);
        }
    }
}

unsafe impl<K, V, UT> Lift<UT> for HashMap<K, V>
where
    K: Lift<UT> + std::hash::Hash + Eq,
    V: Lift<UT>,
{
    type FFIType = FFIBuffer;
    fn try_lift(buf: FFIBuffer) -> FFIResult<HashMap<K, V>> {
        Self::try_lift_from_buffer(buf)
    }
    fn try_read(buf: &mut &[u8]) -> FFIResult<HashMap<K, V>> {
        check_remaining(buf, 4)?;
        let len = usize::try_from(buf.get_i32())?;
        let mut map = HashMap::with_capacity(len);
        for _ in 0..len {
            let key = <K as Lift<UT>>::try_read(buf)?;
            let value = <V as Lift<UT>>::try_read(buf)?;
            map.insert(key, value);
        }
        Ok(map)
    }
}

impl<K, V, UT> TypeId<UT> for HashMap<K, V>
where
    K: TypeId<UT> + std::hash::Hash + Eq,
    V: TypeId<UT>,
{
    const TYPE_ID_META: MetadataBuffer = MetadataBuffer::from_code(metadata::codes::TYPE_HASH_MAP)
        .concat(K::TYPE_ID_META)
        .concat(V::TYPE_ID_META);
}

derive_ffi_traits!(blanket u8);
derive_ffi_traits!(blanket i8);
derive_ffi_traits!(blanket u16);
derive_ffi_traits!(blanket i16);
derive_ffi_traits!(blanket u32);
derive_ffi_traits!(blanket i32);
derive_ffi_traits!(blanket u64);
derive_ffi_traits!(blanket i64);
derive_ffi_traits!(blanket f32);
derive_ffi_traits!(blanket f64);
derive_ffi_traits!(blanket bool);
derive_ffi_traits!(blanket String);
