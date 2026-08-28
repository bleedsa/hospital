/** make an html page */
#[macro_export]
macro_rules! page {
    { ($($t:tt)*), $($b:tt)* } => {{
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
                        <span class="bold">{site_title}</span>
                        ::
                        <a href="/login">login</a>
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
        ))
    }};
}
