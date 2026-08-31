use axum::{extract::Path, response::Html};
use axum_cookie::prelude::*;

use crate::{db::Db, pre::*};

pub async fn by_name<'a>(
    Path(n): Path<String>,
    c: CookieManager,
) -> H<Html<String>> {
    let db = un!(Db::new(), "/");
    let b = un!(db.get_board_by_name(&n), "/");
    let me = un!(db.me(&c), "/b/{}", b.id);

    Ok(page!(db, Some(me.id), {
        ("/{n}/ :: view board"),
        r#"
        <h1>/{n}/</h1>
        <p><span class="italic">{desc}</span></p>
        {hide_board}
 
        <div class="new-thread">
            <h3>new thread</h3>
            <form
                action="/act/new-thread"
                method="post"
                enctype="multipart/form-data"
            >
                <table>
                    <tr>
                        <td>name</td>
                        <td><input type="text" name="name"></td>
                    </tr>
                    <tr>
                        <td>content</td>
                        <td>
                            <textarea
                                name="content"
                                rows="3"
                                cols="30"
                            ></textarea>
                        </td>
                    </tr>
                    <tr>
                        <td>image</td>
                        <td><input type="file" name="file"></td>
                    </tr>
                </table>
                <input type="hidden" name="board" value="{n}">
                <input type="submit" value="go">
            </form>
        </div>

        <div class="threads">
            {threads}
        </div>
        "#,
        desc = b.desc,
        hide_board = if me.admin {
            if b.hidden {
                format!(
                    r#"
                    <form action="/admin/unhide" method="post">
                        <input type="submit" value="unhide board">
                        <input type="hidden" name="id" value="{id}">
                        <input type="hidden" name="ty" value="b">
                        <input type="hidden" name="goto" value="/b/{name}">
                    </form>
                    "#,
                    id = b.id,
                    name = b.name,
                )
            } else {
                format!(
                    r#"
                    <form action="/act/hide-board" method="post">
                        <input type="submit" value="hide board">
                        <input type="hidden" name="id" value="{}">
                    </form>
                    "#,
                    b.id
                )
            }
        } else {
            String::new()
        },
        threads = un!(db.get_visible_threads(b.id), "/b/{}", b.id)
            .map(|t| Ok(format!(
                r#"
                <div class="thread-box">
                    <h3 id="{id}">
                        <a href="/b/{bname}#{id}">#{id}</a>::<a href="/t/{id}">{name}</a>::<a href="/u/{uname}">{uname}</a>@<span class="unix-time">{time}</span> {unread}
                    </h3>
                    <p>{cont}</p>
                    <div class="admin-panel-box">{admin_panel}</div>
                </div>
                "#,
                id = t.id,
                bname = b.name,
                uname = h!(format!("~{}", un!(db.get_user(t.author)).name)),
                name = h!(t.name),
                time = t.time,
                unread = if db.is_read(me.id, t.id)? {
                    ""
                } else {
                    "(unread)"
                },
                cont = {
                    let L = t.cont.len();
                    const M: usize = 32;
                    let z = if L > M {
                        M
                    } else {
                        L
                    };
                    format!("{}{}", h!(&t.cont[..z]), if z == M { "..." } else { "" })
                },
                admin_panel = if me.admin {
                    format!(
                        r#"
                        <form action="/act/hide-thread" method="post">
                            <div class="admin-button">
                                <input type="submit" value="hide">
                                <input type="hidden" name="id" value="{id}">
                            </div>
                        </form>

                        <form action="/act/lock-thread" method="post">
                            <div class="admin-button">
                                <input type="submit" value="{lock_unlock}">
                                <input type="hidden" name="id" value="{id}">
                                <input type="hidden" name="locked" value="{locked}">
                            </div>
                        </form>
                        "#,
                        id = t.id,
                        lock_unlock = if t.locked() {
                            "unlock"
                        } else {
                            "lock"
                        },
                        locked = t.locked(),
                    )
                } else {
                    String::new()
                },
            )))
            .collect::<R<String>>()?,
    }))
}
