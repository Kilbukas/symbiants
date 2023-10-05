use super::{commands::AntCommandsExt, AntInventory, AntOrientation, Dead, Initiative};
use crate::{
    common::{get_entity_from_id, Id},
    element::Element,
    story_time::{DEFAULT_TICKS_PER_SECOND, SECONDS_PER_DAY},
    world_map::{position::Position, WorldMap},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Hunger {
    value: f32,
    max: f32,
}

impl Default for Hunger {
    fn default() -> Self {
        Self {
            value: 0.0,
            max: 100.0,
        }
    }
}

impl Hunger {
    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn max(&self) -> f32 {
        self.max
    }

    pub fn tick(&mut self, rate_of_hunger: f32) {
        self.value = (self.value + rate_of_hunger).min(self.max);
    }

    pub fn is_hungry(&self) -> bool {
        self.value >= self.max / 2.0
    }

    pub fn is_starved(&self) -> bool {
        self.value >= self.max
    }

    pub fn reset(&mut self) {
        self.value = 0.0;
    }
}

pub fn ants_hunger(
    mut ants_hunger_query: Query<
        (
            Entity,
            &mut Hunger,
            &AntOrientation,
            &Position,
            &mut AntInventory,
            &mut Initiative,
        ),
        Without<Dead>,
    >,
    elements_query: Query<&Element>,
    id_query: Query<(Entity, &Id)>,
    mut commands: Commands,
    world_map: Res<WorldMap>,
) {
    for (entity, mut hunger, orientation, position, mut inventory, mut initiative) in
        ants_hunger_query.iter_mut()
    {
        // Get 100% hungry once per full real-world day.
        let rate_of_hunger = hunger.max() / (SECONDS_PER_DAY * DEFAULT_TICKS_PER_SECOND) as f32;
        hunger.tick(rate_of_hunger);

        if hunger.is_starved() {
            commands.entity(entity).insert(Dead);
        } else if hunger.is_hungry() {
            if !initiative.can_act() {
                continue;
            }

            // If there is food near the hungry ant then pick it up and if the ant is holding food then eat it.
            if inventory.0 == None {
                let ahead_position = orientation.get_ahead_position(position);
                if world_map.is_element(&elements_query, ahead_position, Element::Food) {
                    let food_entity = world_map.get_element_entity(ahead_position).unwrap();
                    commands.dig(entity, ahead_position, *food_entity);
                    initiative.consume();
                }
            } else {
                let id = inventory.0.clone().unwrap();
                let entity = get_entity_from_id(id, &id_query).unwrap();
                let element = elements_query.get(entity).unwrap();

                if *element == Element::Food {
                    inventory.0 = None;
                    hunger.reset();
                    initiative.consume();
                }
            }
        }
    }
}
