use std::error::Error;
use std::alloc::Layout;
use std::marker::PhantomData;
use std::mem::{self, ManuallyDrop};
use std::ops;

use thiserror::Error;

use owned::IntoOwned;

use crate::pointee::Pointee;
use crate::refs::Ref;
use crate::blob::*;
use crate::ptr::Ptr;

pub mod impls;

pub trait Decode<Z> : ValidateBlob {
    fn decode_blob(blob: BlobDecoder<Z, Self>) -> Self;
}

pub trait Load<Z> : IntoOwned + ValidateBlobPtr {
    fn load_blob(blob: BlobDecoder<Z, Self>) -> Self::Owned;

    fn deref_blob<'a>(blob: BlobDecoder<'a, '_, Z, Self>) -> Ref<'a, Self> {
        Ref::Owned(Self::load_blob(blob))
    }
}

impl<Z, T: Decode<Z>> Load<Z> for T {
    fn load_blob<'a>(blob: BlobDecoder<'a, '_, Z, Self>) -> Self {
        Self::decode_blob(blob)
    }
}

pub struct BlobDecoder<'a, 'z, Z, T: ?Sized + BlobLen> {
    cursor: BlobCursor<'a, T, ValidBlob<'a, T>>,
    zone: &'z Z,
}

impl<'a, 'z, Z, T: ?Sized + BlobLen> ops::Deref for BlobDecoder<'a, 'z, Z, T> {
    type Target = BlobCursor<'a, T, ValidBlob<'a, T>>;

    fn deref(&self) -> &Self::Target {
        &self.cursor
    }
}

impl<'a, 'z, Z, T: ?Sized + BlobLen> ops::DerefMut for BlobDecoder<'a, 'z, Z, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cursor
    }
}

impl<'a, 'z, Z, T: ?Sized + BlobLen> BlobDecoder<'a, 'z, Z, T> {
    pub fn new(blob: ValidBlob<'a, T>, zone: &'z Z) -> Self {
        Self {
            cursor: blob.into(),
            zone
        }
    }

    pub unsafe fn field_unchecked<F: Decode<Z>>(&mut self) -> F {
        let blob = self.field_blob::<F>().assume_valid();
        F::decode_blob(BlobDecoder::new(blob, self.zone))
    }

    pub fn zone(&self) -> &'z Z {
        self.zone
    }

    pub fn to_value(self) -> &'a T
        where T: Persist
    {
        self.cursor.into_inner().as_value()
    }

    pub fn finish(self) {
        self.cursor.finish();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
    }
}
