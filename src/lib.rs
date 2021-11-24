mod utils;

use wasm_bindgen::prelude::*;
use pulldown_cmark::{Options, Parser, html::push_html};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log(msg: &str);
}

#[wasm_bindgen]
pub fn compile(text: &str) -> String {
    log(text);
    let compiler = Parser::new_ext(text.trim(), Options::all());
    let mut html = String::new();
    push_html(&mut html, compiler);
    log(html);
    return html;
}