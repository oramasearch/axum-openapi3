use std::net::{IpAddr, SocketAddr};

use axum::extract::{Path, Query};
use axum::response::IntoResponse;
use axum::Json;
use axum_openapi3::utoipa; // Needed for ToSchema and IntoParams derive
use axum_openapi3::utoipa::openapi::{InfoBuilder, OpenApiBuilder};
use axum_openapi3::utoipa::*; // Needed for ToSchema and IntoParams derive
use serde::{Deserialize, Serialize};

use axum_openapi3::{
    build_openapi, // function for building the openapi spec
    endpoint,      // macro for defining endpoints
    reset_openapi, // function for cleaning the openapi cache (mostly used for testing)
    AddRoute,      // `add` method for Router to add routes also to the openapi spec
};

#[derive(Serialize, Deserialize, ToSchema)]
struct Todo {
    id: u64,
    title: String,
    completed: bool,
}
#[derive(Serialize, Deserialize, IntoParams)]
struct TodoFilter {
    completed: bool,
}

#[endpoint(method = "POST", path = "/todos", description = "Insert a new todo")]
async fn insert_todo(_: Json<Todo>) -> Json<Todo> {
    unreachable!("")
}
#[endpoint(
    method = "PATCH",
    path = "/todos/:id/complete",
    description = "Mark a todo as completed"
)]
async fn mark_todo_as_complete(_: Path<u64>, _: Json<u64>) -> Json<Todo> {
    unreachable!("")
}
#[endpoint(
    method = "GET",
    path = "/todos/filter",
    description = "Filter todos by completed status"
)]
async fn filter(_: Query<TodoFilter>) -> Json<Vec<Todo>> {
    unreachable!("")
}

fn get_router() -> axum::Router {
    axum::Router::new()
        .add(insert_todo())
        .add(mark_todo_as_complete())
        .add(filter())
        .add(openapi())
}

#[endpoint(method = "GET", path = "/openapi.json", description = "OpenAPI spec")]
async fn openapi() -> impl IntoResponse {
    // `build_openapi` caches the openapi spec, so it's not necessary to call it every time
    let openapi = build_openapi(|| {
        OpenApiBuilder::new().info(InfoBuilder::new().title("My Webserver").version("0.1.0"))
    });

    Json(openapi)
}

#[tokio::main]
async fn main() {
    reset_openapi(); // clean the openapi cache. Mostly used for testing

    let ip: IpAddr = "127.0.0.1".parse().unwrap();
    let _addr = SocketAddr::new(ip, 8080);

    let _router = get_router();

    // De-comment the following lines to start the server

    // let listener = tokio::net::TcpListener::bind(addr)
    //     .await
    //     .expect("failed to bind address");
    // println!("Address binded. Starting web server on http://{}", addr);
    // axum::serve(listener, router).await.expect("server failed");
}
