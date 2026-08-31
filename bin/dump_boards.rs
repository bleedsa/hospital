#![allow(non_snake_case)]

use hospital::{db::{Board, Db, Thread, Post}, pre::*};

fn main() -> R<()> {
    let db = Db::new()?.init()?;

    for b in db.get_boards()? {
        println!("{}\n{b}", Board::fields());

        println!("  :: {}", Thread::fields());
        for t in db.get_threads(b.id)?.into_iter() {
            println!("  => {t}");

            let ps = db.get_posts(t.id)?;
            if ps.len() > 0 {
                println!("    ** {}", Post::fields());
                for p in ps.into_iter() {
                    println!("    -> {p}");
                }
            }
        }

        println!();
    }

    Ok(())
}
