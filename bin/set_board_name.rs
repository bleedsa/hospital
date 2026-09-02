#![allow(non_snake_case)]

use clap::Parser;

use hospital::{db::{Board, Db}, pre::*};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    old: String,
    #[arg(short, long)]
    new: String,
}

fn main() -> R<()> {
    let A = Args::parse();
    let db = Db::new()?.init()?;

    let u = db.get_board_by_name(&A.old)?;
    db.update_board_name(u.id, &A.new)?;

    let b = db.get_board(u.id)?;
    println!("{}\n{b}", Board::fields());

    Ok(())
}
