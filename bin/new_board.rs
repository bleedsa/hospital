#![allow(non_snake_case)]

use clap::Parser;
use hospital::{db::Db, pre::*};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, short)]
    board: String,
    #[arg(long, short)]
    description: String,
}

fn main() -> R<()> {
    let A = Args::parse();
    let db = Db::new()?.init()?;

    puts!("creating new board {} ({})...", A.board, A.description);
    let b = db.new_board(&A.board, &A.description)?;
    println!("ok");

    println!("created new board: {b:#?}");

    Ok(())
}
