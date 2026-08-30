use std::{collections::HashMap, fs, sync::LazyLock};
use crate::{Site, pre::*, db::{Db, User}};

/** include the default css as a static str. RAWDOG. */
static DEFAULT_CSS: &str = include_str!("../css/base.css");

fn theme_path(f: &str) -> String {
    let d = (&*CFG).site.clone().unwrap_or(Site::default()).themes;
    format!("{d}/{f}.css")
}

pub static THEMES: LazyLock<HashMap<&'static str, String>> = LazyLock::new(|| {
    let mut r = HashMap::new();

    for (n, f) in [("blue screen of death", "blue"), ("default", "default")]
        .into_iter()
    {
        let p = theme_path(f);
        let c = if let Ok(x) = fs::read_to_string(&p) {
            x
        } else {
            continue;
        };

        r.insert(n, c);
    }

    r
});

#[inline(always)]
pub fn get_theme<N>(n: N) -> R<&'static str>
where
    N: AsRef<str>
{
    let n = n.as_ref();

    for (k, f) in &*THEMES {
        if &n == k {
            return Ok(&f);
        }
    }

    err_fmt!("theme \"{n}\" not found (no {})", theme_path(n))
}

pub fn get_theme_names() -> impl Iterator<Item=&'static str> {
    (&*THEMES).into_iter()
        .map(|(n, _)| *n)
}

/** get the css stylesheet as a string to inject into <style> in page!{} */
pub fn css(me: Option<i64>) -> R<String> {
    let t = if let Some(id) = me {
        get_theme(Db::new()?.get_theme(id)?).unwrap_or("")
    } else {
        ""
    };
    Ok(format!("{}\n{}", t, DEFAULT_CSS))
}
