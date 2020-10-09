#[macro_export]
macro_rules! log {
    ($($x:expr),*) => {
        {
            let mut msg = String::new();
            use std::any::Any;
            $(
                if let Some(s) = (&$x as &dyn Any).downcast_ref::<&dyn std::fmt::Display>() {
                    msg.push_str(&format!("{} ", s));
                } else {
                    msg.push_str(&format!("{:?} ",$x));
                }
            )*
            web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&msg));
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

