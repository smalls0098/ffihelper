// https://github.com/mozilla/uniffi-rs/blob/ca3f9d2facbfe1ce8648a2cda3ad855d7e6b2979/uniffi_core/src/metadata.rs

pub mod codes {
    pub const TYPE_U8: u8 = 0;
    pub const TYPE_U16: u8 = 1;
    pub const TYPE_U32: u8 = 2;
    pub const TYPE_U64: u8 = 3;
    pub const TYPE_I8: u8 = 4;
    pub const TYPE_I16: u8 = 5;
    pub const TYPE_I32: u8 = 6;
    pub const TYPE_I64: u8 = 7;
    pub const TYPE_F32: u8 = 8;
    pub const TYPE_F64: u8 = 9;
    pub const TYPE_BOOL: u8 = 10;
    pub const TYPE_STRING: u8 = 11;
    pub const TYPE_OPTION: u8 = 12;
    pub const TYPE_VEC: u8 = 13;
    pub const TYPE_HASH_MAP: u8 = 14;
}

const BUF_SIZE: usize = 16384;

#[derive(Debug)]
pub struct MetadataBuffer {
    pub bytes: [u8; BUF_SIZE],
    pub size: usize,
}

impl MetadataBuffer {
    pub const fn new() -> Self {
        Self {
            bytes: [0; BUF_SIZE],
            size: 0,
        }
    }

    pub const fn from_code(value: u8) -> Self {
        Self::new().concat_value(value)
    }

    pub const fn concat(mut self, other: MetadataBuffer) -> MetadataBuffer {
        assert!(self.size + other.size <= BUF_SIZE);
        let mut i = 0;
        while i < other.size {
            self.bytes[self.size] = other.bytes[i];
            self.size += 1;
            i += 1;
        }
        self
    }

    pub const fn concat_value(mut self, value: u8) -> Self {
        assert!(self.size < BUF_SIZE);
        self.bytes[self.size] = value;
        self.size += 1;
        self
    }

    pub const fn concat_u32(mut self, value: u32) -> Self {
        assert!(self.size + 4 <= BUF_SIZE);
        // store the value as little-endian
        self.bytes[self.size] = value as u8;
        self.bytes[self.size + 1] = (value >> 8) as u8;
        self.bytes[self.size + 2] = (value >> 16) as u8;
        self.bytes[self.size + 3] = (value >> 24) as u8;
        self.size += 4;
        self
    }

    pub const fn concat_bool(self, value: bool) -> Self {
        self.concat_value(value as u8)
    }

    pub const fn concat_str(mut self, string: &str) -> Self {
        assert!(string.len() < 256);
        assert!(self.size + string.len() < BUF_SIZE);
        self.bytes[self.size] = string.len() as u8;
        self.size += 1;
        let bytes = string.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            self.bytes[self.size] = bytes[i];
            self.size += 1;
            i += 1;
        }
        self
    }

    pub const fn concat_long_str(mut self, string: &str) -> Self {
        assert!(self.size + string.len() + 1 < BUF_SIZE);
        let [lo, hi] = (string.len() as u16).to_le_bytes();
        self.bytes[self.size] = lo;
        self.bytes[self.size + 1] = hi;
        self.size += 2;
        let bytes = string.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            self.bytes[self.size] = bytes[i];
            self.size += 1;
            i += 1;
        }
        self
    }

    pub const fn into_array<const SIZE: usize>(self) -> [u8; SIZE] {
        let mut result: [u8; SIZE] = [0; SIZE];
        let mut i = 0;
        while i < SIZE {
            result[i] = self.bytes[i];
            i += 1;
        }
        result
    }

    pub const fn checksum(&self) -> u16 {
        calc_checksum(&self.bytes, self.size)
    }
}

impl AsRef<[u8]> for MetadataBuffer {
    fn as_ref(&self) -> &[u8] {
        &self.bytes[..self.size]
    }
}

pub const fn checksum_metadata(buf: &[u8]) -> u16 {
    calc_checksum(buf, buf.len())
}

const fn calc_checksum(bytes: &[u8], size: usize) -> u16 {
    // Taken from the fnv_hash() function from the FNV crate (https://github.com/servo/rust-fnv/blob/master/lib.rs).
    // fnv_hash() hasn't been released in a version yet.
    const INITIAL_STATE: u64 = 0xcbf81ce484333325; // 魔改
    const PRIME: u64 = 0x100000001b3;

    let mut hash = INITIAL_STATE;
    let mut i = 0;
    while i < size {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(PRIME);
        i += 1;
    }
    // Convert the 64-bit hash to a 16-bit hash by XORing everything together
    (hash ^ (hash >> 16) ^ (hash >> 32) ^ (hash >> 48)) as u16
}
