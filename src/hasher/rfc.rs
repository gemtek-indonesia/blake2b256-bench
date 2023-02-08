use super::{SlicedHashUpdate, OUTPUT_LENGTH};

#[derive(Clone)]
pub struct Rfc {
    inner: blake2_rfc::blake2b::Blake2b,
}

impl Default for Rfc {
    fn default() -> Self {
        Self {
            inner: blake2_rfc::blake2b::Blake2b::new(OUTPUT_LENGTH),
        }
    }
}

impl SlicedHashUpdate for Rfc {
    fn update_with_slice(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    fn finalize_into(self, output: &mut [u8]) {
        let result = self.inner.finalize();
        output.copy_from_slice(result.as_bytes());
    }
}
