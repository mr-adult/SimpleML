# SimpleML

This crate is a rust implementation of the [Simple Markup Language](https://www.simpleml.com/) specification. This specification is built on top of the [Whitespace Separated Value](https://www.whitespacesv.com/) specification and provides a human-friendly configuration file format.


## 0.3.0 patch notes

Version 0.3.0 did not make many changes. The only changes that impact consumers are the following:
- The dependency on tree_iterators_rs was changed from version 1.1.4 to 1.2.1
- The [parse_owned](https://docs.rs/simpleml/0.2.2/simpleml/fn.parse_owned.html) function was added


## Parsing

This crate only provides a single API for parsing SML files. The [parse](https://docs.rs/simpleml/latest/simpleml/fn.parse.html) function will parse the SML input, returning a Result. The Ok variant is a [TreeNode](https://docs.rs/tree_iterators_rs/latest/tree_iterators_rs/prelude/struct.TreeNode.html) from the [tree_iterators_rs](https://crates.io/crates/tree_iterators_rs) crate representing the elements of the hierarchy. I have chosen to build on top of it to provide you with all of the tree traversal utility methods built there when working with your SML content. I recommend adding `use tree_iterators_rs::prelude::*;` to the files that handle the SML content to pull the traversal methods into scope.

Because SML is an extension on top of WSV, I have also pulled in [whitespacesv](https://crates.io/crates/whitespacesv) as a dependency.


## Writing

In order to write an SML file, use the [SMLWriter](https://docs.rs/simpleml/latest/simpleml/struct.SMLWriter.html) struct. It provides several configuration options, including:
1. any custom end keyword
2. the column alignment of SML attribute WSV tables
3. the indentation string (this must be whitespace)

The list of whitespace characters is defined as follows:
```text
Codepoint 	Name
U+0009 	    Character Tabulation
U+000A 	    Line Feed
U+000B 	    Line Tabulation
U+000C 	    Form Feed
U+000D 	    Carriage Return
U+0020 	    Space
U+0085 	    Next Line
U+00A0 	    No-Break Space
U+1680 	    Ogham Space Mark
U+2000 	    En Quad
U+2001 	    Em Quad
U+2002 	    En Space
U+2003 	    Em Space
U+2004 	    Three-Per-Em Space
U+2005 	    Four-Per-Em Space
U+2006 	    Six-Per-Em Space
U+2007 	    Figure Space
U+2008 	    Punctuation Space
U+2009 	    Thin Space
U+200A 	    Hair Space
U+2028 	    Line Separator
U+2029 	    Paragraph Separator
U+202F 	    Narrow No-Break Space
U+205F 	    Medium Mathematical Space
U+3000 	    Ideographic Space
```

An example of how to use the SMLWriter is as follows:
```rust
use tree_iterators_rs::prelude::*;
use simpleml::{SMLWriter, SMLElement, SMLAttribute};

// Build up our value set
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

// actually write the values
let str = SMLWriter::new(my_sml_values)
    // Setting up a custom end keyword
    .with_end_keyword(Some("my_custom_end_keyword"))
    // Using 8 spaces as the indent string. The default is 4 spaces.
    .indent_with("        ")
    .unwrap()
    // Align the WSV tables to the right.
    .align_columns(whitespacesv::ColumnAlignment::Right)
    .to_string()
    .unwrap();

/// Result:
/// Configuration
///         Video
///                  Resolution 1280 720
///                 RefreshRate   60
///                  Fullscreen true
///         my_custom_end_keyword
///         Audio
///                 Volume 100
///                  Music  80
///         my_custom_end_keyword
///         Player
///                 Name "Hero 123"
///         my_custom_end_keyword
/// my_custom_end_keyword
println!("{}", str);
```