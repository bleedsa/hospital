use axum::{response::Redirect, Form};
use serde::Deserialize;

use crate::{pre::*, db::Db};

#[derive(Deserialize)]
pub struct LoginForm {
    user: String,
    pass: String,
}

pub async fn login(Form(f): Form<LoginForm>) -> R<Redirect> {
    let db = Db::new()?;
    let u = db.get_user_by_name(&f.user)?;
    Ok(Redirect::to(format!("/dbg/user/{}", u.id).as_ref()))
}
