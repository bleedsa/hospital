#![allow(non_snake_case)]

use clap::Parser;

use hospital::{
    db::{Board, Db, Post, Thread},
    pre::*,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    threads: bool,
    #[arg(short, long)]
    posts: bool,
}

fn main() -> R<()> {
    let db = Db::new()?.init()?;
    let A = Args::parse();

    for b in db.get_boards()? {
        println!("{}\n{b}", Board::fields());

        if A.threads {
            println!("  :: {}", Thread::fields());
            for t in db.get_threads(b.id)?.into_iter() {
                println!("  => {t}");

                if A.posts {
                    let ps = db.get_posts(t.id)?;
                    if ps.len() > 0 {
                        println!("    ** {}", Post::fields());
                        for p in ps.into_iter() {
                            println!("    -> {p}");
                        }
                    }
                }
            }
        }

        println!();
    }

    Ok(())
}
