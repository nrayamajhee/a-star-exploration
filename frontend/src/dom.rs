#![allow(dead_code)]
use crate::RcCell;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use wasm_bindgen::{closure::Closure, JsCast};
use wasm_bindgen_futures::{future_to_promise, JsFuture};
use web_sys::{
    Document, Element, Event, EventTarget, HtmlElement, HtmlHeadElement, HtmlInputElement,
    HtmlStyleElement, NodeList, Request, RequestInit, Response, Window,
};

pub fn query_els(selector: &str) -> NodeList {
    document()
        .query_selector_all(selector)
        .unwrap_or_else(|_| panic!("No element matches selector: {}", selector))
}

pub fn html_el_from(el: Element) -> HtmlElement {
    el.dyn_into::<HtmlElement>()
        .expect("Can't cast the html element as elment")
}

pub fn for_each<F: 'static + Fn(Element)>(node_list: &NodeList, clo: F) {
    for i in 0..node_list.length() {
        clo(node_list.get(i).unwrap().dyn_into::<Element>().unwrap());
    }
}

pub fn try_query_el(selector: &str) -> Option<Element> {
    document().query_selector(selector).unwrap_or_else(|err| {
        panic!(
            "There was an error running query selector: {}\n{:?}",
            selector, err
        )
    })
}

pub fn query_el(selector: &str) -> Element {
    try_query_el(selector).unwrap_or_else(|| panic!("No element matches selector: {}", selector))
}

pub fn get_el(id: &str) -> Element {
    document()
        .get_element_by_id(id)
        .unwrap_or_else(|| panic!("Element with id {} not found in document!", id))
}

pub fn get_value(id: &str) -> String {
    get_el(id)
        .dyn_into::<HtmlInputElement>()
        .unwrap_or_else(|e| panic!("Element with id {} not an input element!:\n{:#?}", id, e))
        .value()
}

pub fn event_as_input(event: &Event) -> HtmlInputElement {
    event
        .target()
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap()
}

pub enum HtmlPosition {
    Before,
    Start,
    End,
    After,
}

impl HtmlPosition {
    fn as_str(&self) -> &'static str {
        match self {
            HtmlPosition::Before => "beforebegin",
            HtmlPosition::Start => "afterbegin",
            HtmlPosition::End => "beforeend",
            HtmlPosition::After => "afterend",
        }
    }
}

pub fn insert_html_at(element: &Element, html: &str, location: HtmlPosition) {
    element
        .insert_adjacent_html(location.as_str(), html)
        .unwrap();
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

#[derive(PartialEq, Debug, Clone)]
pub enum FetchMethod {
    Get,
    Post(JsValue),
}

impl FetchMethod {
    pub fn post<S: Serialize>(body: &S) -> Self {
        FetchMethod::Post(serde_json::to_string(body).unwrap().into())
    }
}

pub async fn fetch(
    url: String,
    method: FetchMethod,
    content_type: Option<&str>,
) -> Result<JsValue, JsValue> {
    let mut opts = RequestInit::new();
    if let FetchMethod::Post(body) = method {
        opts.method("POST");
        opts.body(Some(&body));
    } else {
        opts.method("GET");
    }
    let request = Request::new_with_str_and_init(&url, &opts)?;
    if let Some(content_type) = content_type {
        request.headers().set("Content-Type", content_type)?;
    }
    let resp_value = JsFuture::from(window().fetch_with_request(&request)).await?;
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();
    let json = JsFuture::from(resp.json()?).await?;
    Ok(json)
}

pub async fn fetch_json<T: for<'a> Deserialize<'a>>(url: String, method: FetchMethod) -> T {
    let json = fetch(url, method, Some("application/json"))
        .await
        .unwrap_or_else(|err| panic!("Couldn't fetch response:\n{:?}", err));
    json.into_serde::<T>()
        .expect("Couldn't serialize API json into response!")
}

pub fn try_fetch_then<F: 'static + Fn(JsValue)>(url: String, method: FetchMethod, closure: F) {
    let resolve =
        Closure::wrap(Box::new(move |json: JsValue| closure(json)) as Box<dyn FnMut(JsValue)>);
    let m = method.clone();
    let reject = Closure::wrap(Box::new(move |err| {
        crate::log!("Failed to", m, "data!\nError:\n", err);
    }) as Box<dyn FnMut(JsValue)>);
    let _ = future_to_promise(fetch(url, method, Some("application/json")))
        .then(&resolve)
        .catch(&reject);
    resolve.forget();
    reject.forget();
}

pub fn get_target(e: &Event) -> EventTarget {
    e.target().expect("No target element for the event!")
}

pub fn get_target_el(e: &Event) -> Element {
    get_target(&e)
        .dyn_into::<Element>()
        .expect("Can't cast as Element!")
}

pub fn fetch_then<T: for<'a> Deserialize<'a>, F: 'static + Fn(T)>(
    url: String,
    method: FetchMethod,
    closure: F,
) {
    try_fetch_then(url, method, move |json: JsValue| {
        closure(json.into_serde::<T>().unwrap_or_else(|err| {
            panic!(
                "Couldn't deserialize response into the given type!\n{:?}",
                err
            )
        }));
    });
}
