use axum::{Router, routing::post};
use axum_cookie::CookieLayer;
use axum_test::TestServer;
use cookie::Cookie;

use crate::{
    db::Db,
    routes::{self, act::LoginForm, admin::UnhideForm},
};
use std::fs;

fn O(p: &str) -> (Db, Option<String>) {
    let p = "run/test/act/".to_owned() + p + ".db";

    if fs::exists(&p).unwrap() {
        fs::remove_file(&p).unwrap();
    }

    (Db::create(p.clone()).unwrap().init().unwrap(), Some(p))
}

fn U<F>(f: F) -> Router
where
    F: Fn(Router) -> Router,
{
    f(Router::new()
        .route("/act/login", post(routes::act::login)))
        .route("/admin/unhide", post(routes::admin::unhide))
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

#[tokio::test]
async fn unhide() {
    let (db, db_path) = O("unhide");

    let b = db.new_board("a", "a").unwrap();
    let u = db.new_user("a", "a").unwrap();
    let t = db.new_thread(b.id, u.id, "a", "a", None).unwrap();
    let p = db.new_post(t.id, u.id, "a", None).unwrap();
    let s = db.new_session(u.id).unwrap();
    let c = Cookie::new("session", s.hash);

    db.new_admin(u.id).unwrap();

    let app = U(|r| r);
    let srv = TestServer::new(app);

    let UH = async |id, ty, f: fn(String) -> bool| {
        let res = srv
            .post("/admin/unhide")
            .form(&UnhideForm {
                id,
                ty,
                goto: None,
                db_path,
            })
            .add_cookie(c)
            .await;

        println!("{}", res.text());
        assert!(!res.text().contains("not an admin"));
        assert!(!res.text().contains("not logged in"));
        assert!(f(res.text()));
    };

    /* valids */
    UH.clone()(b.id, 'b', |_| true).await;
    UH.clone()(t.id, 't', |_| true).await;
    UH.clone()(p.id, 'p', |_| true).await;

    /* invalids: not found */
    UH.clone()(0, 'b', |t| t.contains("not found")).await;
    UH.clone()(0, 't', |t| t.contains("not found")).await;
    UH.clone()(0, 'p', |t| t.contains("not found")).await;

    /* invalid type */
    UH.clone()(b.id, 'x', |t| t.contains("invalid form type")).await;
}
