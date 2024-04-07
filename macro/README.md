# SimpleML Macro

This crate is an extension of [simpleml](https://crates.io/crates/simpleml). It creates an "sml" macro to allow you to declare SML-based config within your code. In order to use this macro, add the following to your Cargo.toml.

NOTE: This macro requires the nightly compiler for the [proc_macro_span](https://github.com/rust-lang/rust/issues/54725) feature.

```toml
[dependencies]
tree_iterators_rs = "1.2.1"
simpleml = "1.0.0"
simpleml_macro = "1.0.0"
```

Once you have these dependencies in place, simply add a using for the macro, and use it in your code. As an example, let's convert the following in-line declaration of the SML data structure over to using the macro. This example comes from the README.md file of [simpleml](https://crates.io/crates/simpleml).

### Before
```rust
use tree_iterators_rs::prelude::*;
use simpleml::{SMLWriter, SMLElement, SMLAttribute};

let my_sml_values = TreeNode {
    value: SMLElement {
        name: "Configuration",
        attributes: Vec::with_capacity(0),
    },
    children: Some(vec![
        TreeNode {
            value: SMLElement {
                name: "Video",
                attributes: vec![
                    SMLAttribute {
                        name: "Resolution",
                        values: vec![Some("1280"), Some("720")],
                    },
                    SMLAttribute {
                        name: "RefreshRate",
                        values: vec![Some("60")],
                    },
                    SMLAttribute {
                        name: "Fullscreen",
                        values: vec![Some("true")],
                    },
                ],
            },
            children: None,
        },
        TreeNode {
            value: SMLElement {
                name: "Audio",
                attributes: vec![
                    SMLAttribute {
                        name: "Volume",
                        values: vec![Some("100")],
                    },
                    SMLAttribute {
                        name: "Music",
                        values: vec![Some("80")],
                    },
                ],
            },
            children: None,
        },
        TreeNode {
            value: SMLElement {
                name: "Player",
                attributes: vec![SMLAttribute {
                    name: "Name",
                    values: vec![Some("Hero 123")],
                }],
            },
            children: None,
        },
    ]),
};
```

### After
```rust
use simpleml_macro::sml;

let my_sml_values = sml!{
    Configuration
        Video
            Resolution  1280 720
            RefreshRate 60
            Fullscreen  true
        my_custom_end_keyword
        Audio
            Volume 100
            Music  80
        my_custom_end_keyword
        Player
            Name "Hero 123"
        my_custom_end_keyword
    my_custom_end_keyword
};
```