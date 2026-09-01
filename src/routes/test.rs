use axum::{Router, routing::post};
use axum_cookie::prelude::*;
use axum_test::{TestRequest, TestServer};

use crate::{
    db::Db,
    pre::*,
    routes::{self, act::LoginForm},
};
use std::fs;

fn O(p: &str) -> Db {
    let p = "run/test/act/".to_owned() + p + ".db";

    if fs::exists(&p).unwrap() {
        fs::remove_file(&p).unwrap();
    }

    set_db_path(p.clone()).unwrap();

    Db::create(p).unwrap().init().unwrap()
}

fn U<F>(f: F) -> Router
where
    F: Fn(Router) -> Router,
{
    f(Router::new().route("/act/login", post(routes::act::login)))
        .layer(CookieLayer::default())
}

fn login_test<D, E>(db_p: &str, d: D, e: E) -> TestRequest
where
    D: Fn(&Db),
    E: Fn(TestServer) -> TestRequest,
{
    let db = O(db_p);

    d(&db);

    let app = U(|r| r);
    let srv = TestServer::new(app);

    e(srv)
}

#[tokio::test]
async fn login_user_not_found() {
    let db = O("login_user_not_found");

    let app = U(|r| r);
    let srv = TestServer::new(app);
    let res = srv.post("/act/login").form(&LoginForm {
        user: "A".to_string(),
        pass: "A".to_string(),
    }).await;

    assert!(res.text().contains("user not found"));
}

#[tokio::test]
async fn login_empty_inputs() {
    for f in [
        LoginForm {
            user: String::new(),
            pass: "A".to_string(),
        },
        LoginForm {
            user: "A".to_string(),
            pass: String::new(),
        },
    ]
        .into_iter()
    {
        let db = O("login_empty_inputs");
        let app = U(|r| r);
        let srv = TestServer::new(app);
        let res = srv.post("/act/login").form(&f).await;

        println!("{}", res.text());
        assert!(res.text().contains("invalid form parameters"));
    }
}
