use axum::{Router, routing::post};
use axum_cookie::prelude::*;
use axum_test::TestServer;

use crate::{
    db::Db,
    pre::*,
    routes::{self, act::LoginForm},
};
use std::fs;

fn O(p: &str) -> (Db, Option<String>) {
    let p = "run/test/act/".to_owned() + p + ".db";

    if fs::exists(&p).unwrap() {
        fs::remove_file(&p).unwrap();
    }

    set_db_path(p.clone()).unwrap();

    (Db::create(p.clone()).unwrap().init().unwrap(), Some(p))
}

fn U<F>(f: F) -> Router
where
    F: Fn(Router) -> Router,
{
    f(Router::new().route("/act/login", post(routes::act::login)))
        .layer(CookieLayer::default())
}

#[tokio::test]
async fn login_user_not_found() {
    let (_, db_path) = O("login_user_not_found");

    let app = U(|r| r);
    let srv = TestServer::new(app);
    let res = srv.post("/act/login").form(&LoginForm {
        user: "A".to_string(),
        pass: "A".to_string(),
        db_path,
    }).await;

    assert!(res.text().contains("user not found"));
}

#[tokio::test]
async fn login_empty_inputs() {
    let (_, db_path) = O("login_empty_inputs");
    for f in [
        LoginForm {
            user: String::new(),
            pass: "A".to_string(),
            db_path: db_path.clone(), 
        },
        LoginForm {
            user: "A".to_string(),
            pass: String::new(),
            db_path: db_path.clone(),
        },
    ]
        .into_iter()
    {
        let app = U(|r| r);
        let srv = TestServer::new(app);
        let res = srv.post("/act/login").form(&f).await;

        println!("{}", res.text());
        assert!(res.text().contains("invalid form parameters"));
    }
}

#[tokio::test]
async fn login() {
    let (db, db_path) = O("login");

    let u = db.new_user("a", "a").unwrap();
    println!("{u}");

    let app = U(|r| r);
    let srv = TestServer::new(app);
    let res = srv.post("/act/login").form(&LoginForm {
        user: "a".to_string(),
        pass: "a".to_string(),
        db_path,
    }).await;

    println!("{}", res.text());
    assert!(!res.text().contains("user not found"));
}
