#![allow(non_snake_case)]

use clap::Parser;

use hospital::{db::Db, pre::*};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    user: String,
    #[arg(short, long)]
    pass: String,
}

fn main() -> R<()> {
    let A = Args::parse();
    let db = Db::new()?.init()?;

    let u = db.get_user_by_name(&A.user)?;
    db.update_pass(u.id, &A.pass)?;

    let u = db.get_user(u.id)?;
    println!("{u:#?}");

    Ok(())
}
