//! This example demonstrates simple default integration with [`axum`].

use std::{net::SocketAddr, sync::Arc};

use axum::{
    Extension, Router,
    routing::{MethodFilter, get, on},
};
use juniper::{EmptyMutation, EmptySubscription, RootNode, graphql_interface, graphql_object};
use juniper_axum::{graphiql, graphql};
use tokio::net::TcpListener;

#[derive(Clone, Copy, Debug)]
pub struct QueryRoot;

#[graphql_object]
impl QueryRoot {
    fn animals() -> Vec<AnimalValue> {
        vec![AnimalValueEnum::Dog(Dog {}), AnimalValueEnum::Cat(Cat {})]
    }
}

#[graphql_interface(for = [Dog, Cat])]
trait Animal {
    fn name(&self) -> &str;
}

struct Dog;

#[graphql_object]
#[graphql(impl = [AnimalValue])]
impl Dog {
    fn name(&self) -> &str {
        "dog"
    }
}

struct Cat;

#[graphql_object]
#[graphql(impl = [AnimalValue])]
impl Cat {
    fn name(&self) -> &str {
        "cat"
    }
}

type Schema = RootNode<'static, QueryRoot, EmptyMutation, EmptySubscription>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let schema = Schema::new(QueryRoot, EmptyMutation::new(), EmptySubscription::new());

    let app = Router::new()
        .route(
            "/graphql",
            on(
                MethodFilter::GET.or(MethodFilter::POST),
                graphql::<Arc<Schema>>,
            ),
        )
        .route("/", get(graphiql("/graphql", None)))
        .layer(Extension(Arc::new(schema)));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("failed to listen on {addr}: {e}"));

    tracing::info!("listening on {addr}");
    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| panic!("failed to run `axum::serve`: {e}"));
}
