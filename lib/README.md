# SimpleML

This crate is a rust implementation of the
[Simple Markup Language](https://www.simpleml.com/) specification. This
specification is built on top of the
[Whitespace Separated Value](https://dev.stenway.com/WSV/Specification.html)
specification and provides a human-friendly configuration file format.

## Patch Notes

### 3.0.0

Updating the dependency on
[tree_iterators_rs](https://crates.io/crates/tree_iterators_rs) to version 3.2
to get new features.

### 2.0.0

Updating the dependency on
[tree_iterators_rs](https://crates.io/crates/tree_iterators_rs) to version 2.0
for better performance. This is a breaking change and all consumers will need to
update their code to get these performance benefits. If performance is not a
problem for you, there is no reason to switch.

### 1.0.1

Updating the dependency on [whitespacesv](https://crates.io/crates/whitespacesv)
from 1.0.1 to 1.0.2, which
[fixes a panic when writing jagged arrays](https://github.com/mr-adult/WhitespaceSV/issues/1).
Since SML is built on top of WSV, this impacted the SMLWriter as well.

### 1.0.0

This crate is 1.0 now! No breaking changes were introduced in this version, but
it fixes a bug where malformed SML like the following would cause a panic:

```simpleml
Root
    InnerElement
    # No closing tag
End
```

## Parsing

This crate only provides a two APIs for parsing SML files. The
[parse](https://docs.rs/simpleml/latest/simpleml/fn.parse.html) and
[parse_owned](https://docs.rs/simpleml/latest/simpleml/fn.parse_owned.html)
functions will parse the SML input, returning a Result. The Ok variant is a
[Tree](https://docs.rs/tree_iterators_rs/3.5.0/tree_iterators_rs/prelude/struct.Tree.html)
from the [tree_iterators_rs](https://crates.io/crates/tree_iterators_rs) crate
representing the elements of the hierarchy. I have chosen to build on top of it
to provide you with all of the tree traversal utility methods built there when
working with your SML content. I recommend adding
`use tree_iterators_rs::prelude::*;` to the files that handle the SML content to
pull the traversal methods into scope. The only difference between the two APIs
is that [parse](https://docs.rs/simpleml/latest/simpleml/fn.parse.html) uses
Cow<'_, str>'s to represent the SML nodes, and
[parse_owned](https://docs.rs/simpleml/latest/simpleml/fn.parse_owned.html) uses
Strings to represent the SML nodes. If you don't want to deal with lifetimes,
use [parse_owned](https://docs.rs/simpleml/latest/simpleml/fn.parse_owned.html).

Because SML is an extension on top of WSV, I have also pulled in
[whitespacesv](https://crates.io/crates/whitespacesv) as a dependency.

## In-line Declaration

If you plan to include any SimpleML in your rust code or build system, consider
using [simpleml_macro](https://crates.io/crates/simpleml_macro) to maintain
everything in valid SML format. Note that this does not play well with
rust-analyzer and may cause phantom errors to be shown which still compile
successfully.

## Writing

In order to write an SML file, use the
[SMLWriter](https://docs.rs/simpleml/latest/simpleml/struct.SMLWriter.html)
struct. It provides several configuration options, including:

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
let my_sml_values = Tree {
    value: SMLElement {
        name: "Configuration",
        attributes: Vec::with_capacity(0),
    },
    children: vec![
        Tree {
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
            children: Vec::new(),
        },
        Tree {
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
            children: Vec::new(),
        },
        Tree {
            value: SMLElement {
                name: "Player",
                attributes: vec![SMLAttribute {
                    name: "Name",
                    values: vec![Some("Hero 123")],
                }],
            },
            children: Vec::new(),
        },
    ],
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
