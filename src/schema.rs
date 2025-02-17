use std::{collections::HashMap, sync::Arc};

use crate::config::{AnimalConfig, AnimalKind, FieldType};
use crate::data::{Animal, AnimalLike, Database, FieldValue};
use juniper::{
    meta::MetaType, Arguments, EmptyMutation, EmptySubscription, ExecutionResult, Executor,
    GraphQLType, GraphQLValue, GraphQLValueAsync, Registry, RootNode, ScalarValue, Selection,
};

const ANIMALS_QUERY_NAME: &str = "animals";
const ANIMAL_NAME_FIELD_NAME: &str = "name";

const ANIMAL_TYPE_NAME: &str = "Animal";

#[derive(Debug)]
pub(crate) struct AnimalLikeConfig {
    current: AnimalConfig,
    config: SharedQueryConfig,
}

impl<S> GraphQLType<S> for Animal
where
    S: ScalarValue,
{
    fn name(_: &SharedQueryConfig) -> Option<&'static str> {
        Some(ANIMAL_TYPE_NAME)
    }

    fn meta<'r>(i: &SharedQueryConfig, registry: &mut Registry<'r, S>) -> MetaType<'r, S>
    where
        S: 'r,
    {
        // Register all the animal implementations and provide the type
        // info for each dynamic `AnimalLike` type.
        for animal in i.animals.values() {
            let config = AnimalLikeConfig {
                current: animal.clone(),
                config: i.clone(),
            };
            let _ = registry.get_type::<AnimalLike>(&config);
        }

        let fields = &[registry.field::<Option<String>>(ANIMAL_NAME_FIELD_NAME, &())];
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
            ANIMAL_NAME_FIELD_NAME => executor.resolve(&(), &self.name),
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
        // Resolve the current interface's actual type and provide the expected
        // animal type info
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

        let mut fields = vec![registry.field::<Option<String>>(ANIMAL_NAME_FIELD_NAME, &())];

        // Generate and add dynamically each field specified in the configuration
        // to the current type.
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
            ANIMAL_NAME_FIELD_NAME => executor.resolve(&(), &self.data.name),
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
        let fields = &[registry.field::<Vec<Animal>>(ANIMALS_QUERY_NAME, i)];
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
            ANIMALS_QUERY_NAME => {
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

pub(crate) struct QueryRoot;

pub(crate) type Schema =
    RootNode<'static, QueryRoot, EmptyMutation<Database>, EmptySubscription<Database>>;

#[derive(Debug)]
pub(crate) struct QueryConfig {
    animals: HashMap<AnimalKind, AnimalConfig>,
}

#[derive(Default)]
pub(crate) struct SchemaGenerator {
    animals: HashMap<AnimalKind, AnimalConfig>,
}

impl SchemaGenerator {
    pub(crate) fn new() -> Self {
        SchemaGenerator::default()
    }

    pub(crate) fn with_config(&mut self, config: HashMap<AnimalKind, AnimalConfig>) -> &mut Self {
        self.animals = config;
        self
    }

    pub(crate) fn generate(&self) -> Schema {
        Schema::new_with_info(
            QueryRoot,
            EmptyMutation::new(),
            EmptySubscription::new(),
            Arc::new(QueryConfig {
                animals: self.animals.clone(),
            }),
            (),
            (),
        )
    }
}
