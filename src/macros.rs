#[macro_export]
macro_rules! log {
    ($($x:expr),*) => {
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

#[macro_export]
macro_rules! use_mod {
    ($($mod:ident),+,$(,)?) => {
        $(
            mod $mod;
            #[doc(inine)]
            pub use $mod::*;
        )*
    }
}

