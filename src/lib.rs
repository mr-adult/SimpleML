use std::borrow::Cow;
use tree_iterators_rs::prelude::{OwnedTreeNode, TreeNode};
use whitespacesv::{ColumnAlignment, WSVError, WSVWriter};

/// Parses the Simple Markup Language text into a tree of SMLElements.
/// For details about how to use TreeNode, see [tree_iterators_rs](https://crates.io/crates/tree_iterators_rs)
/// and the documentation related to that crate.
pub fn parse(source_text: &str) -> Result<TreeNode<SMLElement<Cow<'_, str>>>, ParseError> {
    let wsv_result = whitespacesv::parse(source_text);
    let wsv = match wsv_result {
        Err(err) => return Err(ParseError::WSV(err)),
        Ok(wsv) => wsv,
    };

    let end_keyword = match wsv.iter().rev().find(|line| !line.is_empty()) {
        None => {
            return Err(ParseError::SML(SMLError {
                err_type: SMLErrorType::EndKeywordNotDetected,
                line_num: wsv.len(),
            }))
        }
        Some(last_line) => match last_line.get(0).unwrap() {
            None => None,
            Some(val) => Some(val.to_lowercase()),
        },
    };

    let mut lines_iter = wsv.into_iter().enumerate();
    let root_element_name;
    loop {
        let first_line = lines_iter.next();
        match first_line {
            None => panic!("Found an empty file, but this should've returned an SMLError::EndKeywordNotDetected"), 
            Some((line_num, mut first_line)) => {
                if first_line.is_empty() { continue; }
                if first_line.len() > 1 { return Err(ParseError::SML(SMLError {
                    err_type: SMLErrorType::InvalidRootElementStart,
                    line_num,
                })) }
                match std::mem::take(first_line.get_mut(0).unwrap()) {
                    None => return Err(ParseError::SML(SMLError {
                        err_type: SMLErrorType::NullValueAsElementName,
                        line_num,
                    })),
                    Some(root) => {
                        root_element_name = root;
                        break;
                    }
                }
            }
        }
    }

    let root = TreeNode {
        value: SMLElement {
            name: root_element_name,
            attributes: Vec::with_capacity(0),
        },
        children: None,
    };
    let mut nodes_being_built = vec![root];
    let mut result = None;

    for (line_num, mut line) in lines_iter {
        if line.is_empty() {
            continue;
        }
        if line.len() == 1 {
            let val;
            let val_lowercase;
            match line.get_mut(0) {
                None => {
                    if end_keyword.is_some() {
                        return Err(ParseError::SML(SMLError {
                            err_type: SMLErrorType::NullValueAsElementName,
                            line_num,
                        }));
                    }
                    val = None;
                    val_lowercase = None;
                }
                Some(inner_val) => match inner_val {
                    None => {
                        val = None;
                        val_lowercase = None;
                    }
                    Some(innermost_val) => {
                        val_lowercase = Some(innermost_val.to_lowercase());
                        val = Some(std::mem::take(innermost_val));
                    }
                },
            };

            if val_lowercase == end_keyword {
                match nodes_being_built.pop() {
                    None => {
                        return Err(ParseError::SML(SMLError {
                            err_type: SMLErrorType::OnlyOneRootElementAllowed,
                            line_num,
                        }))
                    }
                    Some(top) => {
                        let nodes_being_built_len = nodes_being_built.len();
                        if nodes_being_built_len == 0 {
                            if result.is_some() {
                                return Err(ParseError::SML(SMLError {
                                    err_type: SMLErrorType::OnlyOneRootElementAllowed,
                                    line_num,
                                }));
                            } else {
                                result = Some(top);
                                continue;
                            }
                        }

                        let new_top = nodes_being_built
                            .get_mut(nodes_being_built_len - 1)
                            .unwrap();

                        match &mut new_top.children {
                            None => new_top.children = Some(vec![top]),
                            Some(children) => children.push(top),
                        }
                    }
                }
            } else {
                nodes_being_built.push(TreeNode {
                    value: SMLElement {
                        name: val.expect("BUG: Null element names are prohibited."),
                        attributes: Vec::with_capacity(0),
                    },
                    children: None,
                });
            }
        } else {
            let mut values = line.into_iter();
            let name = match values.next().unwrap() {
                None => {
                    return Err(ParseError::SML(SMLError {
                        err_type: SMLErrorType::NullValueAsAttributeName,
                        line_num,
                    }))
                }
                Some(val) => val,
            };

            let attr_values = values.collect::<Vec<_>>();
            let nodes_being_built_len = nodes_being_built.len();
            if nodes_being_built_len == 0 {
                return Err(ParseError::SML(SMLError {
                    err_type: SMLErrorType::OnlyOneRootElementAllowed,
                    line_num,
                }));
            }

            let current = nodes_being_built
                .get_mut(nodes_being_built_len - 1)
                .unwrap();
            current.value.attributes.push(SMLAttribute {
                name,
                values: attr_values,
            });
        }
    }

    return Ok(result.unwrap());
}

