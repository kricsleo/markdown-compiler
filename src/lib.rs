use wasm_bindgen::prelude::*;
use syntect::parsing::{SyntaxSet};
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::util::LinesWithEndings;
use lazy_static::lazy_static;
use pulldown_cmark::{Options, Parser, html::push_html, Event, Tag, CodeBlockKind, CowStr};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

lazy_static! {
    static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
}

fn compile_code(str: &String, language: &String) -> String {
    match SYNTAX_SET.find_syntax_by_extension(language) {
        Some(syntax) => {
            let mut generator = ClassedHTMLGenerator::new_with_class_style(
                &syntax,
                &SYNTAX_SET,
                ClassStyle::Spaced
            );
            for line in LinesWithEndings::from(str) {
                generator.parse_html_for_line_which_includes_newline(line).unwrap();
            }
            let html = generator.finalize();
            return html;
        },
        None => {
            return String::from(str);
        }
    };
}

#[wasm_bindgen]
pub fn compile(text: &str) -> String {
    let mut html = String::new();
    let mut language = String::new();
    let html_parser = Parser::new_ext(text.trim(), Options::all())
        .map(|event| match event {
            Event::Text(t) => {
                return if language.is_empty() {
                    Event::Text(t)
                } else {
                    let c = CowStr::Boxed(Box::from(compile_code(&t.as_ref().to_string(), &language)));
                    Event::Html(c)
                };
            },
            Event::Start(Tag::CodeBlock(t)) => {
                match &t {
                    CodeBlockKind::Fenced(lang) => {
                        language = lang.clone().split(' ').next().unwrap().to_string();
                    }
                    _ => ()
                }
                return Event::Start(Tag::CodeBlock(t));
            },
            Event::End(Tag::CodeBlock(t)) => {
                language = String::new();
                return Event::End(Tag::CodeBlock(t));
            },
            _ => event
        });
    push_html(&mut html, html_parser);
    return html;
}