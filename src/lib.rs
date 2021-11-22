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
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

#[wasm_bindgen]
pub fn compile(text: &str) -> String {
    log(&format!("received input: {:?}", text));
    let compiler = Parser::new_ext(text.trim(), Options::all());
    let mut html = String::new();
    push_html(&mut html, compiler);
    log(&format!("compiled output: {}", html));
    return html;
}

#[wasm_bindgen]
pub fn add(a: usize, b: usize) -> usize {
    return a + b;
}