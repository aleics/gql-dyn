use std::collections::HashMap;

use crate::{Animal, AnimalConfig, AnimalKind, FieldValue};

pub(crate) fn generate_animals(
    config: &HashMap<AnimalKind, AnimalConfig>,
    amount: usize,
) -> Vec<Animal> {
    let mut result = Vec::with_capacity(amount * config.len());

    for kind in config.keys() {
        for i in 0..(amount / config.len()) {
            let animal = match *kind {
                "Dog" => dog(i, kind),
                "Cat" => cat(i, kind),
                "Elephant" => elephant(i, kind),
                _ => panic!("Unknown kind in config"),
            };
            result.push(animal);
        }
    }

    result
}

fn dog(i: usize, kind: AnimalKind) -> Animal {
    Animal {
        name: format!("{} {}", kind, i),
        kind,
        fields: HashMap::from_iter([("breed", FieldValue::String("Retriever".to_string()))]),
    }
}

fn cat(i: usize, kind: AnimalKind) -> Animal {
    Animal {
        name: format!("{} {}", kind, i),
        kind,
        fields: HashMap::from_iter([("fur", FieldValue::String("long".to_string()))]),
    }
}

fn elephant(i: usize, kind: AnimalKind) -> Animal {
    Animal {
        name: format!("{} {}", kind, i),
        kind,
        fields: HashMap::from_iter([("age", FieldValue::Number(i as i32))]),
    }
}
