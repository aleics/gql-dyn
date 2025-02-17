use std::collections::HashMap;

pub(crate) type AnimalKind = &'static str;

#[derive(Clone, Debug)]
pub(crate) enum FieldType {
    String,
    Number,
}

#[derive(Clone, Debug)]
pub(crate) struct AnimalConfig {
    pub(crate) kind: AnimalKind,
    pub(crate) fields: HashMap<&'static str, FieldType>,
}

#[derive(Default)]
pub(crate) struct ConfigProvider;

impl ConfigProvider {
    pub(crate) fn generate() -> HashMap<&'static str, AnimalConfig> {
        let mut cat_config = AnimalConfig {
            kind: "Cat",
            fields: HashMap::new(),
        };

        cat_config.fields.insert("fur", FieldType::String);

        let mut dog_config = AnimalConfig {
            kind: "Dog",
            fields: HashMap::new(),
        };

        dog_config.fields.insert("breed", FieldType::String);

        let mut elephant_config = AnimalConfig {
            kind: "Elephant",
            fields: HashMap::new(),
        };

        elephant_config.fields.insert("age", FieldType::Number);

        let mut config = HashMap::new();

        config.insert(cat_config.kind, cat_config);
        config.insert(dog_config.kind, dog_config);
        config.insert(elephant_config.kind, elephant_config);

        config
    }
}
