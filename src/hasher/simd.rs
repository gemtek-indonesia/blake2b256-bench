use super::{SlicedHashUpdate, OUTPUT_LENGTH};

#[derive(Clone)]
pub struct Simd {
    inner: blake2b_simd::State,
}

impl Default for Simd {
    fn default() -> Self {
        let mut hasher_params = blake2b_simd::Params::default();
        hasher_params.hash_length(OUTPUT_LENGTH);
        let inner = hasher_params.to_state();

        Self { inner }
    }
}

impl SlicedHashUpdate for Simd {
    fn update_with_slice(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    fn finalize_into(self, output: &mut [u8]) {
        let result = self.inner.finalize();
        output.copy_from_slice(result.as_bytes());
    }
}
