pub mod grid;
pub mod position;

use crate::{
    app_state::check_story_over, crater_simulation::crater::AtCrater, nest_simulation::{element::update_element_exposure, nest::AtNest}, story_time::set_rate_of_time
};

use self::position::Position;
use super::{
    app_state::{
        begin_story, continue_startup, finalize_startup, post_setup_clear_change_detection,
        restart, AppState,
    },
    external_event::{
        initialize_external_event_resources, process_external_event,
        remove_external_event_resources,
    },
    // TODO: Element should live in common once I finish adding it to Crater.
    nest_simulation::element::denormalize_element,
    save::{
        bind_save_onbeforeunload, delete_save_file, initialize_save_resources, load,
        remove_save_resources, save, unbind_save_onbeforeunload,
    },
    settings::{initialize_settings_resources, register_settings, remove_settings_resources},
    story_time::{
        initialize_story_time_resources, register_story_time, remove_story_time_resources,
        setup_story_time, update_story_elapsed_ticks, update_story_real_world_time,
        update_time_scale, StoryPlaybackState,
    },
    CleanupSet,
    FinishSetupSet,
    SimulationTickSet,
    SimulationUpdate,
};
use bevy::prelude::*;

// This maps to AtNest or AtCrater
/// Use an empty trait to mark Nest and Crater zones to ensure strong type safety in generic systems.
pub trait Zone: Component {}

pub fn register_common(app_type_registry: ResMut<AppTypeRegistry>) {
    app_type_registry.write().register::<Entity>();
    app_type_registry.write().register::<Option<Entity>>();
    app_type_registry.write().register::<Position>();
}

pub fn despawn_model<Model: Component>(
    model_query: Query<Entity, With<Model>>,
    mut commands: Commands,
) {
    for model_entity in model_query.iter() {
        commands.entity(model_entity).despawn();
    }
}

pub struct CommonSimulationPlugin;

impl Plugin for CommonSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::BeginSetup),
            (register_settings, register_common, register_story_time),
        );

        app.add_systems(
            OnEnter(AppState::TryLoadSave),
            (
                initialize_save_resources,
                apply_deferred,
                load.pipe(continue_startup),
            )
                .chain(),
        );

        app.add_systems(
            OnEnter(AppState::CreateNewStory),
            (initialize_settings_resources, finalize_startup).chain(),
        );

        app.add_systems(
            OnEnter(AppState::FinishSetup),
            (
                initialize_story_time_resources,
                initialize_external_event_resources,
                bind_save_onbeforeunload,
                post_setup_clear_change_detection,
            )
                .chain()
                .in_set(FinishSetupSet::SimulationFinishSetup),
        );

        // IMPORTANT: setup_story_time sets FixedTime.accumulated which is reset when transitioning between schedules.
        // If this is ran OnEnter FinishSetup then the accumulated time will be reset to zero before FixedUpdate runs.
        app.add_systems(OnExit(AppState::FinishSetup), setup_story_time);

        app.add_systems(
            OnEnter(AppState::PostSetupClearChangeDetection),
            begin_story,
        );

        app.add_systems(
            SimulationUpdate,
            (
                process_external_event::<AtNest>,
                process_external_event::<AtCrater>,
                apply_deferred,
                denormalize_element,
                apply_deferred,
            )
                .chain()
                .in_set(SimulationTickSet::First),
        );

        app.add_systems(
            SimulationUpdate,
            (update_story_elapsed_ticks,)
                .chain()
                .in_set(SimulationTickSet::PostSimulationTick)
                .run_if(not(in_state(StoryPlaybackState::Paused))),
        );

        // TODO: Maybe (some?) of these should just run in Update?
        // Ending story seems like it should check every tick, but updating element exposure/updating story time seems OK to run just in Update?
        app.add_systems(
            SimulationUpdate,
            (
                // If this doesn't run then when user spawns elements they won't gain exposure if simulation is paused.
                apply_deferred,
                check_story_over,
                update_element_exposure,
                // real-world time should update even if the story is paused because real-world time doesn't pause
                // rate_of_time needs to run when app is paused because fixed_time accumulations need to be cleared while app is paused
                // to prevent running FixedUpdate schedule repeatedly (while no-oping) when coming back to a hidden tab with a paused sim.
                (update_story_real_world_time, set_rate_of_time).chain(),
            )
                .chain()
                .in_set(SimulationTickSet::Last),
        );

        app.add_systems(
            Update,
            update_time_scale.run_if(in_state(AppState::TellStory)),
        );

        app.add_systems(
            Update,
            update_story_real_world_time.run_if(in_state(AppState::TellStory)),
        );

        // Saving in WASM writes to local storage which requires dedicated support.
        app.add_systems(
            PostUpdate,
            // Saving is an expensive operation. Skip while fast-forwarding for performance.
            // TODO: It's weird (incorrect) that this is declared in `simulation` but that the `save` directory is external to simulation.
            // I think this should get moved up a level.
            save.run_if(
                in_state(AppState::TellStory).and_then(in_state(StoryPlaybackState::Playing)),
            ),
        );

        app.add_systems(
            OnEnter(AppState::Cleanup),
            (
                unbind_save_onbeforeunload,
                delete_save_file,
                remove_story_time_resources,
                remove_settings_resources,
                remove_save_resources,
                remove_external_event_resources,
                restart,
            )
                .in_set(CleanupSet::SimulationCleanup),
        );
    }
}
