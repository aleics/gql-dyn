use std::collections::{HashMap, HashSet};

use crate::{
    AnimalKind,
    data::{Animal, FieldValue},
};

pub(crate) fn generate_animals(animal_kinds: &HashSet<AnimalKind>, amount: usize) -> Vec<Animal> {
    let mut result = Vec::with_capacity(amount * animal_kinds.len());

    for kind in animal_kinds {
        for i in 0..(amount / animal_kinds.len()) {
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
