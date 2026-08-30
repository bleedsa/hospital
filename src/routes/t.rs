use axum::{extract::Path, response::Html};
use axum_cookie::prelude::*;

use crate::{db::Db, pre::*};

pub async fn by_id(C: CookieManager, Path(id): Path<i64>) -> H<Html<String>> {
    let db = Db::new()?;
    let me = db.me(&C)?;
    let t = db.get_thread(id)?;
    let goto = format!("/b/{}", t.board);
    let b = un!(db.get_board(t.board), "{goto}");
    let name = &t.name;

    Ok(page!(db, {
         ("{name}"),
         r#"
        <h1><a href="/b/{bname}">/{bname}/</a></h1>
        <h2>{name}</h2>
        {back}
        <div class="post-box">
            <p class="bold">#{id}::<a href="/u/{uname}">{uname}</a>@{time}</p>
            <div class="base-post">
                {img}
                <p class="post-content">{cont}</p>
            </div>
        </div>

        <div class="posts">
            {posts}
        </div>

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

        <hr>
        {back}
        "#,
         id = t.id,
         bname = b.name,
         uname = h!(un!(db.get_user(t.author)).name),
         time = format!(r#"<span class="unix-time">{}</a>"#, t.time),
         img = if let Some(id) = t.file {
             let f = un!(db.get_file(id), "{goto}");
             format!(r#"<img src="/i/{}" class="post-img"><br>"#, f.id)
         } else {
             "".to_string()
         },
         cont = h!(t.cont),
         back = format!(r#"<p><a href="/b/{}">go back</a></p>"#, b.name),
         posts = un!(db.get_visible_posts(t.id))
             .into_iter()
             .map(|p| Ok(format!(
                 r#"
                <div class="post-box" id="{id}">
                    <p class="bold"><a href="/t/{tid}#{id}">#{id}</a>::<a href="/u/{uname}">{uname}</a>@{time}</p>
                    {img}
                    <p class="post-content">{cont}</p>
                    <form action="/act/hide-post" method="post">
                        {hide}
                    </form>
                </div>
                "#,
                 id = p.id,
                 tid = t.id,
                 uname = h!(un!(db.get_user(p.author)).name),
                 time = format!(r#"<span class="unix-time">{}</span>"#, p.time),
                 cont = h!(p.cont),
                 img = if let Some(id) = p.file {
                     let f = if let Ok(f) = db.get_file(id) {
                         f
                     } else {
                         return Ok(String::new())
                     };
                     format!(r#"<img src="/i/{}" class="post-img"><br>"#, f.id)
                 } else {
                     String::new()
                 },
                 hide = if me.admin {
                     format!(
                         r#"
                         <div class="hide-button">
                            <input type="submit" value="hide">
                            <input type="hidden" name="id" value="{id}">
                        </div>
                        "#,
                         id = p.id,
                     )
                 } else {
                     String::new()
                },
             )))
             .collect::<R<String>>()?,
    }))
}
