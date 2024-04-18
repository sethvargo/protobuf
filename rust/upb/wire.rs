use crate::{
    upb_ExtensionRegistry, upb_Message, upb_Message_New, upb_MiniTable, Arena, OwnedData, RawArena,
    RawMessage,
};
use std::ptr::NonNull;

// LINT.IfChange(encode_status)
#[repr(C)]
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum EncodeStatus {
    Ok = 0,
    OutOfMemory = 1,
    MaxDepthExceeded = 2,
    MissingRequired = 3,
}
// LINT.ThenChange()

// LINT.IfChange(decode_status)
#[repr(C)]
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum DecodeStatus {
    Ok = 0,
    Malformed = 1,
    OutOfMemory = 2,
    BadUtf8 = 3,
    MaxDepthExceeded = 4,
    MissingRequired = 5,
    UnlinkedSubMessage = 6,
}
// LINT.ThenChange()

/// If Err, then EncodeStatus != Ok.
///
/// SAFETY:
/// - `msg` must be associated with `mini_table`.
pub unsafe fn encode(
    msg: RawMessage,
    mini_table: *const upb_MiniTable,
) -> Result<OwnedData<[u8]>, EncodeStatus> {
    let arena = Arena::new();
    let mut buf: *mut u8 = std::ptr::null_mut();
    let mut len = 0usize;
    let status = upb_Encode(msg, mini_table, 0, arena.raw(), &mut buf, &mut len);
    if status == EncodeStatus::Ok {
        assert!(!buf.is_null()); // EncodeStatus Ok should never return NULL data, even for len=0.
        // SAFETY: upb guarantees that `buf` is valid to read for `len`.
        let slice = NonNull::new_unchecked(std::ptr::slice_from_raw_parts_mut(buf, len));
        Ok(OwnedData::new(slice, arena))
    } else {
        Err(status)
    }
}

/// Decodes the provided buffer into a new message. If Err, then DecodeStatus !=
/// Ok.
pub fn decode_new(
    buf: &[u8],
    mini_table: *const upb_MiniTable,
) -> Result<OwnedData<RawMessage>, DecodeStatus> {
    let arena = Arena::new();
    // SAFETY: No constraints.
    let msg = unsafe { upb_Message_New(mini_table, arena.raw()).unwrap() };

    // SAFETY: `msg` was just created as mutable and associated with `mini_table`.
    let result = unsafe { decode(buf, msg, mini_table, &arena) };

    // SAFETY:
    // - `msg` was allocated using `arena.
    // - `msg` will not be mutated after this line.
    result.map(|_| unsafe { OwnedData::new(msg, arena) })
}

/// Decodes into the provided message (merge semantics). If Err, then
/// DecodeStatus != Ok.
///
/// SAFETY:
/// - `msg` must be mutable.
/// - `msg` must be associated with `mini_table`.
pub unsafe fn decode(
    buf: &[u8],
    msg: RawMessage,
    mini_table: *const upb_MiniTable,
    arena: &Arena,
) -> Result<(), DecodeStatus> {
    let len = buf.len();
    let buf = buf.as_ptr();
    let status = upb_Decode(buf, len, msg, mini_table, std::ptr::null(), 0, arena.raw());
    match status {
        DecodeStatus::Ok => Ok(()),
        _ => Err(status),
    }
}

extern "C" {
    pub fn upb_Encode(
        msg: RawMessage,
        mini_table: *const upb_MiniTable,
        options: i32,
        arena: RawArena,
        buf: *mut *mut u8,
        buf_size: *mut usize,
    ) -> EncodeStatus;

    pub fn upb_Decode(
        buf: *const u8,
        buf_size: usize,
        msg: RawMessage,
        mini_table: *const upb_MiniTable,
        extreg: *const upb_ExtensionRegistry,
        options: i32,
        arena: RawArena,
    ) -> DecodeStatus;
}
