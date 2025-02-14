use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::AnimalKind;

#[derive(Debug, Clone)]
pub(crate) enum FieldValue {
    String(String),
    Number(i32),
}

#[derive(Debug)]
pub(crate) struct Animal {
    pub(crate) name: String,
    pub(crate) kind: AnimalKind,
    pub(crate) fields: HashMap<&'static str, FieldValue>,
}

#[derive(Debug)]
pub(crate) struct AnimalLike<'a> {
    pub(crate) data: &'a Animal,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct Database {
    pub(crate) animals: Arc<RwLock<Vec<Animal>>>,
}
