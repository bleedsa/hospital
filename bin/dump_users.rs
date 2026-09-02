#![allow(non_snake_case)]

use hospital::{db::Db, pre::*};

fn main() -> R<()> {
    let db = Db::new()?.init()?;
    let a = db.get_all_users()?;

    for u in a.into_iter() {
        println!("{}", u.name);
    }

    Ok(())
}
