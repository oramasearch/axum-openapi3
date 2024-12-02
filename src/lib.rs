#![doc = include_str!("../README.md")]

use std::{
    ops::Deref,
    sync::{Mutex, MutexGuard},
};

use axum::Router;
use once_cell::sync::Lazy;
use serde::Serialize;
use utoipa::openapi::Paths;

#[cfg(feature = "derive")]
extern crate axum_openapi_derive3;
/// Derive macro available if axum-openapi3 is built with `features = ["derive"]`.
#[cfg(feature = "derive")]
pub use axum_openapi_derive3::endpoint;

/// Re-export utoipa. Used internally to generate the openapi spec from rust structs.
pub use utoipa;

/// Mutex to store the endpoints.
/// Don't use directly, use the `endpoint` macro instead.
pub static ENDPOINTS: std::sync::Mutex<Vec<utoipa::openapi::Paths>> = std::sync::Mutex::new(vec![]);

/// Add `add` method to `Router` to add routes also to the openapi spec.
pub trait AddRoute<S> {
    fn add(
        self,
        r: (
            &str,
            axum::routing::MethodRouter<S, std::convert::Infallible>,
        ),
    ) -> Self;
}

impl<S: std::clone::Clone + std::marker::Send + std::marker::Sync + 'static> AddRoute<S>
    for Router<S>
{
    fn add(
        self,
        r: (
            &str,
            axum::routing::MethodRouter<S, std::convert::Infallible>,
        ),
    ) -> Self {
        self.route(r.0, r.1)
    }
}

static OPENAPI_BUILT: Lazy<Mutex<Option<utoipa::openapi::OpenApi>>> =
    Lazy::new(|| Mutex::new(None));

/// Reset the openapi spec. Mostly used for testing.
pub fn reset_openapi() {
    let mut endpoints = ENDPOINTS.lock().unwrap();
    *endpoints = vec![];
    *OPENAPI_BUILT.lock().unwrap() = None;
}

/// Wrapper around the openapi spec.
/// This wrapper is used to serialize the openapi spec.
pub struct OpenApiWrapper<'a> {
    guard: MutexGuard<'a, Option<utoipa::openapi::OpenApi>>,
}
impl Deref for OpenApiWrapper<'_> {
    type Target = utoipa::openapi::OpenApi;

    fn deref(&self) -> &Self::Target {
        self.guard.as_ref().unwrap()
    }
}

impl Serialize for OpenApiWrapper<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.guard.as_ref().unwrap().serialize(serializer)
    }
}

/// Build the openapi spec.
/// This function should be called after all the endpoints are defined.
/// Because the openapi spec is cached, it's cheap to call this function multiple times.
/// The `f` function is called only when the openapi spec is not built yet.
pub fn build_openapi<'openapi, F>(f: F) -> OpenApiWrapper<'openapi>
where
    F: Fn() -> utoipa::openapi::OpenApiBuilder,
{
    let mut openapi = OPENAPI_BUILT.lock().unwrap();
    if openapi.is_none() {
        let mut endpoints = ENDPOINTS.lock().unwrap();

        let paths = endpoints.drain(..).fold(Paths::default(), |mut acc, x| {
            acc.merge(x);
            acc
        });
        let openapi_builder = f().paths(paths);

        *openapi = Some(openapi_builder.build());
    }

    OpenApiWrapper { guard: openapi }
}
