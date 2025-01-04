# ai-rustdoc

Generate Rust API docs and definitions to be used as context for coding AIs.

## Description

Tired of programming with AIs that simply don't know what types and methods are
available in your Rust project? Or perhaps how the APIs of the popular crates
you use have changed in more recent versions? Not to mention the more obscure
crates which AIs are not trained on...

`ai-rustdoc` takes the JSON output of [`rustdoc`](https://doc.rust-lang.org/rustdoc/what-is-rustdoc.html):

```bash
cargo +nightly rustdoc --manifest-path <my_crate>/Cargo.toml -- -Z unstable-options --output-format json
```

and generates neatly organized markdown files containing all APIs in the crate:

---

## `hex.md`

### `fn hex::encode`

```rust
pub fn encode(bytes: &[u8]) -> String;
```

Convert a byte slice to an owned hex string. If you simply need to display a
byte slice as hex, use `display` instead, which avoids the allocation.

### `fn hex::display`

```rust
pub fn display(bytes: &[u8]) -> HexDisplay<'_>;
```

Get a `HexDisplay` which provides a `Debug` and `Display` impl for the
given byte slice. Useful for displaying a hex value without allocating.

Example:

```
let bytes = [69u8; 32];
println!("Bytes as hex: {}", hex::display(&bytes));
```

### `struct hex::HexDisplay;`

Provides `Debug` and `Display` impls for a byte slice.
Useful for displaying hex value without allocating via `encode`.

---

This project has no affiliation with the Rust project or the `rustdoc` tool.

## Progress

`ai-rustdoc` is a work in progress.

- [x] Parse `rustdoc` JSON outputs
- [ ] Print API info in a clean and informative manner suitable for use by AIs
- [ ] Expose `rustdoc` JSON -> markdown conversion as a CLI tool
- [ ] Distribute as a cargo [custom command] `cargo ai-rustdoc [<crate_name>]`
  to generate AI docs for a specific crate, all crates in the workspace, or all
  crates and all dependencies in the workspace. Rename to `cargo-ai-rustdoc`?

[custom command]: https://doc.rust-lang.org/book/ch14-05-extending-cargo.html

## License

MIT
