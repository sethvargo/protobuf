use crate::Arena;
use std::fmt::{self, Debug};
use std::ops::Deref;
use std::ptr::NonNull;

/// An 'owned' T, conceptually similar to a Box<T> where the T is
/// something in a upb Arena. By holding the data pointer and the owned arena
/// together the data lifetime will be maintained.
pub struct OwnedData<T: ?Sized> {
    data: NonNull<T>,
    arena: Arena,
}

impl<T: ?Sized> OwnedData<T> {
    /// Construct `OwnedData` from raw pointers and its owning arena.
    ///
    /// # Safety
    /// - `data` must satisfy the safety constraints of pointer::as_ref::<'a>()
    ///   where 'a is the passed arena's lifetime (`data` must be valid, have
    ///   lifetime at least as long as `arena`, and must not mutate while this
    ///   struct exists)
    pub unsafe fn new(data: NonNull<T>, arena: Arena) -> Self {
        OwnedData { arena, data }
    }

    pub fn data(&self) -> *const T {
        self.data.as_ptr()
    }

    pub fn as_ref(&self) -> &T {
        // SAFETY:
        // - `data` is valid under the conditions set on ::new().
        unsafe { self.data.as_ref() }
    }

    pub fn into_parts(self) -> (NonNull<T>, Arena) {
        (self.data, self.arena)
    }
}

impl<T: ?Sized> Deref for OwnedData<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T: ?Sized> AsRef<T> for OwnedData<T> {
    fn as_ref(&self) -> &T {
        self.as_ref()
    }
}

impl<T: Debug> Debug for OwnedData<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.deref(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str;

    #[test]
    fn test_byte_slice_pointer_roundtrip() {
        let arena = Arena::new();
        let original_data: &'static [u8] = b"Hello world";
        let owned_data = unsafe { OwnedData::new(original_data.into(), arena) };
        assert_eq!(&*owned_data, b"Hello world");
    }

    #[test]
    fn test_alloc_str_roundtrip() {
        let arena = Arena::new();
        let s: &str = "Hello";
        let arena_alloc_str: NonNull<str> = arena.copy_str_in(s).into();
        let owned_data = unsafe { OwnedData::new(arena_alloc_str, arena) };
        assert_eq!(&*owned_data, s);
    }

    #[test]
    fn test_sized_type_roundtrip() {
        let arena = Arena::new();
        let arena_alloc_u32: NonNull<u32> = arena.copy_in(&7u32).into();
        let owned_data = unsafe { OwnedData::new(arena_alloc_u32, arena) };
        assert_eq!(*owned_data, 7);
    }
}
