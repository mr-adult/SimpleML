#![feature(proc_macro_span)]
#![doc = include_str!("../README.md")]

use std::borrow::Cow;

use proc_macro::{
    token_stream::IntoIter, Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream,
    TokenTree,
};
use simpleml::{parse, SMLElement};
use tree_iterators_rs::prelude::Tree;

extern crate proc_macro;

static DEBUG: bool = false;

/// Handles parsing and converting the SML into a Rust-based
/// Tree representation for better portability between
/// SML files.
#[proc_macro]
pub fn sml(stream: TokenStream) -> TokenStream {
    let converted_string = reconstruct_source_whitespace(stream.into_iter());
    #[cfg(debug_assertions)]
    if DEBUG {
        #[cfg(debug_assertions)]
        println!("{}", converted_string);
    }
    match parse(&converted_string) {
        Ok(tree) => {
            let rust = convert_sml_to_rust(tree);
            #[cfg(debug_assertions)]
            if DEBUG {
                #[cfg(debug_assertions)]
                println!("{}", rust)
            }
            rust
        }
        Err(err) => {
            #[cfg(debug_assertions)]
            if DEBUG {
                #[cfg(debug_assertions)]
                println!("{}", err);
            }
            panic!("{}", err);
        }
    }
}

fn reconstruct_source_whitespace(stream: IntoIter) -> String {
    let mut result = String::new();
    reconstruct_source_whitespace_internal(stream, &mut result, None);
    result
}

fn reconstruct_source_whitespace_internal(
    stream: IntoIter,
    builder: &mut String,
    mut previous_token_end: Option<(usize, usize)>,
) -> Option<(usize, usize)> {
    let mut end_position = None;
    for token_tree in stream {
        let token_start = token_tree.span().start();
        let token_end = token_tree.span().end();
        end_position = Some((token_end.line(), token_end.column()));

        match previous_token_end {
            None => {
                for _ in 0..token_start.column() - 1 {
                    builder.push(' ');
                }
            }
            Some(end) => {
                // insert a new line to match the token stream
                if end.0 != token_start.line() {
                    for _ in 0..token_start.line() - end.0 {
                        builder.push('\n');
                    }

                    for _ in 0..token_start.column() - 1 {
                        builder.push(' ');
                    }

                    // insert a space to match the token stream
                } else if end.1 != token_start.column() {
                    for _ in 0..token_start.column() - end.1 {
                        builder.push(' ');
                    }
                }
            }
        }

        match token_tree {
            TokenTree::Group(group) => {
                let symbols = match group.delimiter() {
                    Delimiter::Brace => ('{', '}'),
                    Delimiter::Bracket => ('[', ']'),
                    Delimiter::Parenthesis => ('(', ')'),
                    Delimiter::None => (' ', ' '),
                };

                builder.push(symbols.0);
                let span_start = token_start;
                let opening_bracket_end = Some((span_start.line(), span_start.column() + 1));
                let end_of_inner_tokens = reconstruct_source_whitespace_internal(
                    group.stream().into_iter(),
                    builder,
                    opening_bracket_end,
                );

                match end_of_inner_tokens {
                    None => {}
                    Some((line, col)) => {
                        let bracket_end = token_start;
                        if line != bracket_end.line() {
                            builder.push('\n');
                        } else if col != bracket_end.column() - 1 {
                            builder.push(' ');
                        }
                    }
                }

                builder.push(symbols.1);
            }
            TokenTree::Ident(ident) => builder.push_str(&ident.to_string()),
            TokenTree::Punct(punct) => {
                builder.push(punct.as_char());
            }
            TokenTree::Literal(literal) => {
                builder.push_str(&literal.to_string());
            }
        }

        let end_span = token_end;
        previous_token_end = Some((end_span.line(), end_span.column()));
    }

    end_position
}

fn convert_sml_to_rust(tree: Tree<SMLElement<Cow<'_, str>>>) -> TokenStream {
    TokenStream::from_iter([
        TokenTree::Ident(Ident::new("tree_iterators_rs", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("prelude", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("Tree", Span::call_site())),
        TokenTree::Group(Group::new(
            Delimiter::Brace,
            TokenStream::from_iter([
                TokenTree::Ident(Ident::new("value", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("simpleml", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("SMLElement", Span::call_site())),
                TokenTree::Group(Group::new(
                    Delimiter::Brace,
                    TokenStream::from_iter([
                        TokenTree::Ident(Ident::new("name", Span::call_site())),
                        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                        TokenTree::Literal(Literal::string(&tree.value.name)),
                        TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("attributes", Span::call_site())),
                        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("vec", Span::call_site())),
                        TokenTree::Punct(Punct::new('!', Spacing::Alone)),
                        TokenTree::Group(Group::new(
                            Delimiter::Brace,
                            TokenStream::from_iter(tree.value.attributes.into_iter().map(|attr| {
                                TokenStream::from_iter([
                                    TokenTree::Ident(Ident::new("simpleml", Span::call_site())),
                                    TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                                    TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                                    TokenTree::Ident(Ident::new("SMLAttribute", Span::call_site())),
                                    TokenTree::Group(Group::new(
                                        Delimiter::Brace,
                                        TokenStream::from_iter([
                                            TokenTree::Ident(Ident::new("name", Span::call_site())),
                                            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                                            TokenTree::Literal(Literal::string(&attr.name)),
                                            TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                                            TokenTree::Ident(Ident::new(
                                                "values",
                                                Span::call_site(),
                                            )),
                                            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                                            TokenTree::Ident(Ident::new("vec", Span::call_site())),
                                            TokenTree::Punct(Punct::new('!', Spacing::Alone)),
                                            TokenTree::Group(Group::new(
                                                Delimiter::Bracket,
                                                TokenStream::from_iter(
                                                    attr.values.into_iter().flat_map(|value| {
                                                        let mut tokens = Vec::with_capacity(3);
                                                        match value {
                                                            None => tokens.push(TokenTree::Ident(
                                                                Ident::new(
                                                                    "None",
                                                                    Span::call_site(),
                                                                ),
                                                            )),
                                                            Some(str) => {
                                                                tokens.push(TokenTree::Ident(
                                                                    Ident::new(
                                                                        "Some",
                                                                        Span::call_site(),
                                                                    ),
                                                                ));
                                                                tokens.push(TokenTree::Group(
                                                                    Group::new(
                                                                        Delimiter::Parenthesis,
                                                                        TokenStream::from(
                                                                            TokenTree::Literal(
                                                                                Literal::string(
                                                                                    str.as_ref(),
                                                                                ),
                                                                            ),
                                                                        ),
                                                                    ),
                                                                ))
                                                            }
                                                        }

                                                        tokens.push(TokenTree::Punct(Punct::new(
                                                            ',',
                                                            Spacing::Alone,
                                                        )));

                                                        tokens
                                                    }),
                                                ),
                                            )),
                                        ]),
                                    )),
                                    TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                                ])
                            })),
                        )),
                        TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                    ]),
                )),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                TokenTree::Ident(Ident::new("children", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("vec", Span::call_site())),
                TokenTree::Punct(Punct::new('!', Spacing::Alone)),
                TokenTree::Group(Group::new(
                    Delimiter::Bracket,
                    TokenStream::from_iter(tree.children.into_iter().flat_map(|child| {
                        let mut stream = convert_sml_to_rust(child).into_iter().collect::<Vec<_>>();
                        stream.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
                        stream
                    })),
                )),
            ]),
        )),
    ])
}
