use axum::{
    Form,
    extract::Multipart,
    response::{Html, Redirect},
};
use axum_cookie::prelude::*;
use serde::Deserialize;
use tokio::time::Duration;

use crate::{db::Db, invalid_str, multipart_to_map, passwd, pre::*};
use std::collections::HashMap;

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

/** make a new thread */
pub async fn new_thread(C: CookieManager, mut m: Multipart) -> H<Html<String>> {
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
    let board = M("board")?;
    let name = M("name")?;
    let cont = M("content")?;

    /* sanitize */
    if invalid_str!(name, 64)
        || invalid_str!(cont, 2048)
        || invalid_str!(board, 32)
    {
        return err_page!(("invalid thread form parameters") => ("/b/{board}"));
    }

    /* get file bytes */
    let file = map.get("file");

    let db = un!(Db::new());

    Ok(page!(db, {
        ("new thread created"),
        r#"
        <h1>new thread created</h1>
        <table>
            {fields}
        </table>
        <h3>{name}</h3>
        <p>{cont}</p>
        <p>{file:?}</p>
        "#,
        fields = map.iter()
            .map(|(n, v)| format!(
                "
                <tr>
                    <td>{n}</td>
                    <td>{v:?}</td>
                </tr>
                "
            ))
            .collect::<String>(),
    }))
}
