/** regex */
use regex::{Captures, Regex};

use crate::pre::*;
use std::{borrow::Cow, sync::LazyLock};

static REPLYRE: LazyLock<Regex> =
    LazyLock::new(|| un_fatal!(Regex::new(r#"&gt;&gt;[0-9]+"#)));

static LINKRE: LazyLock<Regex> = LazyLock::new(|| {
    un_fatal!(Regex::new(&format!(
        r#"(http|https):{s}{s}[a-zA-Z0-9]+\.[a-zA-Z0-9]+[a-zA-Z0-9%?~@#&"{s}"]*"#,
        s = "&#x2F;",
    )))
});

pub fn posts<'a, X>(tid: i64, x: X) -> R<String>
where
    X: AsRef<str>,
{
    let x = x.as_ref();

    /* replace links */
    let x = (&*LINKRE).replace_all(x, |C: &Captures| {
        let a = &C[0];
        format!(r#"<a href="{un}" target="_blank">{a}</a>"#, un = unh!(a))
    });

    /* replace replies */
    let x = (&*REPLYRE).replace_all(&x, |C: &Captures| {
        let r = &C[0];
        let id = &r[8..];
        format!(r#"<a href="/t/{tid}#{id}">{r}</a>"#)
    });

    Ok(x.to_string())
}

#[test]
fn post_regex() {
    let a1 = format!(
        r#"<a href="http://test.com/test">http:{s}{s}test.com{s}test</a>"#,
        s = "&#x2F;"
    );

    for (x, y) in [
        (">>123", r#"<a href="/t/0#123">&gt;&gt;123</a>"#),
        ("http://test.com/test", &a1),
    ]
    .into_iter()
    {
        let r = posts(0, h!(x)).unwrap();
        assert_eq!(&r, y);
    }
}
