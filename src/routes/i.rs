use axum::extract::Path;
use axum_cookie::prelude::*;

use crate::{db::Db, pre::*};

pub async fn by_id(C: CookieManager, Path(id): Path<i64>) -> H<Vec<u8>> {
    let db = Db::new()?;
    let _ = db.me(&C)?;
    let f = un!(db.get_file(id), "/");

    Ok(f.bytes)
}
