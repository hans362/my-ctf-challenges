use salvo::prelude::*;
use salvo::session::{CookieStore, SessionHandler};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
mod controller;
mod model;
mod router;
mod service;

#[tokio::main]
async fn main() {
    std::fs::create_dir_all("data").unwrap();
    tracing_subscriber::fmt().init();
    let secret_key = std::fs::read(".secretkey").unwrap_or_else(|_| {
        eprintln!("Error: Could not read secret key from .secretkey. Please create the file with a secret key.");
        std::process::exit(1);
    });
    if secret_key.len() < 128 {
        eprintln!("Error: Secret key must be at least 128 bytes long.");
        std::process::exit(1);
    }
    let session_handler = SessionHandler::builder(CookieStore::new(), &secret_key)
        .build()
        .unwrap();
    let acceptor = TcpListener::new("0.0.0.0:5800").bind().await;
    let mut accounts: HashMap<String, String> = HashMap::new();
    accounts.insert("admin".to_string(), uuid::Uuid::new_v4().to_string());
    let router = Router::new()
        .hoop(affix_state::inject(Arc::new(RwLock::new(accounts))))
        .hoop(session_handler)
        .push(router::api_router())
        .push(router::preview_router());
    println!("{:?}", router);
    Server::new(acceptor).serve(router).await;
}
