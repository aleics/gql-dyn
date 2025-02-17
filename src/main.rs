use std::{
    collections::HashSet,
    iter::FromIterator,
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use axum::{
    routing::{get, post},
    Extension, Router,
};
use config::{AnimalKind, ConfigProvider};
use data::Database;
use juniper_axum::{extract::JuniperRequest, graphiql, response::JuniperResponse};
use lazy_static::lazy_static;
use schema::SchemaGenerator;
use tokio::net::TcpListener;
use tower_http::compression::CompressionLayer;

mod config;
mod data;
mod fixtures;
mod schema;

lazy_static! {
    static ref ANIMAL_KINDS: HashSet<AnimalKind> = HashSet::from_iter(["Cat", "Dog", "Elephant"]);
}

impl juniper::Context for Database {}

async fn graphql(
    Extension(database): Extension<Database>,
    JuniperRequest(request): JuniperRequest,
) -> JuniperResponse {
    // Read the current configuration of the API, and build the schema.
    // This allows us to define a request-dependent schema (e.g. user client)
    let schema = SchemaGenerator::new()
        .with_config(ConfigProvider::generate())
        .generate();

    JuniperResponse(request.execute(&schema, &database).await)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let animals = fixtures::generate_animals(&ANIMAL_KINDS, 100);

    let database = Database {
        animals: Arc::new(RwLock::new(animals)),
    };

    let app = Router::new()
        .route("/graphql", get(graphql))
        .route("/graphql", post(graphql))
        .route("/", get(graphiql("/graphql", None)))
        .layer(Extension(database))
        .layer(CompressionLayer::new().gzip(true));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("failed to listen on {addr}: {e}"));

    tracing::info!("listening on {addr}");
    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| panic!("failed to run `axum::serve`: {e}"));
}
