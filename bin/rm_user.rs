#![allow(non_snake_case)]

use clap::Parser;
use hospital::{db::Db, pre::*};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, short)]
    user: String,
}

fn main() -> R<()> {
    let A = Args::parse();
    let db = Db::new()?;

    let u = db.get_user_by_name(&A.user)?;
    let _ = db.rm_user(u.id)?;

    println!("rm'd user {} with id {}", u.name, u.id);

    Ok(())
}
