use std::collections::HashMap;

pub(crate) type AnimalKind = &'static str;

#[derive(Clone, Debug)]
pub(crate) enum FieldType {
    String,
    Number,
}

#[derive(Clone, Debug)]
pub(crate) struct AnimalConfig {
    pub(crate) name: AnimalKind,
    pub(crate) fields: HashMap<&'static str, FieldType>,
}
