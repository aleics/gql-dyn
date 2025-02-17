use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use axum::{
    Extension, Router,
    routing::{get, post},
};
use config::{AnimalConfig, AnimalKind, ConfigProvider, FieldType};
use data::{Animal, AnimalLike, Database, FieldValue};
use juniper::{
    Arguments, EmptyMutation, EmptySubscription, ExecutionResult, Executor, GraphQLType,
    GraphQLValue, GraphQLValueAsync, Registry, RootNode, ScalarValue, Selection, meta::MetaType,
};
use juniper_axum::{extract::JuniperRequest, graphiql, response::JuniperResponse};
use lazy_static::lazy_static;
use tokio::net::TcpListener;
use tower_http::compression::CompressionLayer;

mod config;
mod data;
mod fixtures;

lazy_static! {
    static ref ANIMAL_KINDS: HashSet<AnimalKind> = HashSet::from_iter(["Cat", "Dog", "Elephant"]);
}

#[derive(Debug)]
struct AnimalLikeConfig {
    current: AnimalConfig,
    config: SharedQueryConfig,
}

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
        for animal in i.animals.values() {
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

    fn resolve_field(
        &self,
        _: &Self::TypeInfo,
        field_name: &str,
        _: &Arguments<S>,
        executor: &Executor<Self::Context, S>,
    ) -> ExecutionResult<S> {
        match field_name {
            "name" => executor.resolve(&(), &self.name),
            _ => panic!("Unknown field {}", field_name),
        }
    }

    fn resolve_into_type(
        &self,
        info: &Self::TypeInfo,
        type_name: &str,
        _selection_set: Option<&[Selection<S>]>,
        executor: &Executor<Self::Context, S>,
    ) -> ExecutionResult<S> {
        let current = info.animals.get(type_name).expect("Type not found");

        executor.resolve_with_ctx(
            &AnimalLikeConfig {
                current: current.clone(),
                config: info.clone(),
            },
            &AnimalLike { data: self },
        )
    }

    fn concrete_type_name(&self, _: &Self::Context, _: &Self::TypeInfo) -> String {
        self.kind.to_string()
    }
}

impl<S> GraphQLType<S> for AnimalLike<'_>
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

impl<S> GraphQLValue<S> for AnimalLike<'_>
where
    S: ScalarValue,
{
    type Context = ();
    type TypeInfo = AnimalLikeConfig;

    fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
        <Self as GraphQLType>::name(info)
    }

    fn resolve_field(
        &self,
        _: &Self::TypeInfo,
        field_name: &str,
        _: &Arguments<S>,
        executor: &Executor<Self::Context, S>,
    ) -> ExecutionResult<S> {
        match field_name {
            "name" => executor.resolve(&(), &self.data.name),
            _ => {
                let field_value = self.data.fields.get(field_name);
                match field_value {
                    Some(FieldValue::String(value)) => executor.resolve(&(), value),
                    Some(FieldValue::Number(value)) => executor.resolve(&(), value),
                    None => ExecutionResult::Ok(juniper::Value::Null),
                }
            }
        }
    }
}

type SharedQueryConfig = Arc<QueryConfig>;

#[derive(Debug)]
struct QueryConfig {
    animals: HashMap<AnimalKind, AnimalConfig>,
}

struct QueryRoot;

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
        let fields = &[registry.field::<Vec<Animal>>("animals", i)];
        registry.build_object_type::<Self>(i, fields).into_meta()
    }
}

impl<S> GraphQLValue<S> for QueryRoot
where
    S: ScalarValue,
{
    type Context = Database;
    type TypeInfo = SharedQueryConfig;

    fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
        <Self as GraphQLType>::name(info)
    }

    fn resolve_field(
        &self,
        info: &Self::TypeInfo,
        field_name: &str,
        _: &Arguments<S>,
        executor: &Executor<Self::Context, S>,
    ) -> ExecutionResult<S> {
        let database = executor.context();
        match field_name {
            "animals" => {
                let animals = database.animals.read().unwrap();
                executor.resolve_with_ctx(info, &animals.iter().collect::<Vec<_>>())
            }
            _ => ExecutionResult::Ok(juniper::Value::Null),
        }
    }
}

impl<S> GraphQLValueAsync<S> for QueryRoot
where
    S: ScalarValue + Send + Sync,
{
    fn resolve_field_async<'a>(
        &'a self,
        info: &'a Self::TypeInfo,
        field_name: &'a str,
        arguments: &'a Arguments<S>,
        executor: &'a Executor<Self::Context, S>,
    ) -> juniper::BoxFuture<'a, ExecutionResult<S>> {
        let value = self.resolve_field(info, field_name, arguments, executor);
        Box::pin(futures::future::ready(value))
    }
}

type Schema = RootNode<'static, QueryRoot, EmptyMutation<Database>, EmptySubscription<Database>>;

impl juniper::Context for Database {}

async fn graphql(
    Extension(database): Extension<Database>,
    JuniperRequest(request): JuniperRequest,
) -> JuniperResponse {
    let schema = Schema::new_with_info(
        QueryRoot,
        EmptyMutation::new(),
        EmptySubscription::new(),
        Arc::new(QueryConfig {
            animals: ConfigProvider::generate(),
        }),
        (),
        (),
    );

    JuniperResponse(request.execute(&schema, &database).await)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let animals = fixtures::generate_animals(&ANIMAL_KINDS, 10_000);

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
