# A Hat in Time Autosplitter for Livesplit's Auto Splitting Runtime

To build you need to add to the rust toolchain:

* `$ rustup target add wasm32-unknown-unknown`

Recommended to use cargo watch while developing to watch file changes:

* `cargo watch -x "build --target wasm32-unknown-unknown"`

To build for release:

* `$ cargo build --release --target wasm32-unknown-unknown`

You can find the resulting WASM file in the target forlder.
