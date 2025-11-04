use itertools::Itertools;

use crate::entity::{
    brain::{
        characteristic::Characteristic,
        player_action::PlayerAction,
        signal::{Signal, SignalContext, SignalRef, WeightedPlayerActions},
    },
    Entity,
};

/// Some future need that can be planned for
#[derive(Clone, Copy, Debug)]
pub enum PlanningSignal {
    /// Do we have access to food in inventory?
    FoodAccess,
    // Do we have access to water in inventory?
    // (NOT REALLY A THING YET)
    // WaterAccess,

    // Do we have shelter available to us?
    // (NOT REALLY A THING YET)
    // Shelter,
}

impl Signal for PlanningSignal {
    // NOTE: all of these use low weights, as they are not immediately important
    fn act_on(&self, ctx: &SignalContext, actions: &mut WeightedPlayerActions) {
        // We need a location for these to make sense
        let Some(hex) = ctx.entity.attributes.hex else {
            return;
        };

        match self {
            // PlanningSignal::WaterAccess => todo!(),
            // PlanningSignal::Shelter => todo!(),
            PlanningSignal::FoodAccess => {
                // Attempt to pick up food at our location
                // Is there food we could pick up?
                for food_entity in ctx
                    .entities
                    .in_hex(hex)
                    .filter(|e| e.attributes.food.is_some())
                {
                    // If the food is morally wrong and we care about that, dont pick it up lol
                    let food = food_entity.attributes.food.as_ref().unwrap();
                    if food.morally_wrong
                        && !ctx.entity.characteristic(Characteristic::Empathy).is_low()
                    {
                        continue;
                    }

                    actions.add(2, PlayerAction::PickUpEntity(food_entity.entity_id.clone()));
                    break;
                }
            }
        }
    }
}

impl Entity {
    pub fn get_planning_signals(&self, ctx: &SignalContext) -> impl Iterator<Item = SignalRef> {
        let mut plan_signals = Vec::new();

        // Get all the items in our inventory, thats a large part of it
        let inventory = self.resolve_inventory(ctx.entities).collect_vec();

        // Do we have food available in inventory?
        let inv_has_food = inventory.iter().any(|e| e.attributes.food.is_some());

        // If no food, plan to get that
        if !inv_has_food {
            plan_signals.push(PlanningSignal::FoodAccess);
        }

        // Do we have water in inventory - no such thing yet
        // let inv_has_food = inventory.iter().any(|e| e.attributes.water_source);

        // If the entity is not good at planning, they dont get these signals
        // (doing this a lazy way here)
        if ctx.entity.characteristic(Characteristic::Planning).is_low() {
            plan_signals.clear();
        }

        // Return all the signals
        plan_signals.into_iter().map(SignalRef::boxed)
    }
}
