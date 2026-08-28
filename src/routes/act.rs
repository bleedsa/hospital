use axum::{Form, response::Redirect};
use axum_cookie::prelude::*;
use serde::Deserialize;
use tokio::time::Duration;

use crate::{db::Db, passwd, pre::*};

#[derive(Deserialize)]
pub struct LoginForm {
    user: String,
    pass: String,
}

/** login action */
pub async fn login(C: CookieManager, Form(f): Form<LoginForm>) -> H<Redirect> {
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
