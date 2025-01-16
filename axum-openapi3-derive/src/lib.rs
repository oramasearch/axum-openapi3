//! Derive macro for defining an endpoint.
//! See [`axum_openapi3`](https://crates.io/crates/axum-openapi3) for more information.

use handler_signature::{parse_handler_arguments, parse_handler_ret_type, HandlerArgument};
use macro_arguments::MacroArgs;
use quote::quote;
use std::fmt::Write;
use syn::{parse_macro_input, spanned::Spanned, ItemFn};

mod handler_signature;
mod macro_arguments;
mod util;

/// Derive macro for defining an endpoint.
#[proc_macro_attribute]
pub fn endpoint(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let macro_args = parse_macro_input!(args as MacroArgs);
    let input_fn = parse_macro_input!(input as ItemFn);

    let fn_args = match parse_handler_arguments(&input_fn.sig) {
        Ok(args) => args,
        Err(err) => return err.to_compile_error().into(),
    };

    let ret_type = match parse_handler_ret_type(&input_fn.sig) {
        Ok(ty) => ty,
        Err(err) => return err.to_compile_error().into(),
    };

    let fn_name = input_fn.sig.ident.clone();
    let fn_name_str = fn_name.to_string();

    let path = macro_args.path;
    let method = macro_args.method.to_string();
    let description = macro_args.description.unwrap_or_default();

    let method: http::Method = method.parse().unwrap(); //The HTTP method parsing fails before

    let (utoipa_method_name, axum_method) = match get_method_tokens(method) {
        Ok(value) => value,
        Err(_) => {
            return syn::Error::new(input_fn.sig.span(), "Unsupported HTTP method")
                .to_compile_error()
                .into()
        }
    };

    let ret_type = get_ret_type_token(ret_type);

    let request_body = get_request_body_token(&fn_args);

    let query_params = get_query_params_token(&fn_args);

    let path_param_names = extract_params(&path);
    let path_params = get_path_params_token(&fn_args, path_param_names);

    let state = get_state_token(fn_args);

    let path_for_openapi = transform_route(&path);

    let output = quote! {
        fn #fn_name() -> (&'static str, axum::routing::MethodRouter < #state , std::convert::Infallible >)
        {
            #input_fn

            let handler = axum::routing:: #axum_method (#fn_name);

            let op_builder = axum_openapi3::utoipa::openapi::path::OperationBuilder::new()
                .description(Some(#description));

            #ret_type

            #request_body

            #query_params

            #path_params

            let op_builder = op_builder.operation_id(Some(#fn_name_str));

            let paths = axum_openapi3::utoipa::openapi::PathsBuilder::new()
                .path(#path_for_openapi, axum_openapi3::utoipa::openapi::path::PathItemBuilder::new()
                    .operation(
                        axum_openapi3::utoipa::openapi::HttpMethod:: #utoipa_method_name,
                        op_builder.build()
                    )
                    .build())
                .build();

            axum_openapi3::ENDPOINTS.lock().unwrap().push(paths);

            (#path, handler)
        }

    };

    output.into()
}

fn get_ret_type_token(ret_type: Option<String>) -> proc_macro2::TokenStream {
    let ret_type = if let Some(ret_type) = ret_type {
        format!(
            r#"
let response_schema = < {ret_type} as axum_openapi3::utoipa::PartialSchema > :: schema();
let op_builder = op_builder.response(
    "200", 
    axum_openapi3::utoipa::openapi::ResponseBuilder::new()
        .content(
            "application/json", 
            axum_openapi3::utoipa::openapi::ContentBuilder::new()
                .schema(Some(response_schema))
                .build()
        )
        .build()
);
            "#
        )
    } else {
        "let op_builder = op_builder;".to_string()
    };
    let ret_type: proc_macro2::TokenStream = ret_type.parse().unwrap();
    ret_type
}

fn get_request_body_token(fn_args: &[HandlerArgument]) -> proc_macro2::TokenStream {
    let request_body = fn_args.iter().find_map(|arg| match arg {
        HandlerArgument::RequestBody(ty) => Some(format!(
            r#"
let request_body = < {ty} as axum_openapi3::utoipa::PartialSchema > :: schema();
let op_builder = op_builder
        .request_body(Some(
            axum_openapi3::utoipa::openapi::request_body::RequestBodyBuilder::new()
                .content(
                    "application/json",
                    axum_openapi3::utoipa::openapi::ContentBuilder::new()
                        .schema(Some(request_body))
                        .build()
                )
                .build()
        ));
            "#
        )),
        _ => None,
    });
    let request_body: proc_macro2::TokenStream = if let Some(request_body) = request_body {
        request_body.parse().unwrap()
    } else {
        "let op_builder = op_builder;".parse().unwrap()
    };
    request_body
}

fn get_state_token(fn_args: Vec<HandlerArgument>) -> proc_macro2::TokenStream {
    let state = fn_args
        .iter()
        .find_map(|arg| match arg {
            HandlerArgument::State(ty) => Some(ty.as_str()),
            _ => None,
        })
        .unwrap_or("()");
    let state: proc_macro2::TokenStream = state.parse().unwrap();
    state
}

fn get_path_params_token(
    fn_args: &[HandlerArgument],
    path_param_names: Vec<String>,
) -> proc_macro2::TokenStream {
    let path_params: String = fn_args
        .iter()
        .filter_map(|arg| match arg {
            HandlerArgument::Path(ty) => Some(ty),
            _ => None,
        })
        .zip(path_param_names.iter())
        .fold(String::new(), |mut acc, (ty, name)| {
            let _ = write!(
                acc,
                r#"
let schema = < {ty} as axum_openapi3::utoipa::PartialSchema > :: schema();
let path_param = axum_openapi3::utoipa::openapi::path::ParameterBuilder::new()
    .parameter_in(axum_openapi3::utoipa::openapi::path::ParameterIn::Path)
    .name("{name}")
    .required(axum_openapi3::utoipa::openapi::Required::True)
    .schema(Some(schema))
    .build();

let op_builder = op_builder
    .parameter(path_param);
"#
            );
            acc
        });
    let path_params: proc_macro2::TokenStream = if path_params.is_empty() {
        "let op_builder = op_builder;".parse().unwrap()
    } else {
        path_params.parse().unwrap()
    };
    path_params
}

fn get_query_params_token(fn_args: &[HandlerArgument]) -> proc_macro2::TokenStream {
    let query_params = fn_args.iter().find_map(|arg| {
        match arg {
            HandlerArgument::Query(ty) => Some(format!(r#"
let query_params = < {ty} as axum_openapi3::utoipa::IntoParams > :: into_params(|| Some(axum_openapi3::utoipa::openapi::path::ParameterIn::Query));
let op_builder = op_builder
    .parameters(Some(query_params));
            "#)),
            _ => None,
        }
    });
    let query_params: proc_macro2::TokenStream = if let Some(query_params) = query_params {
        query_params.parse().unwrap()
    } else {
        "let op_builder = op_builder;".parse().unwrap()
    };
    query_params
}

fn get_method_tokens(
    method: http::Method,
) -> Result<(proc_macro2::TokenStream, proc_macro2::TokenStream), ()> {
    let (utoipa_method_name, axum_method): (proc_macro2::TokenStream, proc_macro2::TokenStream) =
        match method {
            http::Method::GET => ("Get".parse().unwrap(), "get".parse().unwrap()),
            http::Method::POST => ("Post".parse().unwrap(), "post".parse().unwrap()),
            http::Method::PUT => ("Put".parse().unwrap(), "put".parse().unwrap()),
            http::Method::DELETE => ("Delete".parse().unwrap(), "delete".parse().unwrap()),
            http::Method::HEAD => ("Head".parse().unwrap(), "head".parse().unwrap()),
            http::Method::OPTIONS => ("Options".parse().unwrap(), "options".parse().unwrap()),
            http::Method::CONNECT => ("Connect".parse().unwrap(), "connect".parse().unwrap()),
            http::Method::PATCH => ("Patch".parse().unwrap(), "patch".parse().unwrap()),
            // Ensure the HTTP method is valid
            _ => return Err(()),
        };
    Ok((utoipa_method_name, axum_method))
}

fn extract_params(input: &str) -> Vec<String> {
    input
        .split('/')
        .filter_map(|segment| {
            if segment.starts_with('{') && segment.ends_with("}") {
                Some(
                    segment
                        .trim_start_matches('{')
                        .trim_end_matches("}")
                        .to_string(),
                )
            } else {
                None
            }
        })
        .collect()
}

fn transform_route(route: &str) -> String {
    route
        .split('/') // Split the route by '/'
        .map(|segment| {
            if let Some(stripped) = segment.strip_prefix(':') {
                format!("{{{}}}", stripped) // Replace ':id' with '{id}'
            } else {
                segment.to_string() // Keep other segments unchanged
            }
        })
        .collect::<Vec<_>>() // Collect transformed segments
        .join("/") // Rejoin segments into a single string
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_extract_params() {
        assert_eq!(super::extract_params("/foo/{id}/bar"), vec!["id"]);
        assert_eq!(
            super::extract_params("/foo/{id}/bar/{baz}"),
            vec!["id", "baz"]
        );
        assert_eq!(
            super::extract_params("/foo/{id}/bar/{baz}/"),
            vec!["id", "baz"]
        );
        assert_eq!(
            super::extract_params("/foo/{id}/bar/{baz}/{qux}"),
            vec!["id", "baz", "qux"]
        );
    }

    #[test]
    fn test_transform_route() {
        let routes = vec![
            ("/todos", "/todos"),
            ("/todos/:id", "/todos/{id}"),
            ("/todos/:id/foo", "/todos/{id}/foo"),
            ("/bar/:bar_id/foo/:foo_id", "/bar/{bar_id}/foo/{foo_id}"),
            (
                "/bar/{bar_id}/foo/{foo_id}/baz",
                "/bar/{bar_id}/foo/{foo_id}/baz",
            ),
        ];

        for (input, expected) in routes {
            assert_eq!(super::transform_route(input), expected);
        }
    }
}
