#![allow(non_snake_case)]

use clap::Parser;

use hospital::{db::Db, pre::*};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    id: i64,
    #[arg(short, long)]
    post: bool,
}

fn main() -> R<()> {
    let A = Args::parse();
    let db = Db::new()?.init()?;

    if A.post {
        db.visible_post(A.id)?;
        println!("{}", db.get_post(A.id)?);
    } else {
        db.visible_thread(A.id)?;
        println!("{}", db.get_thread(A.id)?);
    }

    Ok(())
}
