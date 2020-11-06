#![feature(proc_macro_hygiene, async_closure)]
mod macros;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[derive(Clone)]
pub struct RcCell<T>(pub Rc<RefCell<T>>);

impl<T> RcCell<T> {
    pub fn new(inner: T) -> Self {
        Self(Rc::new(RefCell::new(inner)))
    }
    pub fn mutate(&self, value: T) {
        *self.0.borrow_mut() = value;
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn logv(x: &JsValue);
}

use std::ops::Deref;

impl<T> Deref for RcCell<T> {
    type Target = RefCell<T>;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

pub mod dom;
crate::use_mod!(app, renderer, start);

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[doc(hidden)]
#[wasm_bindgen(start)]
pub fn run() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    start::start();
}
