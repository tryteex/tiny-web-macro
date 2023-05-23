# tiny-web-macro

`tiny-web-macro` is a macro library for automatically connecting the web engine for the `tiny-web` project.

## Installation

Add `tiny-web-macro` to your `Cargo.toml` dependencies:

```toml
[dependencies]
tiny-web-macro = "0.1.0"
```

## Usage

### For add mod

To connect to Actions (web controllers) in the tiny-web project, you need to create the file `./app/mod.rs` in the project root directory. Insert this macro

```rust
tiny_web_macro::addmod!();
```

into the file `./app/mod.rs`.

### For create engine

In this case, you don't need to do anything, because the project `tiny-web` already contains the necessary macro.

## License

This project is licensed under the MIT License - see the LICENSE file for details.