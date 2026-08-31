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
        r#"(http|https):{s}{s}[a-zA-Z0-9]+\.[a-zA-Z0-9]+[a-zA-Z0-9%?@#&"{s}"]*"#,
        s = "&#x2F;",
    )).unwrap()
});

/** mentions ie ~phoebe */
static MENTIONRE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"~[a-zA-Z0-9_\-@~]+"#).unwrap()
});

/** replace links */
#[inline(always)]
fn links<X>(x: X) -> String
where
    X: AsRef<str>
{
    (&*LINKRE).replace_all(x.as_ref(), |C: &Captures| {
        let a = &C[0];
        format!(r#"<a href="{un}" target="_blank">{a}</a>"#, un = unh!(a))
    }).to_string()
}

/** replace replies */
fn replies<X>(tid: i64, x: X) -> String
where
    X: AsRef<str>
{
    (&*REPLYRE).replace_all(x.as_ref(), |C: &Captures| {
        let r = &C[0];
        let id = &r[8..];
        format!(r#"<a href="/t/{tid}#{id}">{r}</a>"#)
    }).to_string()
}

/** replace mentions */
fn mentions<X>(x: X) -> String
where
    X: AsRef<str>
{
    (&*MENTIONRE).replace_all(x.as_ref(), |C: &Captures| {
        format!(r#"<a href="/u/{r}">{r}</a>"#, r = &C[0])
    }).to_string()
}

pub fn posts<X>(tid: i64, x: X) -> String
where
    X: AsRef<str>,
{
    mentions(replies(tid, links(x.as_ref())))
}

pub fn threads<X>(x: X) -> String
where
    X: AsRef<str>
{
    mentions(links(x.as_ref()))
}

#[test]
fn post_regex() {
    let s = "&#x2F;";
    let a1 = format!(
        r#"<a href="http://test.com/test" target="_blank">http:{s}{s}test.com{s}test</a>"#,
    );
    let a2 = format!(
        r#"<a href="https://google.com" target="_blank">https:{s}{s}google.com</a>"#
    );

    for (x, y) in [
        (">>123", r#"<a href="/t/0#123">&gt;&gt;123</a>"#),
        ("http://test.com/test", &a1),
        ("~phoebe",  r#"<a href="/u/~phoebe">~phoebe</a>"#),
        ("https://google.com", &a2),
   ]
    .into_iter()
    {
        let r = posts(0, h!(x));
        assert_eq!(&r, y);
    }
}

#[test]
fn thread_regex() {
    for (x, y) in [
        (">>123", r#"&gt;&gt;123"#),
        ("~phoebe", r#"<a href="/u/~phoebe">~phoebe</a>"#),
    ]
        .into_iter()
    {
        let r = threads(h!(x));
        assert_eq!(&r, y);
    }
}

