use crate::hasher::SlicedHashUpdate;
use digest::consts::U32;
use digest::generic_array::GenericArray;
use digest::Digest;

#[derive(Clone)]
pub struct RustCrypto {
    inner: blake2::Blake2b<U32>,
}

impl Default for RustCrypto {
    fn default() -> Self {
        Self {
            inner: blake2::Blake2b::<U32>::new(),
        }
    }
}

impl SlicedHashUpdate for RustCrypto {
    fn update_with_slice(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    fn finalize_into(self, output: &mut [u8]) {
        self.inner
            .finalize_into(GenericArray::from_mut_slice(output));
    }
}
