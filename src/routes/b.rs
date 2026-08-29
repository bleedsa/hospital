use crate::{db::Db, pre::*};
use axum::{extract::Path, response::Html};
use axum_cookie::prelude::*;

pub async fn by_name<'a>(
    Path(n): Path<String>,
    c: CookieManager,
) -> H<Html<String>> {
    let db = un!(Db::new(), "/");
    let me = un!(db.me(&c), "/");
    let b = un!(db.get_board_by_name(&n), "/");

    Ok(page!(db, {
        ("{n} ({}) :: view board", me.name),
        r#"
        <h1>{n}</h1>
        <p><span class="italic">{desc}</span></p>
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
        <hr>
        "#,
        desc = b.desc,
    }))
}
