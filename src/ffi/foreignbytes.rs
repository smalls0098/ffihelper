#[repr(C)]
pub struct FFIForeignBytes {
    len: i32,
    data: *const u8,
}

impl FFIForeignBytes {
    pub unsafe fn from_raw_parts(data: *const u8, len: i32) -> Self {
        Self { len, data }
    }

    pub fn as_slice(&self) -> &[u8] {
        if self.data.is_null() {
            assert_eq!(self.len, 0, "null ForeignBytes had non-zero length");
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.data, self.len()) }
        }
    }

    pub fn len(&self) -> usize {
        self.len
            .try_into()
            .expect("bytes length negative or overflowed")
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}
