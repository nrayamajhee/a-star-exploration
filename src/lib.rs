#![feature(proc_macro_hygiene)]

mod macros;
use futures_channel::oneshot::{self, Receiver};
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

crate::use_mod!(app, dom, grid, renderer, start, node, pool,);

use pool::WorkerPool;
use rayon::ThreadPool;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[doc(hidden)]
#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    //let (otx, orx) = oneshot::channel();
    let concurrency = window().navigator().hardware_concurrency() as usize;
    let worker_pool = WorkerPool::new(concurrency).unwrap();
    let mut data = [1., 2., 3., 4., 5., 6.];
    for chunk in data.chunks(3) {
        let (tx, rx) = std::sync::mpsc::channel();
        for each in chunk.iter() {
            worker_pool.run(|| {
                tx.send(each * 2.);
            });
        }
        let mut res: Vec<_> = rx.iter().collect();
        crate::log!(res);
        //otx.send(res);
    }
    //let data = async move { orx.await.unwrap() };
    //start::start();
    Ok(())
}
