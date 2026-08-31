/** regex */
use regex::{Captures, Regex};

use crate::pre::*;
use std::sync::LazyLock;

/** replies ie >>12345 */
static REPLYRE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"&gt;&gt;[0-9]+"#).unwrap());

/** links ie http://badboy.institute/~skye */
static LINKRE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r#"(http|https):{s}{s}[a-zA-Z0-9]+\.[a-zA-Z0-9]+[a-zA-Z0-9%?~@#&"{s}"]*"#,
        s = "&#x2F;",
    )).unwrap()
});

/** mentions ie ~phoebe */
static MENTIONRE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\~[a-zA-Z0-9_\-@~]+"#).unwrap()
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

    /* replace mentions */
    let x = (&*MENTIONRE).replace_all(&x, |C: &Captures| {
        format!(r#"<a href="/u/{r}">{r}</a>"#, r = &C[0])
    });

    Ok(x.to_string())
}

#[test]
fn post_regex() {
    let a1 = format!(
        r#"<a href="http://test.com/test" target="_blank">http:{s}{s}test.com{s}test</a>"#,
        s = "&#x2F;"
    );

    for (x, y) in [
        (">>123", r#"<a href="/t/0#123">&gt;&gt;123</a>"#),
        ("http://test.com/test", &a1),
        ("~phoebe", r#"<a href="/u/~phoebe">~phoebe</a>"#),
    ]
    .into_iter()
    {
        let r = posts(0, h!(x)).unwrap();
        assert_eq!(&r, y);
    }
}
