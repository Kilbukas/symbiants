pub mod ant;
pub mod background;
pub mod element;

use self::{
    ant::{cleanup_ants, on_spawn_ant, rerender_ants},
    background::{cleanup_background, spawn_background, CraterBackground},
    element::{
        cleanup_elements, on_spawn_element, rerender_elements, spawn_element_tilemap,
        ElementTilemap,
    },
};
use crate::common::{
    despawn_view, despawn_view_by_model, on_despawn,
    visible_grid::{VisibleGrid, VisibleGridState},
};
use bevy::prelude::*;
use simulation::{
    app_state::AppState,
    crater_simulation::crater::{AtCrater, Crater},
    nest_simulation::{ant::Ant, element::Element},
    CleanupSet,
};

pub struct CraterRenderingPlugin;

// TODO: Create a CraterGrid + CraterBackground
// Initially every spot in the grid is filled with air and the background draws a gray/brown floor
// Then need to spawn food at a location, put an ant sprite, and implement walking so that ant can grab it

impl Plugin for CraterRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Spawn
                (on_spawn_ant, on_spawn_element),
                // Despawn
                (on_despawn::<Ant, AtCrater>, on_despawn::<Element, AtCrater>),
            )
                .run_if(
                    in_state(AppState::TellStory)
                        .or_else(in_state(AppState::PostSetupClearChangeDetection)),
                ),
        );

        app.add_systems(
            OnEnter(VisibleGridState::Crater),
            (
                (spawn_element_tilemap),
                apply_deferred,
                (
                    spawn_background,
                    rerender_ants,
                    rerender_elements,
                    mark_crater_visible,
                ),
            )
                .chain()
                .run_if(in_state(AppState::TellStory)),
        );

        app.add_systems(
            OnExit(VisibleGridState::Crater),
            (
                despawn_view::<CraterBackground>,
                despawn_view_by_model::<Ant, AtCrater>,
                despawn_view_by_model::<Element, AtCrater>,
                despawn_view::<ElementTilemap>,
                mark_crater_hidden,
            )
                .run_if(in_state(AppState::TellStory)),
        );

        app.add_systems(
            OnEnter(AppState::Cleanup),
            (
                despawn_view::<CraterBackground>,
                cleanup_background,
                despawn_view_by_model::<Ant, AtCrater>,
                cleanup_ants,
                despawn_view_by_model::<Element, AtCrater>,
                despawn_view::<ElementTilemap>,
                cleanup_elements,
            )
                .in_set(CleanupSet::BeforeSimulationCleanup),
        );
    }
}

pub fn mark_crater_visible(
    crater_query: Query<Entity, With<Crater>>,
    mut visible_grid: ResMut<VisibleGrid>,
) {
    visible_grid.0 = Some(crater_query.single());
}

pub fn mark_crater_hidden(mut visible_grid: ResMut<VisibleGrid>) {
    visible_grid.0 = None;
}
