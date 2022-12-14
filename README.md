# bevy-snake

This is my take on implementing the Snake game using Bevy game engine based on the excellent tutorial at <https://mbuffett.com/posts/bevy-snake-tutorial/>

Play the WASM version here online: <https://arunkd13.github.io/bevy-snake/>

## Running

### Desktop

```bash
cargo run
```

### WASM

Refer: <https://bevy-cheatbook.github.io/platforms/wasm.html>

Ensure you have WASM  target for rust compiler installed:

```bash
rustup target install wasm32-unknown-unknown
```

Ensure you have the wasm-server-runner installed locally to try out the game locally on your browser:

```bash
cargo install wasm-server-runner
```

Now you can just run the game with

```bash
cargo run --target wasm32-unknown-unknown
```

## Releasing

You need the wasm-bindgen CLI which you can install by running

```bash
cargo install wasm-bindgen-cli
```

For now I use the following to create a release WASM version

```bash
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./out/ --target web ./target/
```

Then copy it into <https://github.com/arunkd13/arunkd13.github.io/tree/main/static/bevy-snake> with an index.html file which loads the copied files.
