use axum::{Form, response::{Redirect, Html}};
use axum_cookie::prelude::*;
use serde::Deserialize;

use crate::{db::Db, pre::*};

macro_rules! H {
    ($v:expr, $T:expr) => {{
        un!($v)
            .into_iter()
            .map(|q| {
                format!(
                    r#"
                    <tr>
                        <td>{q}</td>
                        <td>
                            <div class="unhide-form">
                                <form method="post" action="/admin/unhide">
                                    <input type="submit" value="make visible">
                                    <input type="hidden" name="id" value="{id}">
                                    <input type="hidden" name="ty" value="{T}">
                                </form>
                            </div>
                        </td>
                    </tr>
                    "#,
                    T = $T,
                    id = q.id,
                )
            })
            .collect::<String>()
    }}
}

pub async fn admin(C: CookieManager) -> H<Html<String>> {
    let db = Db::new()?;
    let me = db.me(&C)?;

    if !me.admin {
        return err_page!(("not an admin") => ("/"));
    }

    Ok(page!(db, Some(me.id), {
        ("admin panel"),
        r#"
        <h1>admin panel</h1>
        <div class="admin-panel">
            <h3>hidden boards</h3>
            <table class="hidden-table">{hidden_boards}</table>

            <h3>hidden threads</h3>
            <table class="hidden-table">{hidden_threads}</table>

            <h3>hidden posts</h3>
            <table class="hidden-table">{hidden_posts}</table>
        </div>
        "#,
        hidden_boards = H!(db.get_all_hidden_boards(), 'b'),
        hidden_threads = H!(db.get_all_hidden_threads(), 't'),
        hidden_posts = H!(db.get_all_hidden_posts(), 'p'),
    }))
}

#[derive(Deserialize)]
pub struct UnhideForm {
    pub id: i64,
    pub ty: char,
    pub goto: Option<String>,
}

pub async fn unhide(
    C: CookieManager,
    Form(f): Form<UnhideForm>,
) -> H<Redirect>
{
    let db = Db::new()?;
    let me = db.me(&C)?;

    if !me.admin {
        return err_page!(("not an admin") => ("/"));
    }

    match f.ty {
        'b' => {db.visible_board(f.id)?;},
        't' => {db.visible_thread(f.id)?;},
        'p' => {db.visible_post(f.id)?;},
        t => return err_page!(("invalid form type: {t}") => ("/admin")),
    };

    Ok(Redirect::to(if let Some(g) = f.goto.as_ref() {
        g
    } else {
        "/admin"
    }))
}
