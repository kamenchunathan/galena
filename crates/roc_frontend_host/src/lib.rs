mod roc;

use std::sync::{Arc, LazyLock, Mutex, OnceLock};

use roc::call_roc_view;
use wasm_bindgen::prelude::*;
use web_sys;
use wee_alloc::WeeAlloc;

// use roc::call_roc_init;

#[global_allocator]
static ALLOC: WeeAlloc<'_> = WeeAlloc::INIT;

// static MODEL: OnceLock<roc::Model> = OnceLock::new();
// static CAPTURES: OnceLock<roc::Model> = OnceLock::new();
static MODEL: LazyLock<Arc<Mutex<roc::Model>>> = LazyLock::new(|| {
    let model = roc::call_roc_init();
    Arc::new(Mutex::new(model))
});

#[wasm_bindgen]
pub fn run() {
    // if let Ok(model) = MODEL.lock() {
    //     call_roc_view((*model).clone());
    // }

    let window = web_sys::window().unwrap();
    let document = window.document().expect("could not get document");
    let body = document.body().expect("Could not get document body");

    let root_element = document
        .create_element("div")
        .expect("Could not create element");
    root_element.set_text_content(Some("Bingo"));
    body.append_child(&root_element)
        .expect("could not append to body");

    // if let Some(model) = MODEL.get().cloned() {
    //     let ViewResult {
    //         model,
    //         captures,
    //         view,
    //     } = call_roc_view(model);
    //
    //     MODEL.set(model);
    //     CAPTURES.set(captures);
    // }
}
