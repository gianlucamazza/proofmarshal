//! Zone of missing data.

use std::ptr;

use thiserror::Error;

use super::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Missing;

#[derive(Debug, Error)]
#[error("missing")]
pub struct MissingError;

fn make_missing_ptr<T: ?Sized + Pointee>(metadata: T::Metadata) -> OwnedPtr<T, Missing> {
    // SAFETY: Missing pointers are trivially valid
    unsafe {
        OwnedPtr::new_unchecked(ValidPtr::new_unchecked(
            FatPtr {
                raw: (),
                metadata,
            }
        ))
    }
}

impl Zone for Missing {
    type Ptr = ();
    type Persist = Self;
    type PersistPtr = ();

    type Error = MissingError;

    fn alloc<T: ?Sized + Pointee>(src: impl Take<T>) -> OwnedPtr<T, Self> {
        src.take_unsized(|src| {
            let metadata = T::metadata(src);

            // src is a &mut ManuallyDrop<T>, so we need to specify that we want to drop a T, or
            // the drop will do nothing
            unsafe { ptr::drop_in_place(src as *mut _ as *mut T) };

            make_missing_ptr(metadata)
        })
    }

    fn duplicate(&self) -> Self {
        Missing
    }

    unsafe fn clone_ptr_unchecked<T: Clone>(ptr: &Self::Ptr) -> Self::Ptr {
        *ptr
    }

    fn try_get_dirty<T: ?Sized + Pointee>(ptr: &ValidPtr<T, Self>) -> Result<&T, FatPtr<T, Self::Persist>> {
        Err(FatPtr {
            raw: (),
            metadata: ptr.metadata,
        })
    }

    fn try_take_dirty_unsized<T: ?Sized + Pointee, R>(
        ptr: OwnedPtr<T, Self>,
        f: impl FnOnce(Result<&mut ManuallyDrop<T>, FatPtr<T, Self::Persist>>) -> R,
    ) -> R
    {
        let fatptr = ptr.into_inner().into_inner();
        f(Err(fatptr))
    }
}

impl TryGet for Missing {
    unsafe fn try_get_unchecked<'a, T: ?Sized + Load<Self>>(&self, _ptr: &'a (), _metadata: T::Metadata)
        -> Result<Ref<'a, T, Self>, Self::Error>
    {
        Err(MissingError)
    }

    unsafe fn try_take_unchecked<T: ?Sized + Load<Self>>(&self, _: (), _metadata: T::Metadata)
        -> Result<Own<T::Owned, Self>, Self::Error>
    {
        Err(MissingError)
    }
}

impl TryGetMut for Missing {
    unsafe fn try_get_mut_unchecked<'a, T: ?Sized + Load<Self>>(&self, _: &'a mut (), _: T::Metadata)
        -> Result<RefMut<'a, T, Self>, Self::Error>
    {
        Err(MissingError)
    }
}

impl Alloc for Missing {
    fn alloc<T: ?Sized + Pointee>(&self, src: impl Take<T>) -> OwnedPtr<T, Self> {
        <Self as Zone>::alloc(src)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use dropcheck::DropCheck;

    #[test]
    fn alloc_drops() {
        let check = DropCheck::new();
        let _ = Missing.alloc(check.token());
        <Missing as Zone>::alloc(check.token());
    }
}
