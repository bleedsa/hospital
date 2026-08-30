use axum::{Form, extract::Multipart, response::Redirect};
use axum_cookie::prelude::*;
use serde::Deserialize;
use tokio::time::Duration;

use crate::{db::Db, invalid_str, multipart_to_map, passwd, pre::*};

#[derive(Deserialize)]
pub struct LoginForm {
    user: String,
    pass: String,
}

/** login action */
pub async fn login(C: CookieManager, Form(f): Form<LoginForm>) -> H<Redirect> {
    /* safety */
    if invalid_str!(f.user, 64) || invalid_str!(f.pass, 64) {
        return err_page!(("invalid form parameters") => ("/login"));
    }

    /* make a db and get the user from the form */
    let db = Db::new()?;
    let u = db.get_user_by_name(&f.user)?;

    /* check the password */
    if let Err(_) = passwd::verify(&f.pass, &u.hash) {
        return err_page!(("invalid password for user {}", u.name) => ("/login"));
    }

    /* make a new session */
    let s = db.new_session(u.id)?;

    /* set the cookie */
    let mut c = Cookie::new("session", s.hash);
    c.set_path("/");
    c.set_max_age(Duration::from_secs(1_000_000));
    C.add(c);

    /* go back */
    Ok(Redirect::to("/"))
}

/** make a new post in a thread */
pub async fn new_post(C: CookieManager, mut m: Multipart) -> H<Redirect> {
    let db = Db::new()?;
    let me = db.me(&C)?;

    let map = multipart_to_map(&mut m).await?;

    /* get string fields as string */
    let M = |n| -> R<String> {
        Ok(un!(str::from_utf8(un!(map
            .get(n)
            .ok_or(format!("no such field {n}")))))
        .to_string())
    };

    /* grab the thread and board */
    let thread = un!(M("thread")?.parse::<i64>());
    let goto = format!("/t/{thread}");

    /* grab the thread and board structs */
    let thread = un!(db.get_thread(thread), "{goto}");

    /* convert other params to str */
    let cont = M("content")?;

    if cont.len() > 2048 {
        return err_page!(
            ("invalid content: post content must not exceed 2048 chars")
            =>
            ("{goto}")
        );
    }

    /* wrap up the file content in an Option */
    let file = if let Some(f) = map.get("file") {
        if f.is_empty() { None } else { Some(f) }
    } else {
        None
    };

    let p = un!(
        db.new_post(thread.id, me.id, &cont, file.cloned()),
        "{goto}"
    );

    Ok(Redirect::to(&format!("/t/{}#{}", thread.id, p.id)))
}

/** make a new thread */
pub async fn new_thread(C: CookieManager, mut m: Multipart) -> H<Redirect> {
    let db = Db::new()?;
    let me = db.me(&C)?;

    let map = multipart_to_map(&mut m).await?;

    /* get string fields as string */
    let M = |n| -> R<String> {
        Ok::<_, String>(
            un!(str::from_utf8(un!(map
                .get(n)
                .ok_or(format!("no such field {n}")))))
            .to_string(),
        )
    };

    /* board setup */
    let board = M("board")?;
    if invalid_str!(board, 32) {
        return err_page!(("board {board} is invalid") => ("/"));
    }
    let goto = format!("/b/{board}");

    /* get the board by name */
    let board = un!(db.get_board_by_name(board), "{goto}");

    /* convert other params to strings */
    let name = M("name")?;
    let cont = M("content")?;

    /* sanitize */
    if invalid_str!(name, 64) || invalid_str!(cont, 2048) {
        return err_page!(("invalid thread form parameters") => ("{goto}"));
    }

    /* get file bytes */
    let file = if let Some(f) = map.get("file") {
        if f.is_empty() { None } else { Some(f) }
    } else {
        None
    };

    let t = un!(
        db.new_thread(board.id, me.id, &name, &cont, file.cloned()),
        "{goto}"
    );

    Ok(Redirect::to(&format!("/t/{}", t.id)))
}

#[derive(Deserialize)]
pub struct UpdateUserForm {
    pub bio: String,
    pub pass: String,
}

pub async fn update_user(
    C: CookieManager,
    Form(f): Form<UpdateUserForm>,
) -> H<Redirect> {
    let db = Db::new()?;
    let me = db.me(&C)?;
    let goto = format!("/u/{}", me.name);

    if f.bio.len() > 1024 || f.pass.len() > 64 {
        return err_page!(("invalid user fields") => ("{goto}"));
    }

    if !f.bio.is_empty() {
        db.update_bio(me.id, &f.bio)?;
    }

    if !f.pass.is_empty() {
        db.update_pass(me.id, &f.pass)?;
    }

    Ok(Redirect::to(&goto))
}

#[derive(Deserialize)]
pub struct HideThreadForm {
    pub id: i64,
}

pub async fn hide_thread(
    C: CookieManager,
    Form(f): Form<HideThreadForm>,
) -> H<Redirect> {
    let db = Db::new()?;
    let me = db.me(&C)?;

    if !me.admin {
        return err_page!(("you aren't an admin") => ("/"));
    }

    let t = db.get_thread(f.id)?;
    let b = db.get_board(t.board)?;
    let goto = format!("/b/{}", b.name);

    un!(db.hide_thread(t.id), "{goto}");

    Ok(Redirect::to(&goto))
}

#[derive(Deserialize)]
pub struct HidePostForm {
    pub id: i64,
}

pub async fn hide_post(
    C: CookieManager,
    Form(f): Form<HidePostForm>,
) -> H<Redirect> {
    let db = Db::new()?;
    let me = db.me(&C)?;

    if !me.admin {
        return err_page!(("you aren't an admin") => ("/"));
    }

    let p = db.get_post(f.id)?;
    let t = db.get_thread(p.thread)?;
    let goto = format!("/t/{}", t.id);

    un!(db.hide_post(p.id), "{goto}");

    Ok(Redirect::to(&goto))
}