pub struct SMLWriter<StrAsRef>
where
    StrAsRef: AsRef<str> + From<&'static str> + ToString,
{
    indent_str: String,
    end_keyword: Option<String>,
    column_alignment: ColumnAlignment,
    values: TreeNode<SMLElement<StrAsRef>>,
}

impl<StrAsRef> SMLWriter<StrAsRef>
where
    StrAsRef: AsRef<str> + From<&'static str> + ToString,
{
    pub fn new(values: TreeNode<SMLElement<StrAsRef>>) -> Self {
        Self {
            values,
            indent_str: "    ".to_string(), // default to 4 spaces
            end_keyword: None, // Use minified as the default
            column_alignment: ColumnAlignment::default(),
        }
    }

    /// Sets the indentation string to be used in the output.
    /// If the passed in str contains any non-whitespace characters,
    /// this call will fail and return None.
    pub fn indent_with(mut self, str: &str) -> Option<Self> {
        if str.chars().any(|ch| !Self::is_whitespace(ch)) {
            return None;
        }
        self.indent_str = str.to_string();
        return Some(self);
    }

    /// Sets the end keyword to be used in the output.
    /// If the passed in string is the empty string "",
    /// '-' will be used instead.
    pub fn with_end_keyword(mut self, str: Option<&str>) -> Self {
        match str {
            None | Some("") => {
                self.end_keyword = None;
                return self;
            }
            Some(str) => {
                debug_assert!(!str.is_empty());
                let needs_quotes = str
                    .chars()
                    .any(|ch| ch == '"' || ch == '#' || ch == '\n' || Self::is_whitespace(ch))
                    || str == "-";
                
                if !needs_quotes {
                    self.end_keyword = Some(str.to_string());
                } else {
                    let mut result = String::new();
                    result.push('"');
                    for ch in str.chars() {
                        match ch {
                            '"' => result.push_str("\"\""),
                            '\n' => result.push_str("\"/\""),
                            ch => result.push(ch),
                        }
                    }
                    result.push('"');
                    self.end_keyword = Some(result);
                }
                return self;
            }
        }
    }

    /// Sets the column alignment of the attributes' generated WSV.
    /// The element alignment will be unaffected, but all attributes
    /// and their values will be aligned this way.
    pub fn align_columns(mut self, alignment: ColumnAlignment) -> Self {
        self.column_alignment = alignment;
        return self;
    }

    /// Writes the values in this SMLWriter out to a String. This operation
    /// can fail if any of the values would result in an SML attribute or
    /// element where the name is the same as the "End" keyword. If that
    /// happens, you as the caller will receive an Err() variant of Result.
    pub fn to_string(self) -> Result<String, SMLWriterError> {
        let mut result = String::new();
        Self::to_string_helper(
            self.values,
            0,
            &self.column_alignment,
            &self.indent_str,
            self.end_keyword.as_ref(),
            &mut result,
        )?;
        return Ok(result);
    }

    fn to_string_helper(
        value: TreeNode<SMLElement<StrAsRef>>,
        depth: usize,
        alignment: &ColumnAlignment,
        indent_str: &str,
        end_keyword: Option<&String>,
        buf: &mut String,
    ) -> Result<(), SMLWriterError> {
        let (value, children) = value.get_value_and_children();
        if let Some(end_keyword) = end_keyword {
            if value.name.as_ref() == end_keyword {
                return Err(SMLWriterError::ElementHasEndKeywordName);
            }
        }

        for _ in 0..depth {
            buf.push_str(indent_str);
        }
        buf.push_str(value.name.as_ref());

        if !value.attributes.is_empty() {
            buf.push('\n');
            for _ in 0..depth + 1 {
                buf.push_str(indent_str);
            }
        }

        if let Some(end_keyword) = end_keyword {
            for attribute in value.attributes.iter() {
                if attribute.name.as_ref() == end_keyword {
                    return Err(SMLWriterError::AttributeHasEndKeywordName);
                }
            }
        }

        let values_for_writer = value
            .attributes
            .into_iter()
            .map(|attr| std::iter::once(Some(attr.name)).chain(attr.values.into_iter()));

        match alignment {
            ColumnAlignment::Packed => {
                for ch in WSVWriter::new(values_for_writer) {
                    buf.push(ch);
                    if ch == '\n' {
                        for _ in 0..depth + 1 {
                            buf.push_str(indent_str);
                        }
                    }
                }
            }
            ColumnAlignment::Left | ColumnAlignment::Right => {
                for ch in WSVWriter::new(values_for_writer)
                    .align_columns(match alignment {
                        ColumnAlignment::Left => ColumnAlignment::Left,
                        ColumnAlignment::Right => ColumnAlignment::Right,
                        ColumnAlignment::Packed => {
                            panic!("BUG: ColumnAlignment::Packed shouldn't go down this path")
                        }
                    })
                    .to_string()
                    .chars()
                {
                    buf.push(ch);
                    if ch == '\n' {
                        for _ in 0..depth + 1 {
                            buf.push_str(indent_str);
                        }
                    }
                }
            }
        }

        for child in children.into_iter().flat_map(|opt| opt) {
            buf.push('\n');
            Self::to_string_helper(child, depth + 1, alignment, indent_str, end_keyword, buf)?;
        }
        buf.push('\n');
        for _ in 0..depth {
            buf.push_str(indent_str);
        }
        match end_keyword {
            None => buf.push('-'),
            Some(end) => buf.push_str(end),
        }

        return Ok(());
    }

    const fn is_whitespace(ch: char) -> bool {
        match ch {
            '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{000D}' | '\u{0020}' | '\u{0085}'
            | '\u{00A0}' | '\u{1680}' | '\u{2000}' | '\u{2001}' | '\u{2002}' | '\u{2003}'
            | '\u{2004}' | '\u{2005}' | '\u{2006}' | '\u{2007}' | '\u{2008}' | '\u{2009}'
            | '\u{200A}' | '\u{2028}' | '\u{2029}' | '\u{202F}' | '\u{205F}' | '\u{3000}' => {
                return true;
            }
            _ => return false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SMLWriterError {
    ElementHasEndKeywordName,
    AttributeHasEndKeywordName,
}

#[derive(Debug, Clone)]
pub enum ParseError {
    WSV(WSVError),
    SML(SMLError),
}

#[derive(Debug, Clone)]
pub struct SMLError {
    err_type: SMLErrorType,
    line_num: usize,
}

impl SMLError {
    pub fn err_type(&self) -> SMLErrorType {
        self.err_type
    }
    pub fn line_num(&self) -> usize {
        self.line_num
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SMLErrorType {
    /// This error can happen for 2 reasons:
    /// 1. the file was empty or only contained comments
    /// 2. the file is invalid SML.
    EndKeywordNotDetected,
    InvalidRootElementStart,
    NullValueAsElementName,
    NullValueAsAttributeName,
    RootNotClosed,
    OnlyOneRootElementAllowed,
}

#[derive(Debug)]
pub struct SMLElement<StrAsRef>
where
    StrAsRef: AsRef<str>,
{
    pub name: StrAsRef,
    pub attributes: Vec<SMLAttribute<StrAsRef>>,
}

#[derive(Debug)]
pub struct SMLAttribute<StrAsRef>
where
    StrAsRef: AsRef<str>,
{
    pub name: StrAsRef,
    pub values: Vec<Option<StrAsRef>>,
}

#[cfg(test)]
mod tests {
    use tree_iterators_rs::prelude::OwnedTreeNode;

    use crate::{SMLAttribute, SMLElement, SMLWriter};

    #[test]
    fn reads_example_correctly() {
        let result = super::parse(include_str!("../example.txt")).unwrap();
        for (i, element) in result.dfs_preorder().enumerate() {
            match i {
                0 => {
                    assert_eq!("Configuration", &element.name);
                    assert_eq!(0, element.attributes.len());
                }
                1 => {
                    assert_eq!("Video", &element.name);
                    assert_eq!(3, element.attributes.len());
                    for (j, attribute) in element.attributes.into_iter().enumerate() {
                        match j {
                            0 => {
                                assert_eq!("Resolution", attribute.name);
                                assert_eq!(2, attribute.values.len());
                                assert_eq!(
                                    "1280",
                                    attribute
                                        .values
                                        .get(0)
                                        .as_ref()
                                        .unwrap()
                                        .as_ref()
                                        .unwrap()
                                        .as_ref()
                                );
                                assert_eq!(
                                    "720",
                                    attribute
                                        .values
                                        .get(1)
                                        .as_ref()
                                        .unwrap()
                                        .as_ref()
                                        .unwrap()
                                        .as_ref()
                                );
                            }
                            1 => {
                                assert_eq!("RefreshRate", attribute.name);
                                assert_eq!(1, attribute.values.len());
                                assert_eq!(
                                    "60",
                                    attribute
                                        .values
                                        .get(0)
                                        .as_ref()
                                        .unwrap()
                                        .as_ref()
                                        .unwrap()
                                        .as_ref()
                                );
                            }
                            2 => {
                                assert_eq!("Fullscreen", attribute.name);
                                assert_eq!(1, attribute.values.len());
                                assert_eq!(
                                    "true",
                                    attribute
                                        .values
                                        .get(0)
                                        .as_ref()
                                        .unwrap()
                                        .as_ref()
                                        .unwrap()
                                        .as_ref()
                                );
                            }
                            _ => panic!("Should only have 3 attributes"),
                        }
                    }
                }
                2 => {
                    assert_eq!("Audio", &element.name);
                    assert_eq!(2, element.attributes.len());
                    for (j, attribute) in element.attributes.into_iter().enumerate() {
                        match j {
                            0 => {
                                assert_eq!("Volume", attribute.name);
                                assert_eq!(1, attribute.values.len());
                                assert_eq!(
                                    "100",
                                    attribute
                                        .values
                                        .get(0)
                                        .as_ref()
                                        .unwrap()
                                        .as_ref()
                                        .unwrap()
                                        .as_ref()
                                );
                            }
                            1 => {
                                assert_eq!("Music", attribute.name);
                                assert_eq!(1, attribute.values.len());
                                assert_eq!(
                                    "80",
                                    attribute
                                        .values
                                        .get(0)
                                        .as_ref()
                                        .unwrap()
                                        .as_ref()
                                        .unwrap()
                                        .as_ref()
                                );
                            }
                            _ => panic!("Should only have 2 values under audio"),
                        }
                    }
                }
                3 => {
                    assert_eq!("Player", element.name);
                    assert_eq!(1, element.attributes.len());
                    let attr = element.attributes.get(0).unwrap();
                    assert_eq!("Name", attr.name);
                    assert_eq!(1, attr.values.len());
                    assert_eq!(
                        "Hero 123",
                        attr.values
                            .get(0)
                            .as_ref()
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .as_ref()
                    );
                }
                _ => panic!("Should only have 4 sub-elements"),
            }
        }
    }

    #[test]
    fn test_write() {
        let input = include_str!("../example.txt");
        println!(
            "{}",
            super::SMLWriter::new(super::parse(input).unwrap())
                .align_columns(whitespacesv::ColumnAlignment::Right)
                .indent_with(" ")
                .unwrap()
                .to_string()
                .unwrap()
        );
    }

    #[test]
    fn readme_example() {
        use tree_iterators_rs::prelude::*;

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
    }

    #[test]
    fn parses_input_with_strange_end_keyword() {
        let input = r#"
        Configuration
                Video
                        Resolution 1280 720
                        RefreshRate   60
                        Fullscreen true
                "End with spaces"
                Audio
                        Volume 100
                        Music  80
                "End with spaces"
                Player
                        Name "Hero 123"
                "End with spaces"
        "End with spaces""#;

        println!("{:?}", super::parse(input).unwrap());
    }

    #[test]
    fn parses_input_with_null_end_keyword() {
        let input = r#"
        Configuration
                Video
                        Resolution 1280 720
                        RefreshRate   60
                        Fullscreen true
                -
                Audio
                        Volume 100
                        Music  80
                -
                Player
                        Name "Hero 123"
                -
        -"#;

        println!("{:?}", super::parse(input).unwrap());
    }
}
