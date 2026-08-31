/** regex */

use regex::{Captures, Regex};

use std::{sync::LazyLock, borrow::Cow};
use crate::pre::*;

static REPLYRE: LazyLock<Regex> = LazyLock::new(|| un_fatal!(Regex::new(r#"&gt;&gt;[0-9]+"#)));

pub fn posts<'a, X>(tid: i64, x: X) -> R<String>
where
    X: AsRef<str>,
{
    let x = x.as_ref();

    /* replace replies */
    let x = (&*REPLYRE).replace_all(x, |C: &Captures| {
        let r = &C[0];
        let id = &r[8..];
        format!(
            r#"<a href="/t/{tid}#{id}">{r}</a>"#,
        )
    });

    Ok(x.to_string())
}

#[test]
fn post_regex() {
    for (x, y) in [
        (">>123", r#"<a href="\#123">&gt;&gt;123</a>"#)
    ]
    .into_iter()
    {
        let r = posts(h!(x)).unwrap();
        assert_eq!(&r, y);
    }
}
