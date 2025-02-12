use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{
    Extension, Router,
    routing::{MethodFilter, get, on},
};
use juniper::{
    EmptyMutation, EmptySubscription, GraphQLType, GraphQLValue, GraphQLValueAsync, Registry,
    RootNode, ScalarValue, meta::MetaType,
};
use juniper_axum::{graphiql, graphql};
use tokio::net::TcpListener;

#[derive(Clone)]
enum FieldType {
    String,
    Number,
}

struct Animal;

struct AnimalLike;

#[derive(Clone)]
struct AnimalConfig {
    name: &'static str,
    fields: HashMap<&'static str, FieldType>,
}

struct AnimalLikeConfig {
    current: AnimalConfig,
    config: SharedQueryConfig,
}

struct QueryConfig {
    animals: Vec<AnimalConfig>,
}

type SharedQueryConfig = Arc<QueryConfig>;

struct QueryRoot;

impl<S> GraphQLType<S> for Animal
where
    S: ScalarValue,
{
    fn name(_: &SharedQueryConfig) -> Option<&'static str> {
        Some("Animal")
    }

    fn meta<'r>(i: &SharedQueryConfig, registry: &mut Registry<'r, S>) -> MetaType<'r, S>
    where
        S: 'r,
    {
        for animal in &i.animals {
            let config = AnimalLikeConfig {
                current: animal.clone(),
                config: i.clone(),
            };
            let _ = registry.get_type::<AnimalLike>(&config);
        }

        let fields = &[registry.field::<Option<String>>("name", &())];
        registry.build_interface_type::<Self>(i, fields).into_meta()
    }
}

impl<S> GraphQLValue<S> for Animal
where
    S: ScalarValue,
{
    type Context = ();
    type TypeInfo = SharedQueryConfig;

    fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
        <Self as GraphQLType>::name(info)
    }
}

impl<S> GraphQLType<S> for AnimalLike
where
    S: ScalarValue,
{
    fn name(i: &Self::TypeInfo) -> Option<&'static str> {
        Some(i.current.name)
    }

    fn meta<'r>(i: &Self::TypeInfo, registry: &mut Registry<'r, S>) -> MetaType<'r, S>
    where
        S: 'r,
    {
        let _ = registry.get_type::<AnimalLike>(i);

        let mut fields = vec![registry.field::<Option<String>>("name", &())];

        for (field_name, field_type) in &i.current.fields {
            let custom_field = match field_type {
                FieldType::String => registry.field::<String>(field_name, &()),
                FieldType::Number => registry.field::<i32>(field_name, &()),
            };

            fields.push(custom_field);
        }

        registry
            .build_object_type::<Self>(i, fields.as_slice())
            .interfaces(&[registry.get_type::<Animal>(&i.config)])
            .into_meta()
    }
}

impl<S> GraphQLValue<S> for AnimalLike
where
    S: ScalarValue,
{
    type Context = ();
    type TypeInfo = AnimalLikeConfig;

    fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
        <Self as GraphQLType>::name(info)
    }
}

impl<S> GraphQLType<S> for QueryRoot
where
    S: ScalarValue,
{
    fn name(_: &SharedQueryConfig) -> Option<&'static str> {
        Some("Query")
    }

    fn meta<'r>(i: &SharedQueryConfig, registry: &mut Registry<'r, S>) -> MetaType<'r, S>
    where
        S: 'r,
    {
        let fields = &[registry.field::<Option<Animal>>("animal", i)];

        registry.build_object_type::<Self>(i, fields).into_meta()
    }
}

impl<S> GraphQLValue<S> for QueryRoot
where
    S: ScalarValue,
{
    type Context = ();
    type TypeInfo = SharedQueryConfig;

    fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
        <Self as GraphQLType>::name(info)
    }
}

impl<S> GraphQLValueAsync<S> for QueryRoot where S: ScalarValue + Send + Sync {}

type Schema = RootNode<'static, QueryRoot, EmptyMutation, EmptySubscription>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let animals = vec![
        AnimalConfig {
            name: "Cat",
            fields: HashMap::from_iter([("flur", FieldType::String)]),
        },
        AnimalConfig {
            name: "Dog",
            fields: HashMap::from_iter([("breed", FieldType::String)]),
        },
        AnimalConfig {
            name: "Elephant",
            fields: HashMap::from_iter([("age", FieldType::Number)]),
        },
    ];

    // How to provide QueryConfig to the query root
    let schema = Schema::new_with_info(
        QueryRoot,
        EmptyMutation::new(),
        EmptySubscription::new(),
        Arc::new(QueryConfig { animals }),
        (),
        (),
    );

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
