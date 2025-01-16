use axum::{
    extract::{Path, Query},
    response::IntoResponse,
    Json, Router,
};
use axum_openapi3::utoipa::{
    openapi::{Paths, RefOr, Schema},
    IntoParams, PartialSchema, ToSchema,
};
use axum_openapi3::*;
use serde::{Deserialize, Serialize};
use utoipa::openapi::{
    path::{Parameter, ParameterBuilder, ParameterIn},
    OpenApiBuilder, Required,
};

pub fn get_router() -> Router {
    let mut router = Router::new();

    router = router
        .add(static_str())
        .add(get_list_string())
        .add(add_string())
        .add(get_todos())
        .add(insert_todo())
        .add(mark_todo_as_complete())
        .add(filter())
        .add(get_todo())
        .add(mark_todo_as())
        .add(generic());

    router
}

#[endpoint(method = "GET", path = "/", description = "Welcome")]
async fn static_str() -> Json<&'static str> {
    Json("static str")
}

#[endpoint(method = "GET", path = "/strings", description = "List all string")]
async fn get_list_string() -> Json<Vec<String>> {
    unreachable!("")
}

#[endpoint(method = "POST", path = "/strings", description = "Insert new string")]
async fn add_string(Json(_): Json<String>) -> Json<String> {
    unreachable!("")
}

#[derive(Serialize, Deserialize, ToSchema)]
struct Todo {
    id: u64,
    title: String,
    completed: bool,
}
#[endpoint(method = "GET", path = "/todos", description = "List all todos")]
async fn get_todos() -> Json<Vec<Todo>> {
    unreachable!("")
}
#[endpoint(method = "POST", path = "/todos", description = "Insert a new todo")]
async fn insert_todo(_: Json<Todo>) -> Json<Todo> {
    unreachable!("")
}
#[endpoint(
    method = "PATCH",
    path = "/todos",
    description = "Mark a todo as completed"
)]
async fn mark_todo_as_complete(_: Json<u64>) -> Json<Todo> {
    unreachable!("")
}

#[derive(Serialize, Deserialize, IntoParams)]
struct TodoFilter {
    completed: bool,
}
#[endpoint(
    method = "GET",
    path = "/todos/filter",
    description = "Filter todos by completed status"
)]
async fn filter(_: Query<TodoFilter>) -> Json<Vec<Todo>> {
    unreachable!("")
}

#[endpoint(method = "GET", path = "/generic", description = "Generic endpoint")]
#[allow(dependency_on_unit_never_type_fallback)]
async fn generic() -> impl IntoResponse {
    unreachable!("")
}

#[endpoint(method = "GET", path = "/todos/{id}", description = "Get todo by id")]
async fn get_todo(Path(_): Path<u64>) -> Json<Todo> {
    unreachable!("")
}
#[endpoint(
    method = "PATCH",
    path = "/todos/{id}/complete",
    description = "Mark a todo as ..."
)]
async fn mark_todo_as(_: Path<u64>, _: Json<bool>) -> Json<Todo> {
    unreachable!("")
}

#[test]
fn test_all() {
    reset_openapi();
    _ = get_router();

    let openapi = build_openapi(OpenApiBuilder::new);

    let paths = &openapi.paths;

    assert_endpoint(
        paths,
        "/",
        "get",
        "static_str",
        "Welcome",
        Some(<&'static str as PartialSchema>::schema()),
        None,
        None,
        None,
    );

    assert_endpoint(
        paths,
        "/strings",
        "get",
        "get_list_string",
        "List all string",
        Some(Vec::<String>::schema()),
        None,
        None,
        None,
    );

    assert_endpoint(
        paths,
        "/strings",
        "post",
        "add_string",
        "Insert new string",
        Some(String::schema()),
        Some(String::schema()),
        None,
        None,
    );

    assert_endpoint(
        paths,
        "/todos",
        "get",
        "get_todos",
        "List all todos",
        Some(Vec::<Todo>::schema()),
        None,
        None,
        None,
    );

    assert_endpoint(
        paths,
        "/todos",
        "post",
        "insert_todo",
        "Insert a new todo",
        Some(Todo::schema()),
        Some(Todo::schema()),
        None,
        None,
    );

    assert_endpoint(
        paths,
        "/todos",
        "patch",
        "mark_todo_as_complete",
        "Mark a todo as completed",
        Some(Todo::schema()),
        Some(u64::schema()),
        None,
        None,
    );

    assert_endpoint(
        paths,
        "/todos/filter",
        "get",
        "filter",
        "Filter todos by completed status",
        Some(Vec::<Todo>::schema()),
        None,
        Some(TodoFilter::into_params(|| Some(ParameterIn::Query))),
        None,
    );

    assert_endpoint(
        paths,
        "/todos/{id}",
        "get",
        "get_todo",
        "Get todo by id",
        Some(Todo::schema()),
        None,
        None,
        Some(vec![ParameterBuilder::new()
            .parameter_in(ParameterIn::Path)
            .name("id")
            .required(Required::True)
            .schema(Some(u64::schema()))
            .build()]),
    );

    assert_endpoint(
        paths,
        "/generic",
        "get",
        "generic",
        "Generic endpoint",
        None,
        None,
        None,
        None,
    );
}

#[allow(clippy::too_many_arguments)]
fn assert_endpoint(
    paths: &Paths,
    path: &'static str,
    method: &'static str,
    operation_id: &'static str,
    description: &'static str,
    expected_schema: Option<RefOr<Schema>>,
    expected_request_body: Option<RefOr<Schema>>,
    expected_query_parameters: Option<Vec<Parameter>>,
    expected_path_params: Option<Vec<Parameter>>,
) {
    let path_item = &paths.paths[path];

    let operation = match method {
        "get" => path_item.get.as_ref().unwrap(),
        "post" => path_item.post.as_ref().unwrap(),
        "patch" => path_item.patch.as_ref().unwrap(),
        _ => panic!("Unsupported method"),
    };

    assert_eq!(operation.operation_id, Some(operation_id.to_string()));
    assert_eq!(operation.description, Some(description.to_string()));

    if let Some(expected_schema) = expected_schema {
        let response = resolve_as_t(&operation.responses.responses["200"]);
        let schema = response.content["application/json"]
            .schema
            .as_ref()
            .unwrap();
        assert_eq!(schema, &expected_schema);
    }

    if let Some(expected_request_body) = expected_request_body {
        let request_body = operation.request_body.as_ref().unwrap();
        let request_body = request_body.content["application/json"]
            .schema
            .as_ref()
            .unwrap();
        assert_eq!(request_body, &expected_request_body);
    } else {
        assert!(operation.request_body.is_none());
    }

    if let Some(par) = &operation.parameters {
        let query_params: Vec<_> = par
            .iter()
            .filter(|p| p.parameter_in == ParameterIn::Query)
            .cloned()
            .collect();
        assert_eq!(query_params, expected_query_parameters.unwrap_or_default());
    } else {
        assert!(expected_query_parameters.is_none());
    }

    if let Some(par) = &operation.parameters {
        let path_params: Vec<_> = par
            .iter()
            .filter(|p| p.parameter_in == ParameterIn::Path)
            .cloned()
            .collect();
        assert_eq!(path_params, expected_path_params.unwrap_or_default());
    } else {
        assert!(expected_path_params.is_none());
    }
}

fn resolve_as_t<Inner>(ref_or: &RefOr<Inner>) -> &Inner {
    match ref_or {
        RefOr::T(inner) => inner,
        RefOr::Ref(_) => panic!("Reference not supported"),
    }
}
