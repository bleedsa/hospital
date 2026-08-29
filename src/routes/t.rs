use axum::{extract::Path, response::Html};
use axum_cookie::prelude::*;

use crate::{db::Db, pre::*};

pub async fn by_id(C: CookieManager, Path(id): Path<i64>) -> H<Html<String>> {
    let db = Db::new()?;
    let _ = db.me(&C)?;
    let t = db.get_thread(id)?;
    let goto = format!("/b/{}", t.board);
    let name = &t.name;

    Ok(page!(db, {
        ("{name}"),
        r#"
        <h1>{name}</h1>
        <p>#{id}@{time}</p>
        <div class="base-post">
            <img src="{img}" class="thread-img">
            <p class="post-content">{cont}</p>
        </p>
        <hr>

        <div class="new-post">
            <h3>new post</h3>
            <form
                action="/act/new-post"
                method="post"
                enctype="multipart/form-data"
            >
                <table>
                    <tr>
                        <td>post content</td>
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
                <input type="hidden" name="thread" value="{id}">
                <input type="submit" value="go">
            </form>
        </div>

        <div class="posts">
            {posts}
        </div>
        "#,
        id = t.id,
        time = timestamp_to_time(t.time),
        img = if let Some(id) = t.file {
            let f = un!(db.get_file(id), "{goto}");
            format!("/i/{}", f.id)
        } else {
            "".to_string()
        },
        cont = t.cont,
        posts = un!(db.get_posts(t.id))
            .into_iter()
            .map(|p| format!(
                r#"
                <div class="post-box" id="{id}">
                    <p class="bold"><a href="/t/{tid}#{id}">#{id}</a>::{time}</p>
                    <p class="post-content">{cont}</p>
                </div>
                "#,
                id = p.id,
                tid = t.id,
                time = timestamp_to_time(p.time),
                cont = p.cont,
            ))
            .collect::<String>(),
    }))
}
