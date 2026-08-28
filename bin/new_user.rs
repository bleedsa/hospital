#![allow(non_snake_case)]

use clap::Parser;

use hospital::{db::Db, pre::*};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    user: String,
    #[arg(short, long)]
    password: String,
}

fn main() -> R<()> {
    let A = Args::parse();
    let db = Db::new()?;

    let u = db.new_user(&A.user, &A.password)?;
    println!("{u:#?}");

    Ok(())
}
