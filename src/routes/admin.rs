use axum::response::Html;
use axum_cookie::prelude::*;

use crate::{db::Db, pre::*};

pub async fn main(
    C: CookieManager,
) -> H<Html<String>>
{
    let db = Db::new()?;
    let me = db.me(&C)?;

    Ok(page!(db, Some(me.id), {
        ("admin panel"),
        r#"
        <h1>admin panel</h1>
        <div class="admin-panel">
            <h3>hidden threads</h3>
            <div class="hidden-items">
                {hidden_threads}
            </div>
            <br>

            <h3>hidden posts</h3>
            <div class="hidden-items">
                {hidden_posts}
            </div>
        </div>
        "#,
        hidden_threads = "",
        hidden_posts = "",
    }))
}
