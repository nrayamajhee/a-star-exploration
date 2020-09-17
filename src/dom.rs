#![allow(dead_code)]
use crate::RcCell;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{
    Document, Element, Event, EventTarget, HtmlElement, HtmlHeadElement, HtmlStyleElement, Window,
};

pub fn get_el(id: &str) -> Element {
    document().get_element_by_id(id).unwrap()
}

pub fn insert_html_at(element: &Element, html: &str, location: &str) {
    element.insert_adjacent_html(html, location).unwrap();
}

pub fn window() -> Window {
    web_sys::window().unwrap()
}

pub fn document() -> Document {
    window().document().unwrap()
}

pub fn body() -> HtmlElement {
    document().body().unwrap()
}

pub fn head() -> HtmlHeadElement {
    document().head().unwrap()
}

pub fn create_el(name: &str) -> Element {
    document().create_element(name).unwrap()
}

pub fn add_style(style: &str) {
    let style_el = create_el("style").dyn_into::<HtmlStyleElement>().unwrap();
    style_el.set_type("text/css");
    style_el.set_inner_html(style);
    head().append_child(&style_el).unwrap();
}

pub fn add_event<F>(el: &EventTarget, type_: &str, closure: F)
where
    F: FnMut(Event) + 'static,
{
    let closure = Closure::wrap(Box::new(closure) as Box<dyn FnMut(_)>);
    el.add_event_listener_with_callback(type_, closure.as_ref().unchecked_ref())
        .unwrap();
    closure.forget();
}

pub fn add_event_mut<'a, T, F>(el: &EventTarget, type_: &str, event: &RcCell<T>, closure: F)
where
    F: Fn(&mut T, Event) + 'static,
    T: Clone + 'static,
{
    let eve = event.clone();
    add_event(&el, type_, move |e| {
        closure(&mut *eve.borrow_mut(), e);
    });
}

pub fn now() -> f64 {
    window()
        .performance()
        .expect("Performance should be available")
        .now()
}

pub fn request_animation_frame(f: &Closure<dyn FnMut()>) -> i32 {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK")
}

pub fn set_timeout<H>(callback: H, timeout: i32)
where
    H: 'static + Fn(),
{
    let cl = Closure::wrap(Box::new(callback) as Box<dyn Fn()>);
    window()
        .set_timeout_with_callback_and_timeout_and_arguments_0(cl.as_ref().unchecked_ref(), timeout)
        .unwrap();
    cl.forget();
}

use std::cell::RefCell;
use std::rc::Rc;

pub fn loop_animation_frame<F>(mut closure: F, fps: Option<f64>)
where
    F: 'static + FnMut(f64),
{
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    let t = Rc::new(RefCell::new(0.));
    let then = t.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let mut then = then.borrow_mut();
        let delta = now() - *then;
        *then = now();
        closure(delta);
        let h = f.clone();
        let next_frame = move || {
            request_animation_frame(h.borrow().as_ref().unwrap());
        };
        if let Some(fps) = fps {
            set_timeout(next_frame, ((1000. / fps) - delta) as i32);
        } else {
            next_frame();
        };
    }) as Box<dyn FnMut()>));
    *t.borrow_mut() = now();
    request_animation_frame(g.borrow().as_ref().unwrap());
}

#[macro_export]
macro_rules! log {
    ($($x:expr) *) => {
        {
            let document = crate::dom::document();
            let console_el = document.get_element_by_id("console");
            let mut msg = String::new();
            use std::any::Any;
            $(
                if let Some(s) = (&$x as &dyn Any).downcast_ref::<&str>() {
                    msg.push_str(&format!("{} ", s));
                } else if let Some(s) = (&$x as &dyn Any).downcast_ref::<&dyn std::fmt::Display>() {
                    msg.push_str(&format!("{} ", s));
                } else {
                    msg.push_str(&format!("{:?} ",$x));
                }
            )*
            if let Some(_) = console_el {
                web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&msg));
                use crate::dom::{insert_html_at, get_el};
                insert_html_at(&get_el("logs"),
                        &format!("<div><i class='material-icons-outlined'>info</i><pre>{}</pre></div>", msg),
                        "afterbegin");
            } else {
                web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&msg));
            }
        }
    };
}
