/** make an html page */
#[macro_export]
macro_rules! page {
    { ($($t:tt)*), $($b:tt)* } => {{
        use $crate::db::Db;
        $crate::page!(Db::new()?, { ($($t)*), $($b)* })
    }};

    ($db:expr, { ($($t:tt)*), $($b:tt)* }) => {{
        use axum::response::Html;
        use $crate::{pre::*, css::css};

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
                </body>
            </html>
            "#,
            site_title = (&*CFG).title(),
            title = format!($($t)*),
            body = format!($($b)*),
            css = css(),
            boards = $db.get_boards()?
                .into_iter()
                .map(|b| format!(
                    r#"
                    <span class="board-a">
                        <a href="/b/{name}">{name}</a>
                    </span>
                    "#,
                    name = b.name,
                ))
                .collect::<Vec<_>>()
                .join("/")
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
