#![allow(dead_code)]
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{
    Document, Element, Event, EventTarget, HtmlElement, HtmlHeadElement, HtmlStyleElement, Window,
};

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
