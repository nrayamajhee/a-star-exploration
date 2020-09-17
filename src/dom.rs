#![allow(dead_code)]
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{
    Document, Element, Event, EventTarget, HtmlElement, HtmlHeadElement, HtmlStyleElement, Window,
};

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

use std::cell::RefCell;
use std::rc::Rc;

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

use std::ops::Deref;

impl<T> Deref for RcCell<T> {
    type Target = RefCell<T>;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

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
