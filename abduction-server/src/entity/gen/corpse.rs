use crate::entity::{Entity, EntityAttributes, EntityFood, EntityMarker};

pub fn generate_corpse(rng: &mut impl rand::Rng, player: Entity) -> Entity {
    // TODO
    Entity {
        entity_id: Entity::id(),
        markers: vec![EntityMarker::Inspectable],
        name: format!("Corpse of {}", &player.name),
        attributes: EntityAttributes {
            hex: player.attributes.hex,
            corpse: Some(player.entity_id),
            food: Some(EntityFood {
                morally_wrong: true,
                ..EntityFood::dubious(rng)
            }),
            ..Default::default()
        },
        ..Default::default()
    }
}
