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
    let db = Db::new()?.init()?;

    let u = db.new_user(&A.user, &A.password)?;
    println!("{u:#?}");

    Ok(())
}

#[cfg(test)]
use hospital::passwd;

#[test]
fn new_user() {
    let db = Db::create("run/test/bin/new_user.db").unwrap().init().unwrap();
    let u = db.new_user("test", "test").unwrap();
    assert_eq!(&u.name, "test");
    assert!(passwd::verify("test", &u.hash).unwrap());
}
