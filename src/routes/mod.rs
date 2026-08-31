use crate::{db::Db, pre::*, tags::tag};
use axum::response::Html;
use axum_cookie::CookieManager;

pub mod act;
pub mod b;
pub mod dbg;
pub mod i;
pub mod t;
pub mod u;
pub mod admin;

#[macro_export]
macro_rules! invalid_str {
    ($s:expr, $L:expr) => {{ $s.is_empty() || $s.len() > $L }};
}

/** index/homepage (GET /) */
pub async fn index(C: CookieManager) -> H<Html<String>> {
    let db = Db::new()?;
    let me = if let Ok(x) = db.me(&C) { Some(x) } else { None };

    Ok(page!(db, me.as_ref().map(|m| m.id), {
        ("index"),
        r#"
        <h1>{title}.</h1>
        <p>{tag}.</p>
        "#,
        title = (&*CFG).title(),
        tag = tag(),
    }))
}

pub async fn login(C: CookieManager) -> H<Html<String>> {
    let db = Db::new()?;

    Ok(page!(db, None, {
        ("login"),
        r#"
        <h1>login.</h1>
        <div class="form">
            <form action="/act/login" method="post">
                <table>
                    <tr>
                        <td><span class="form-item">username</span></td>
                        <td>
                            <span class="form-item">
                                <input type="text" name="user">
                            </span>
                        </td>
                    </tr>
                    <tr>
                        <td><span class="form-item">password</span></td>
                        <td>
                            <span class="form-item">
                                <input type="password" name="pass">
                            </span>
                        </td>
                    </tr>
                </table>
                <input type="submit" value="go">
            </form>
        </div>
        "#
    }))
}
