use axum::{Form, http::header::HeaderMap, response::Redirect};
use serde::Deserialize;

use crate::{db::Db, pre::*};

#[derive(Deserialize)]
pub struct LoginForm {
    user: String,
    pass: String,
}

pub async fn login(Form(f): Form<LoginForm>) -> R<(HeaderMap, Redirect)> {
    let db = Db::new()?;
    let u = db.get_user_by_name(&f.user)?;

    /* make the redirect. goto the user page */
    let red = Redirect::to(format!("/dbg/user/{}", u.id).as_ref());

    /* make the headers so we can set the session cookie */
    let headers = HeaderMap::new();

    Ok((headers, red))
}
