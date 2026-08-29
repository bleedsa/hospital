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

    let threads = un!(db.get_threads(b.id), "/b/{}", b.id);

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

        <div class="threads">
            {threads}
        </div>
        "#,
        desc = b.desc,
        threads = threads
            .into_iter()
            .map(|t| format!(
                r#"
                <div class="thread-box">
                    <h3 id="{id}">
                        <a href="/b/{bid}#{id}">#{id}</a>::a href="/t/{id}">{name}</a>@<span class="utc">{time}</span>
                    </h3>
                    <p>{cont}</p>
                </div>
                "#,
                id = t.id,
                bid = b.id,
                name = t.name,
                time = timestamp_to_time(t.time),
                cont = {
                    let L = t.cont.len();
                    const M: usize = 32;
                    let z = if L > M {
                        M
                    } else {
                        L
                    };
                    &t.cont[..z]
                },
            ))
            .collect::<String>(),
    }))
}
