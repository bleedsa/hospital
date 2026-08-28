use axum::{
    Router,
    routing::{get, post},
};
use hospital::{pre::*, routes};
use tokio::net::TcpListener;

use hospital::db::Db;

#[tokio::main]
async fn main() {
    let cfg = &*CFG;
    let addr = format!("{}:{}", cfg.server.ip, cfg.server.port);

    /* create a database just to run the base CREATE commands */
    let _ = match Db::new() {
        Ok(_) => (),
        Err(e) => fatal!("{e}"),
    };

    /* create the routes */
    let app = Router::new()
        .route("/login", get(routes::login))
        .route("/act/login", post(routes::act::login))
        .route("/dbg/user/{id}", get(routes::dbg::user))
        .route("/", get(routes::index));

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
}
