use axum::{extract::Path, response::Html};
use axum_cookie::prelude::*;

use crate::{db::Db, pre::*};

pub async fn by_name(C: CookieManager, Path(n): Path<String>) -> H<Html<String>> {
    let db = Db::new()?;
    let me = db.me(&C)?;
    let u = db.get_user_by_name(&n)?;
    todo!()
}
