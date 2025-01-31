#![allow(dead_code)]

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use axum_openapi3::utoipa; // Needed for ToSchema and IntoParams derive
use axum_openapi3::utoipa::*; // Needed for ToSchema and IntoParams derive
use serde::Deserialize;

use axum_openapi3::{
    endpoint,      // macro for defining endpoints
    reset_openapi, // function for cleaning the openapi cache (mostly used for testing)
    AddRoute,      // `add` method for Router to add routes also to the openapi spec
};

struct MyState;

#[derive(Deserialize, IntoParams)]
struct QueryParams {
    #[serde(rename = "api-key")]
    api_key: String,
}
#[derive(Deserialize, ToSchema)]
struct MyJson {
    ids: Vec<u64>,
}

#[endpoint(method = "POST", path = "/query-and-json/{id}", description = "")]
async fn query_and_json(
    _: Path<String>,
    _: Query<QueryParams>,
    _: State<Arc<MyState>>,
    _: Json<MyJson>,
) -> Json<String> {
    unreachable!("");
}

fn get_router() -> axum::Router {
    axum::Router::new()
        .add(query_and_json())
        .with_state(Arc::new(MyState))
        .add(without_state())
}

#[endpoint(method = "GET", path = "/", description = "")]
async fn without_state() -> impl IntoResponse {
    Json("Welcome".to_string())
}

#[tokio::main]
async fn main() {
    reset_openapi(); // clean the openapi cache. Mostly used for testing

    let ip: IpAddr = "127.0.0.1".parse().unwrap();
    let addr = SocketAddr::new(ip, 8080);

    let router = get_router();

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind address");

    println!("Address binded. Starting web server on http://{}", addr);
    println!(
        "Open http://{}/openapi.json to see the generated OpenApi Spec",
        addr
    );
    axum::serve(listener, router).await.expect("server failed");
}
