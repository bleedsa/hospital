use axum::{
    routing::get,
    Router,
};
use tokio::net::TcpListener;
use hospital::{routes, pre::*};

#[tokio::main]
async fn main() {
    let cfg = &*CFG;
    let addr = format!("{}:{}", cfg.ip, cfg.port);

    /* create the routes */
    let app = Router::new()
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
