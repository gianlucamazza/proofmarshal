use super::{
    Persist, Ref, Ptr,
    blob::*,
};

use core::convert::TryFrom;
use core::fmt;
use core::mem::{self, MaybeUninit};
use core::slice;
use core::num::{
    NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128,
    NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128,
};

use leint::Le;

/// A type that contains no pointers, and thus can be stored in any zone.
///
/// `Encode` and `Decode` is implemented for any `T: Primitive`
pub trait Primitive : Sized {
    type Error : super::Error;

    const BLOB_LAYOUT: BlobLayout;
    fn encode_blob<W: WriteBlob>(&self, dst: W) -> Result<W::Ok, W::Error>;

    fn validate_blob<'p, P: Ptr>(blob: Blob<'p, Self, P>) -> Result<FullyValidBlob<'p, Self, P>, Self::Error>;

    fn deref_blob<'p, P: Ptr>(blob: FullyValidBlob<'p, Self, P>) -> &'p Self
        where Self: Persist
    {
        unsafe { blob.assume_valid() }
    }
}

impl Primitive for ! {
    type Error = !;
    const BLOB_LAYOUT: BlobLayout = BlobLayout::never();

    fn encode_blob<W: WriteBlob>(&self, _: W) -> Result<W::Ok, W::Error> {
        match *self {}
    }

    fn validate_blob<'p, P: Ptr>(blob: Blob<'p, Self, P>) -> Result<FullyValidBlob<'p, Self, P>, Self::Error> {
        panic!()
    }
}

impl Primitive for () {
    type Error = !;
    const BLOB_LAYOUT: BlobLayout = BlobLayout::new(0);

    fn encode_blob<W: WriteBlob>(&self, dst: W) -> Result<W::Ok, W::Error> {
        dst.finish()
    }

    fn validate_blob<'p, P: Ptr>(blob: Blob<'p, Self, P>) -> Result<FullyValidBlob<'p, Self, P>, Self::Error> {
        unsafe { Ok(blob.assume_fully_valid()) }
    }
}

unsafe impl Persist for () {}

#[derive(Debug)]
pub struct BoolError(u8);

impl Primitive for bool {
    type Error = BoolError;
    const BLOB_LAYOUT: BlobLayout = BlobLayout::new(1);

    fn encode_blob<W: WriteBlob>(&self, dst: W) -> Result<W::Ok, W::Error> {
        dst.write_bytes(&[*self as u8])?
           .finish()
    }

    fn validate_blob<'p, P: Ptr>(blob: Blob<'p, Self, P>) -> Result<FullyValidBlob<'p, Self, P>, Self::Error> {
        match blob[0] {
            1 | 0 => unsafe { Ok(blob.assume_fully_valid()) },
            x => Err(BoolError(x)),
        }
    }
}

unsafe impl Persist for bool {}

/*
macro_rules! impl_aligned_ints {
    ($( $t:ty, )+) => {
        $(
            impl Primitive for $t {
                type Error = !;
                const BLOB_LAYOUT: BlobLayout = BlobLayout::new(mem::size_of::<Self>());

                fn encode_blob<W: WriteBlob>(&self, dst: W) -> Result<W::Ok, W::Error> {
                    dst.write_bytes(&self.to_le_bytes())?
                       .finish()
                }

                fn validate_blob<'p, P: Ptr>(blob: Blob<'p, Self, P>) -> Result<FullyValidBlob<'p, Self, P>, Self::Error> {
                    unsafe { Ok(blob.assume_fully_valid()) }
                }

                fn decode_blob<'p, P: Ptr>(blob: FullyValidBlob<'p, Self, P>) -> Self {
                    let mut r = [0; mem::size_of::<Self>()];
                    r.copy_from_slice(&blob[..]);
                    <$t>::from_le_bytes(r)
                }
            }
        )+
    }
}

impl_aligned_ints! {
    u16, u32, u64, u128,
    i16, i32, i64, i128,
}
*/

macro_rules! unsafe_impl_persist_ints {
    ($( $t:ty, )+) => {
        $(
            impl Primitive for $t {
                type Error = !;
                const BLOB_LAYOUT: BlobLayout = BlobLayout::new(mem::size_of::<Self>());

                fn encode_blob<W: WriteBlob>(&self, dst: W) -> Result<W::Ok, W::Error> {
                    let src = unsafe { slice::from_raw_parts(self as *const _ as *const u8, mem::size_of::<Self>()) };
                    dst.write_bytes(src)?
                       .finish()
                }

                fn validate_blob<'p, P: Ptr>(blob: Blob<'p, Self, P>) -> Result<FullyValidBlob<'p, Self, P>, Self::Error> {
                    unsafe { Ok(blob.assume_fully_valid()) }
                }
            }

            unsafe impl Persist for $t {}
        )+
    }
}

unsafe_impl_persist_ints! {
    u8, Le<u16>, Le<u32>, Le<u64>, Le<u128>,
    i8, Le<i16>, Le<i32>, Le<i64>, Le<i128>,
}

#[derive(Debug)]
pub struct NonZeroIntError;

macro_rules! unsafe_impl_nonzero_persist_ints {
    ($( $t:ty, )+) => {
        $(
            impl Primitive for $t {
                type Error = NonZeroIntError;
                const BLOB_LAYOUT: BlobLayout = BlobLayout::new(mem::size_of::<Self>());

                fn encode_blob<W: WriteBlob>(&self, dst: W) -> Result<W::Ok, W::Error> {
                    let src = unsafe { slice::from_raw_parts(self as *const _ as *const u8, mem::size_of::<Self>()) };
                    dst.write_bytes(src)?
                       .finish()
                }

                fn validate_blob<'p, P: Ptr>(blob: Blob<'p, Self, P>) -> Result<FullyValidBlob<'p, Self, P>, Self::Error> {
                    if blob.iter().all(|b| *b == 0) {
                        Err(NonZeroIntError)
                    } else {
                        unsafe { Ok(blob.assume_fully_valid()) }
                    }
                }
            }

            unsafe impl Persist for $t {}
        )+
    }
}

unsafe_impl_nonzero_persist_ints! {
    NonZeroU8, Le<NonZeroU16>, Le<NonZeroU32>, Le<NonZeroU64>, Le<NonZeroU128>,
    NonZeroI8, Le<NonZeroI16>, Le<NonZeroI32>, Le<NonZeroI64>, Le<NonZeroI128>,
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    use crate::pile::PileMut;

    #[test]
    fn encodings() {
        let pile = PileMut::default();

        macro_rules! t {
            ($( $value:expr => $expected:expr, )+) => {
                $({
                    let value = $value;
                    let expected = $expected;
                    assert_eq!(pile.save_to_vec(&value), &expected);
                })+
            }
        }

        t! {
            () => [],
            1u8 => [1],
            0xabcd_u16 => [0xcd, 0xab],
        }
    }
}
*/
