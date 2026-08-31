#![allow(non_snake_case)]

use axum::{body::Bytes, extract::Multipart, response::Html};
use chrono::{DateTime, TimeZone, Utc};
use serde::Deserialize;

use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::HashMap, fs, sync::LazyLock};

pub mod css;
pub mod db;
pub mod js;
pub mod page;
pub mod passwd;
pub mod rand;
pub mod x;
pub mod routes;
pub mod tags;

pub mod pre {
    pub use crate::{
        CFG, H, L, R, err_fmt, err_page, fatal, h, int2bool, now, page, puts,
        re, timestamp_to_time, un, un_fatal, unh,
    };
}

/** `panic!()` but make it not ugly. */
#[macro_export]
macro_rules! fatal {
    ($($x:tt),* $(,)*) => {{
        eprintln!($($x),*);
        std::process::exit(-1);
    }};
}

/** print a string without a newline */
#[macro_export]
macro_rules! puts {
    ($($x:tt)*) => {{
        use std::io::{self, Write};
        print!($($x)*);
        let _ = io::stdout()
            .flush()
            .unwrap_or_else(|e| {
                $crate::fatal!("failed to flush stdout: {e}")
            });
    }};
}

/** re-wrap any `Result` into an `R<T>` */
#[macro_export]
macro_rules! re {
    ($r:expr) => {{ $r.map_err(|e| format!("{e}")) }};
    ($r:expr, $($g:tt)*) => {{
        match $r {
            r @ Ok(_) => r,
            Err(e) => Err(page! {
                ("error: {e}"),
                r#"
                <h1>error: {e}</h1>
                <p><a href="{goto}">go back</a></p>
                "#,
                goto = format!($($g)*),
            })?
        }
    }};
}

/** unwrap any `Result` into a `T` */
#[macro_export]
macro_rules! un {
    ($r:expr) => {{ $crate::re!($r)? }};
    ($r:expr, $($g:tt)*) => {{ $crate::re!($r, $($g)*)? }};
}

/** unwrap a result or fatal */
#[macro_export]
macro_rules! un_fatal {
    ($r:expr) => {{
        match $r {
            Ok(x) => x,
            Err(e) => fatal!("un!(): {e}"),
        }
    }};
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

#[macro_export]
macro_rules! h {
    ($str:expr) => {{ html_escape::encode_safe(&$str).to_string() }};
}

#[macro_export]
macro_rules! unh {
    ($str:expr) => {{ html_escape::decode_html_entities(&$str).to_string() }};
}

#[macro_export]
macro_rules! L {
    (($($m:tt)*) => $x:expr) => {{
        use std::{io::Write, fs::{self, OpenOptions}};
        use $crate::pre::*;

        let m = format!("[log][{}]: {}...", timestamp_to_time(now()?), format!($($m)*));

       let p = &(&*CFG).server.log_file;
       let mut f = if un!(fs::exists(p)) {
            un!(OpenOptions::new()
                .write(true)
                .append(true)
                .open(p))
       } else {
           un!(OpenOptions::new()
               .write(true)
               .create(true)
               .open(p))
       };

        puts!("{m}");
        if let Err(e) = write!(f, "{m}") {
            eprintln!("couldn't write to log file {p}: {e}");
        }

        let r = $x;

        println!("ok");
        if let Err(e) = writeln!(f, "ok") {
            eprintln!("couldn't write ok to log file {p}: {e}");
        }

        r
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
    pub log_file: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Site {
    pub title: String,
    pub themes: String,
}

impl Default for Site {
    fn default() -> Self {
        Self {
            title: "badboy hospital".to_string(),
            themes: "css/themes/".to_string(),
        }
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Cfg {
    pub site: Site,
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
    pub fn title(&self) -> &str {
        &self.site.title
    }
}

pub static CFG: LazyLock<Cfg> = LazyLock::new(|| match Cfg::new() {
    Ok(x) => x,
    Err(e) => fatal!("{e}"),
});

pub async fn multipart_to_map(m: &mut Multipart) -> R<HashMap<String, Bytes>> {
    /* map of multipart fields */
    let mut map = HashMap::new();

    while let Some(f) = un!(m.next_field().await) {
        let n = un!(f
            .name()
            .ok_or("no name for multipart field found".to_string()))
        .to_string();
        let d = un!(f.bytes().await);
        map.insert(n, d);
    }

    Ok(map)
}

#[inline(always)]
pub fn now() -> R<i64> {
    let start_time = SystemTime::now();
    Ok(un!(start_time.duration_since(UNIX_EPOCH)).as_secs() as i64)
}

#[inline(always)]
pub fn timestamp_to_time(time: i64) -> DateTime<Utc> {
    Utc.timestamp_opt(time, 0).unwrap()
}
