#[must_use]
pub const fn pack(ptr: u32, len: u32) -> u64 {
    ((ptr as u64) << 32) | (len as u64)
}

#[must_use]
pub const fn unpack(fat: u64) -> (u32, u32) {
    let ptr = (fat >> 32) as u32;
    let len = (fat & 0xFFFF_FFFF) as u32;
    (ptr, len)
}
