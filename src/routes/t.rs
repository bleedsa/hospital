use axum::{extract::Path, response::Html};
use axum_cookie::prelude::*;

use crate::{db::Db, pre::*, x};

pub async fn by_id(C: CookieManager, Path(id): Path<i64>) -> H<Html<String>> {
    let db = Db::new()?;
    let me = db.me(&C)?;
    let t = db.get_thread(id)?;
    let goto = format!("/b/{}", t.board);
    let b = un!(db.get_board(t.board), "{goto}");
    let name = &t.name;

    un!(db.mark_as_read(me.id, t.id), "{goto}");

    Ok(page!(db, Some(me.id), {
         ("{name}"),
         r#"
        <h1><a href="/b/{bname}">/{bname}/</a></h1>
        <h2>{name}</h2>
        {hide_unhide}
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
        {new_post}
        {back}
        "#,
         id = t.id,
         bname = b.name,
         uname = h!(format!("~{}", un!(db.get_user(t.author)).name)),
         time = format!(r#"<span class="unix-time">{}</a>"#, t.time),
         img = if let Some(id) = t.file {
             let f = un!(db.get_file(id), "{goto}");
             format!(r#"<img src="/i/{}" class="post-img"><br>"#, f.id)
         } else {
             "".to_string()
         },
         cont = x::threads(h!(t.cont)),
         back = format!(r#"<p><a href="/b/{}#{}">go back</a></p>"#, b.name, t.id),
         hide_unhide = if me.admin {
             if t.hidden {
                 format!(
                    r#"
                    <form action="/admin/unhide" method="post">
                        <input type="submit" value="unhide thread">
                        <input type="hidden" name="id" value="{id}">
                        <input type="hidden" name="ty" value="t">
                        <input type="hidden" name="goto" value="/t/{id}">
                    </form>
                    "#,
                    id = t.id
                )
             } else {
                 format!(
                     r#"
                     <form action="/act/hide-thread" method="post">
                        <input type="submit" value="hide thread">
                        <input type="hidden" name="id" value="{id}">
                        <input type="hidden" name="goto" value="/t/{id}">
                    </form>
                    "#,
                    id = t.id
                 )
             }
         } else {
             String::new()
         },
         new_post = if t.locked() {
             r#"<h3>this thread is locked. you cannot post in it."#.into()
         } else {
             format!(
                r#"
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
                "#,
             )
         },
         posts = un!(db.get_visible_posts(t.id))
             .into_iter()
             .map(|p| Ok(format!(
                 r#"
                <div class="post-box" id="{id}">
                    <p class="bold"><a href="/t/{tid}#{id}">#{id}</a>::<a href="/u/{uname}">{uname}</a>@{time}<span class="replies">{replies}</span></p>
                    {img}
                    <p class="post-content">{cont}</p>
                    <form action="/act/hide-post" method="post">
                        {hide}
                    </form>
                </div>
                "#,
                 id = p.id,
                 tid = t.id,
                 uname = h!(format!("~{}", un!(db.get_user(p.author)).name)),
                 time = format!(r#"<span class="unix-time">{}</span>"#, p.time),
                 cont = x::posts(t.id, h!(p.cont)),
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
                 replies = if let x = un!(db.get_post_replies(p.id))
                     .into_iter()
                     .map(|id| format!(r#"<a href="/t/{}#{id}">&gt;&gt;{id}</a>"#, t.id))
                     .collect::<Vec<_>>()
                     && x.len() > 0 {
                         "|".to_owned() + &x.join(";")
                 } else {
                     String::new()
                 },
                 hide = if me.admin {
                     format!(
                         r#"
                         <div class="admin-button">
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
