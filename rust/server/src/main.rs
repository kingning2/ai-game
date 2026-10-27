#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
mod error;
mod lifecycle;
mod openapi;
mod orchestr;
mod routes;

#[tokio::main]
async fn main() {
    lifecycle::run().await;
}
