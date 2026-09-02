use std::{collections::HashMap, fs, path::Path, sync::LazyLock};
use crate::{db::Db, pre::*};

/** include the default css as a static str. RAWDOG. */
static DEFAULT_CSS: &str = include_str!("../css/base.css");

pub static THEMES: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    let mut r = HashMap::new();

    macro_rules! U {
        ($x:expr) => {{ U!($x, Ok) }};

        ($x:expr, $q:ident) => {{
            if let $q(x) = $x {
                x
            } else {
                return r;
            }
        }};
    }

    for e in U!(fs::read_dir(&(&*CFG).site.themes)) {
        let d = U!(e);
        let p = d.path();

        if p.is_file() {
            let f = U!(fs::read_to_string(&p));
            let n = Path::new(U!(p.file_name(), Some));
            let n = U!(n.file_prefix(), Some).to_str();
            let n = U!(n, Some).to_string();
            r.insert(n, f);
        }
    }

    r
});

#[inline(always)]
pub fn get_theme<N>(n: N) -> Option<&'static str>
where
    N: AsRef<str>,
{
    let n = n.as_ref();

    for (k, f) in &*THEMES {
        if &n == k {
            return Some(&f);
        }
    }

    None
}

pub fn get_theme_names() -> impl Iterator<Item = &'static str> {
    (&*THEMES).into_iter().map(|(n, _)| n.as_str())
}

/** get the css stylesheet as a string to inject into <style> in page!{} */
pub fn css(db: &Db, me: Option<i64>) -> R<String> {
    let (t, vars, body) = if let Some(id) = me {
        let t = get_theme(db.get_theme(id)?).unwrap_or("");
        let css = db.get_css(id)?;
        if let Some(css) = css {
            (t, css.vars, css.css)
        } else {
            (t, String::new(), String::new())
        }
    } else {
        ("", String::new(), String::new())
    };

    Ok(format!(
        r#"
        :root {{
            {theme}
            {vars}
        }}

        {base}
        {body}
        "#,
        theme = t,
        base = DEFAULT_CSS
    ))
}
