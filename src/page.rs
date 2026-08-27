/** make an html page */
#[macro_export]
macro_rules! page {
    { ($($t:tt)*), $($b:tt)* } => {{
        use axum::response::Html;
        use $crate::css::css;
        Html(format!(
            r#"
            <!DOCTYPE html>
            <html>
                <head>
                    <title>{title} :: badboy hospital</title>
                    <meta charset="utf-8">
                    <style>{css}</style>
                </head>
                <body>
                    <div class="top">
                        <span class="bold">badboy hospital</span>
                    {body}
                </body>
            </html>
            "#,
            title = format!($($t)*),
            body = format!($($b)*),
            css = css(),
        ))
    }};
}
