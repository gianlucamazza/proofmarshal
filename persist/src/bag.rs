use super::*;

use core::fmt;

use crate::marshal::blob::WriteBlob;
use crate::marshal::blob::BlobLayout;

/// An owned pointer to a value in a `Zone`.
#[derive(Debug)]
pub struct Bag<T: ?Sized + Pointee, Z: Zone> {
    ptr: Own<T,Z>,
    zone: Z,
}

impl<T: ?Sized + Pointee, Z: Zone> Bag<T,Z> {
    pub fn new(value: impl Take<T>) -> Self
        where Z: Default
    {
        Self::new_in(value, Z::allocator())
    }

    pub fn new_in(value: impl Take<T>, mut alloc: impl Alloc<Zone=Z>) -> Self {
        Self {
            ptr: alloc.alloc(value),
            zone: alloc.zone(),
        }
    }
}

impl<T: ?Sized + Load<Z>, Z: Zone> Bag<T,Z> {
    pub fn get<'a>(&'a self) -> Ref<'a, T>
        where Z: Get
    {
        self.zone.get(&self.ptr)
    }

    pub fn take<'a>(self) -> T::Owned
        where Z: Get
    {
        self.zone.take(self.ptr)
    }
}

pub struct BagSaver<T: ?Sized + Save<Z>, Z: Zone>(SaveOwnPoll<T,Z>);

impl<T: ?Sized + Pointee, Z: Zone> Save<Z> for Bag<T,Z>
where T: Save<Z>
{
    const BLOB_LAYOUT: BlobLayout = <Own<T,Z> as Save<Z>>::BLOB_LAYOUT;

    type SavePoll = BagSaver<T,Z>;
    fn save_poll(this: impl Take<Self>) -> Self::SavePoll {
        let this = this.take_sized();
        BagSaver(Own::save_poll(this.ptr))
    }
}

impl<T: ?Sized + Pointee, Z: Zone> SavePoll for BagSaver<T,Z>
where T: Save<Z>
{
    type Zone = Z;
    type Target = Bag<T,Z>;

    fn save_children<P>(&mut self, ptr_saver: &mut P) -> Poll<Result<(), P::Error>>
        where P: PtrSaver<Zone = Self::Zone>
    {
        self.0.save_children(ptr_saver)
    }

    fn encode_blob<W: WriteBlob>(&self, dst: W) -> Result<W::Done, W::Error> {
        self.0.encode_blob(dst)
    }
}

impl<T: ?Sized + Pointee, Z: Zone> fmt::Pointer for Bag<T,Z>
where Z::Ptr: fmt::Pointer,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&self.ptr, f)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::heap::Heap;

    #[test]
    fn test() {
        let _bag: Bag<_, Heap> = Bag::new(42u8);

        let _bag = Bag::new_in(42u8, Heap);

        let _bag = Bag::<[u8], Heap>::new(vec![1u8,2,3]);

        let bag = Bag::new_in(42u8, Heap);
        assert_eq!(*bag.get(), 42u8);
        assert_eq!(bag.take(), 42u8);
    }
}