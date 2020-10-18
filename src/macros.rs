#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (crate::log(&format_args!($($t)*).to_string()))
}

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
            crate::log(&msg);
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

