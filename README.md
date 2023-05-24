# tiny-web-macro

`tiny-web-macro` is a macro library for automatically connecting a web engine to the web project.

## Installation

Add `tiny-web-macro` to your `Cargo.toml` dependencies:

```toml
[dependencies]
tiny-web-macro = "0.1"
```

## Usage

### For create engine

You need to make a closure with the `tiny_web_macro::addfn!()` macro and pass it as a parameter to the `tiny_web::run` function.

```rust
/// Actions (web controllers)
pub mod app;

fn main() {
    tiny_web::run(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_DESCRIPTION"),
        || { tiny_web_macro::addfn!(); },
    );
}
```

### For add mod

To connect to Actions (web controllers) in the tiny-web project, you need to create the file `./app/mod.rs` in the project root directory. Insert this macro

```rust
tiny_web_macro::addmod!();
```

into the file `./app/mod.rs`. In addition, it is necessary to add in `main.rs` the use of this module. See the example above.

## License

This project is licensed under the MIT License - see the LICENSE file for details.