# Async on Embedded

Code for my Rust CTCFT talk.

[Slides](https://hackmd.io/@rust-ctcft/ryivZ5c85#/)

Note: code here is radically simplified to optimize for teaching value.
It doesn't handle a few edge cases, do not use it in production! Check
out [Embassy](https://embassy.dev) for a production-ready async embedded
runtime.

## Running the code

- Install probe-run: `cargo install probe-run`
- Plug in the nRF52840-DK using the USB port on the left, not on the bottom.
- `cargo run --release --bin x01_blocking`
