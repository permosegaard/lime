use serde_json::Deserializer;
use specs::prelude::*;

use super::*;

#[test]
fn de() {
    const DATA: &'static str = r#"
    {
        "ent1": {
            "comp1": 5,
            "comp2": {
                "value": 52,
                "name": "hello"
            }
        },
        "ent2": {
            "comp1": 6
        },
        "ent3": {
            "comp2": {
                "value": -45,
                "name": "world"
            }
        }
    }
    "#;

    #[derive(Clone, Debug, Component, Deserialize, Hash, Eq, PartialEq)]
    struct Comp1(i32);

    #[derive(Clone, Debug, Component, Deserialize, Hash, Eq, PartialEq)]
    struct Comp2 {
        value: i64,
        name: String,
    }

    let mut world = World::new();
    let mut registry = Registry::new();
    world.register::<Comp1>();
    registry.register::<Comp1>("comp1");
    world.register::<Comp2>();
    registry.register::<Comp2>("comp2");

    deserialize(&mut Deserializer::from_str(DATA), &registry, &world.res).unwrap();
    world.maintain();

    let ents: Vec<Entity> = (&*world.entities()).join().collect();
    assert_eq!(ents.len(), 3);

    let comp1s = world.read_storage::<Comp1>();
    let comp2s = world.read_storage::<Comp2>();

    assert_eq!(comp1s.get(ents[0]), Some(&Comp1(5)));
    assert_eq!(
        comp2s.get(ents[0]),
        Some(&Comp2 {
            value: 52,
            name: "hello".to_string(),
        })
    );

    assert_eq!(comp1s.get(ents[1]), Some(&Comp1(6)));
    assert_eq!(comp2s.get(ents[1]), None);

    assert_eq!(comp1s.get(ents[2]), None);
    assert_eq!(
        comp2s.get(ents[2]),
        Some(&Comp2 {
            value: -45,
            name: "world".to_string(),
        })
    );
}

#[test]
fn name() {
    const DATA: &'static str = r#"
    {
        "ent1": {
            "comp1": 5,
            "comp2": "ent2"
        },
        "ent2": {
            "comp1": 6
        },
        "ent3": {
            "comp2": "ent2"
        },
        "ent4": {
            "comp2": "ent1"
        }
    }
    "#;

    #[derive(Clone, Debug, Component, Deserialize, Hash, Eq, PartialEq)]
    struct Comp1(i32);

    #[derive(Clone, Debug, Component, Hash, Eq, PartialEq)]
    struct Comp2(Entity);

    impl DeserializeComponent for Comp2 {
        fn deserialize<'de, 'a, D>(
            mut seed: Seed<'de, 'a>,
            deserializer: D,
        ) -> Result<Self, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            #[derive(Deserialize)]
            struct Comp2De<'a>(#[serde(borrow)] &'a str);

            let Comp2De(name) = <Comp2De as de::Deserialize>::deserialize(deserializer)?;
            let entity = seed.get_entity(name);
            Ok(Comp2(entity))
        }
    }

    let mut world = World::new();
    let mut registry = Registry::new();
    world.register::<Comp1>();
    registry.register::<Comp1>("comp1");
    world.register::<Comp2>();
    registry.register::<Comp2>("comp2");

    deserialize(&mut Deserializer::from_str(DATA), &registry, &world.res).unwrap();
    world.maintain();

    let ents: Vec<Entity> = (&*world.entities()).join().collect();
    assert_eq!(ents.len(), 4);

    let comp1s = world.read_storage::<Comp1>();
    let comp2s = world.read_storage::<Comp2>();

    assert_eq!(comp1s.get(ents[0]), Some(&Comp1(5)));
    assert_eq!(comp2s.get(ents[0]), Some(&Comp2(ents[1])));

    assert_eq!(comp1s.get(ents[1]), Some(&Comp1(6)));
    assert_eq!(comp2s.get(ents[1]), None);

    assert_eq!(comp1s.get(ents[2]), None);
    assert_eq!(comp2s.get(ents[2]), Some(&Comp2(ents[1])));

    assert_eq!(comp1s.get(ents[3]), None);
    assert_eq!(comp2s.get(ents[3]), Some(&Comp2(ents[0])));
}