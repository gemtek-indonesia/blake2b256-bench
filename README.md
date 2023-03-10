# Blake2b 256-bit Benchmark

This repo's purpose is to check hashing throughput differences in various [Rust](https://www.rust-lang.org/tools/install) implementation for [blake2b](https://www.blake2.net/)

## Results

### AWS - Neoverse N1

`wasm32-wasi` is about `28% slower` than `aarch64-unknown-linux-gnu`

[![blake2b256-bench with neoverse-n1](https://asciinema.org/a/548945.png)](https://asciinema.org/a/548945?autoplay=1)

### Odroid M1 - Cortex A55

`wasm32-wasi` is about `60% slower` than `aarch64-unknown-linux-gnu`

[![blake2b256-bench with cortex-a55](https://asciinema.org/a/548949.png)](https://asciinema.org/a/548949?autoplay=1)
