use axum::{
    Router,
    routing::{get, post},
    extract::DefaultBodyLimit,
};
use axum_cookie::prelude::*;
use hospital::{pre::*, routes};
use tokio::net::TcpListener;

use hospital::db::Db;

#[tokio::main]
async fn main() -> R<()> {
    let cfg = &*CFG;
    let addr = format!("{}:{}", cfg.server.ip, cfg.server.port);

    /* create a database just to run the base CREATE commands */
    let _ = Db::new()?.init()?;

    /* create the routes */
    let app = Router::new()
        .route("/login", get(routes::login))
        .route("/act/login", post(routes::act::login))
        .route("/act/new-thread", post(routes::act::new_thread))
        .route("/act/new-post", post(routes::act::new_post))
        .route("/act/update-user", post(routes::act::update_user))
        .route("/act/hide-thread", post(routes::act::hide_thread))
        .route("/act/lock-thread", post(routes::act::lock_thread))
        .route("/act/hide-post", post(routes::act::hide_post))
        .route("/dbg/user/{id}", get(routes::dbg::user))
        .route("/b/{name}", get(routes::b::by_name))
        .route("/t/{id}", get(routes::t::by_id))
        .route("/i/{id}", get(routes::i::by_id))
        .route("/u/{name}", get(routes::u::by_name))
        .route("/", get(routes::index))
        .layer(DefaultBodyLimit::max(1073742000))
        .layer(CookieLayer::default());

    /* bind to the address */
    let listener = match TcpListener::bind(&addr).await {
        Ok(x) => x,
        Err(e) => fatal!("{e}"),
    };

    /* listen */
    println!("listening on {addr}");
    match axum::serve(listener, app).await {
        Ok(()) => (),
        Err(e) => fatal!("{e}"),
    };

    Ok(())
}
