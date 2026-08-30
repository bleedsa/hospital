/** make an html page */
#[macro_export]
macro_rules! page {
    { ($($t:tt)*), $($b:tt)* } => {{
        use $crate::db::Db;
        $crate::page!(Db::new()?, None, { ($($t)*), $($b)* })
    }};

    ($db:expr, $me:expr, { ($($t:tt)*), $($b:tt)* }) => {{
        use axum::response::Html;
        use $crate::{pre::*, css, js};

        Html(format!(
            r#"
            <!DOCTYPE html>
            <html>
                <head>
                    <title>{title} :: {site_title}</title>
                    <meta charset="utf-8">
                    <style>{css}</style>
                </head>
                <body>
                    <div class="top">
                        <span class="bold">
                            <a href="/">{site_title}</a>
                        </span>
                        ::
                        <a href="/login">login</a>
                        ::
                        {boards}
                    </div>
                    <hr>
                    {body}
                    <div class="bottom">
                        (c)<a href="http://badboy.institute/~skye" target="_blank">skylar bleed</a> 2026|<a href="https://raw.githubusercontent.com/bleedsa/hospital/refs/heads/master/LICENSE" target="_blank">Mozilla Public License/2.0</a>|<a href="https://github.com/bleedsa/hospital" target="_blank">view source</a>
                    </div>
                </body>
                <script>{js}</script>
            </html>
            "#,
            site_title = (&*CFG).title(),
            title = format!($($t)*),
            body = format!($($b)*),
            css = css::css($me)?,
            js = js::base(),
            boards = $db.get_visible_boards()?
                .into_iter()
                .map(|b| format!(
                    r#"
                    <span class="board-a">
                        <a href="/b/{name}">/{name}/</a>
                    </span>
                    "#,
                    name = b.name,
                ))
                .collect::<Vec<_>>()
                .join("\\"),
        ))
    }};
}

/** make an error page */
#[macro_export]
macro_rules! err_page {
    (($($x:tt)*) => ($($e:tt)*)) => {{
        use axum::response::Html;

        let e = format!($($x)*);
        Err::<_, Html<String>>($crate::page! {
            ("error: {e}"),
            r#"
            <h1>error: {e}</h1>
            <p><a href="{goto}">go back</a></p>
            "#,
            goto = format!($($e)*),
        })
    }};
}
