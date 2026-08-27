#![allow(non_snake_case)]

use serde::Deserialize;
use std::{fs, sync::LazyLock};

pub mod page;
pub mod routes;
pub mod css;

pub mod pre {
    pub use crate::{page, fatal, CFG, err_fmt, un, re};
}

#[macro_export]
macro_rules! fatal {
    ($($x:tt),* $(,)*) => {{
        eprintln!($($x),*);
        std::process::exit(-1);
    }};
}

#[macro_export]
macro_rules! re {
    ($r:expr) => {{
        $r.map_err(|e| format!("{e}"))
    }};
}

#[macro_export]
macro_rules! un {
    ($r:expr) => {{
        $crate::re!($r)?
    }};
}

#[macro_export]
macro_rules! err_fmt {
    ($($x:tt)*) => {{
        Err(format!($($x)*))
    }};
}

pub type R<T> = Result<T, String>;

#[derive(Deserialize)]
pub struct Cfg {
    pub title: String,
    pub ip: usize,
    pub port: String,
}

static CFG_PATHS: &[&str] = &[
    "etc/cfg.toml",
    "etc/config.toml",
];

impl Cfg {
    pub fn new() -> R<Self> {
        let mut n = String::new();
        let mut f = String::new();

        for p in CFG_PATHS.into_iter() {
            if un!(fs::exists(p)) {
                n = p.to_string();
                f = un!(fs::read_to_string(p));
            }
        }

        if n.is_empty() {
            return err_fmt!("cannot find config file out of {CFG_PATHS:#?}");
        }

        un!(toml::from_str(&f))
    }
}

pub static CFG: LazyLock<Cfg> = LazyLock::new(|| match Cfg::new() {
    Ok(x) => x,
    Err(e) => fatal!("{e}"),
});
