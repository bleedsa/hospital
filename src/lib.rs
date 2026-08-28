#![allow(non_snake_case)]

use serde::Deserialize;
use axum::response::Html;

use std::{fs, sync::LazyLock};

pub mod tags;
pub mod css;
pub mod db;
pub mod page;
pub mod passwd;
pub mod routes;

pub mod pre {
    pub use crate::{CFG, R, H, err_fmt, fatal, int2bool, page, re, un};
}

/** `panic!()` but make it not ugly. */
#[macro_export]
macro_rules! fatal {
    ($($x:tt),* $(,)*) => {{
        eprintln!($($x),*);
        std::process::exit(-1);
    }};
}

/** re-wrap any `Result` into an `R<T>` */
#[macro_export]
macro_rules! re {
    ($r:expr) => {{ $r.map_err(|e| format!("{e}")) }};
    ($r:expr, $($g:tt)*) => {{
        $r.map_err(|e| page! {
            ("error: {e}"),
            r#"
            <h1>error: {e}</h1>
            <p><a href="{goto}">go back</a></p>
            "#,
            goto = format!($($g)*),
        })
    }};
}

/** unwrap any `Result` into a `T` */
#[macro_export]
macro_rules! un {
    ($r:expr) => {{ $crate::re!($r)? }};
    ($r:expr, $($g:tt)*) => {{ $crate::re!($r, $($g)*)? }};
}

/** make an `Err(format!($($x)*))`. */
#[macro_export]
macro_rules! err_fmt {
    ($($x:tt)*) => {{
        Err(format!($($x)*))
    }};
}

/** convert integers into booleans. */
#[macro_export]
macro_rules! int2bool {
    ($($x:expr),* $(,)*) => {{
        ($(
            if $x != 0 {
                true
            } else {
                false
            }
        ),*)
    }};
}

/** a basic result type */
pub type R<T> = Result<T, String>;

/** an html result type for routes */
pub type H<T> = Result<T, Html<String>>;

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Server {
    pub ip: String,
    pub port: u16,
    pub db: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Site {
    pub title: String,
}

impl Default for Site {
    fn default() -> Self {
        Self {
            title: "badboy hospital".to_string(),
        }
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Cfg {
    pub site: Option<Site>,
    pub server: Server,
}

static CFG_PATHS: &[&str] = &[
    "etc/cfg.toml",
    "etc/config.toml",
    "/etc/hospital/cfg.toml",
    "/etc/hospital/config.toml",
    "/opt/etc/hospital/cfg.toml",
    "/opt/etc/hospital/config.toml",
];

impl Cfg {
    pub fn new() -> R<Self> {
        let mut n = String::new();
        let mut f = String::new();

        for p in CFG_PATHS.into_iter().rev() {
            if un!(fs::exists(p)) {
                n = p.to_string();
                f = un!(fs::read_to_string(p));
                break;
            }
        }

        if n.is_empty() {
            return err_fmt!("cannot find config file out of {CFG_PATHS:#?}");
        }

        re!(toml::from_str(&f))
    }

    #[inline(always)]
    pub fn title(&self) -> String {
        self.site.clone().unwrap_or(Site::default()).title.clone()
    }
}

pub static CFG: LazyLock<Cfg> = LazyLock::new(|| match Cfg::new() {
    Ok(x) => x,
    Err(e) => fatal!("{e}"),
});
