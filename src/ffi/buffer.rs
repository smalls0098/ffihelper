#[repr(C)]
#[derive(Debug)]
pub struct FFIBuffer {
    pub(crate) capacity: u64,
    pub(crate) len: u64,
    pub(crate) data: *mut u8,
}

unsafe impl Send for FFIBuffer {}

impl FFIBuffer {
    pub fn new() -> Self {
        Self::from_vec(Vec::new())
    }

    pub unsafe fn from_raw_parts(data: *mut u8, len: u64, capacity: u64) -> Self {
        Self {
            capacity,
            len,
            data,
        }
    }

    pub fn len(&self) -> usize {
        self.len
            .try_into()
            .expect("buffer length negative or overflowed")
    }

    pub fn capacity(&self) -> usize {
        self.capacity
            .try_into()
            .expect("buffer length negative or overflowed")
    }

    pub fn data_pointer(&self) -> *const u8 {
        self.data
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn new_with_size(size: u64) -> Self {
        Self::from_vec(vec![0u8; size as usize])
    }

    pub fn from_vec(v: Vec<u8>) -> Self {
        let capacity = u64::try_from(v.capacity()).expect("buffer capacity cannot fit into a u64.");
        let len = u64::try_from(v.len()).expect("buffer length cannot fit into a u64.");
        let mut v = std::mem::ManuallyDrop::new(v);
        unsafe { Self::from_raw_parts(v.as_mut_ptr(), len, capacity) }
    }

    pub fn destroy_into_vec(self) -> Vec<u8> {
        // Rust will never give us a null `data` pointer for a `Vec`, but
        // foreign-language code can use it to cheaply pass an empty buffer.
        if self.data.is_null() {
            assert_eq!(self.capacity, 0, "null RustBuffer had non-zero capacity");
            assert_eq!(self.len, 0, "null RustBuffer had non-zero length");
            vec![]
        } else {
            let capacity: usize = self
                .capacity
                .try_into()
                .expect("buffer capacity negative or overflowed");
            let len: usize = self
                .len
                .try_into()
                .expect("buffer length negative or overflowed");
            assert!(len <= capacity, "RustBuffer length exceeds capacity");
            unsafe { Vec::from_raw_parts(self.data, len, capacity) }
        }
    }

    pub fn destroy(self) {
        drop(self.destroy_into_vec())
    }
}

impl Default for FFIBuffer {
    fn default() -> Self {
        Self::new()
    }
}
