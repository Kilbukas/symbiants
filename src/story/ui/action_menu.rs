// Create a floating menu which contains a set of action icons. Very similar to Photoshop/Paint action menu.
// Used in Sandbox Mode to allow the user to play around with the environment - manually spawning/despawning anything that could exist.
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContexts};

use crate::settings::Settings;
use crate::story::crater_simulation::crater::Crater;
use crate::story::nest_rendering::common::VisibleGrid;
use crate::story::nest_simulation::nest::Nest;
use crate::story::pointer::ExternalSimulationEvent;
use crate::story::story_time::StoryTime;

#[derive(Resource, Default, PartialEq, Copy, Clone, Debug)]
pub enum PointerAction {
    #[default]
    Select,
    DespawnElement,
    SpawnFood,
    SpawnDirt,
    SpawnSand,
    KillAnt,
    SpawnWorkerAnt,
    DespawnWorkerAnt,
}

#[derive(Resource, Default, PartialEq, Copy, Clone, Debug)]
pub struct IsShowingBreathDialog(pub bool);

pub fn setup_action_menu(mut commands: Commands) {
    commands.init_resource::<PointerAction>();
    commands.init_resource::<IsShowingBreathDialog>();
}

pub fn teardown_action_menu(mut commands: Commands) {
    commands.remove_resource::<PointerAction>();
    commands.remove_resource::<IsShowingBreathDialog>();
}

pub fn update_action_menu(
    mut contexts: EguiContexts,
    mut pointer_action: ResMut<PointerAction>,
    mut is_showing_breath_dialog: ResMut<IsShowingBreathDialog>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    settings: Res<Settings>,
    story_time: Res<StoryTime>,
    mut external_simulation_event_writer: EventWriter<ExternalSimulationEvent>,
    visible_grid: Res<VisibleGrid>,
    nest_query: Query<&Nest>,
    crater_query: Query<&Crater>,
) {
    let window = primary_window_query.single();
    let ctx = contexts.ctx_mut();

    // TODO: resetting story doesn't reset window position
    egui::Window::new("Actions")
        .default_pos(egui::Pos2::new(window.width(), 0.0))
        .resizable(false)
        .show(ctx, |ui| {
            // TODO: Make it so that this button can only be clicked once per simulated day
            let food_disabled = settings.is_breathwork_scheduled
                && story_time.is_real_time
                && !story_time.is_within_schedule_window();

            ui.selectable_value(pointer_action.as_mut(), PointerAction::Select, "Select");
            ui.selectable_value(
                pointer_action.as_mut(),
                PointerAction::SpawnSand,
                "Place Sand",
            );

            ui.add_enabled_ui(!food_disabled, |ui| {
                ui.selectable_value(
                    pointer_action.as_mut(),
                    PointerAction::SpawnFood,
                    "Place Food",
                );
            });

            ui.selectable_value(
                pointer_action.as_mut(),
                PointerAction::SpawnDirt,
                "Place Dirt",
            );
            ui.selectable_value(
                pointer_action.as_mut(),
                PointerAction::DespawnElement,
                "Remove Element",
            );

            ui.selectable_value(
                pointer_action.as_mut(),
                PointerAction::SpawnWorkerAnt,
                "Place Worker Ant",
            );
            ui.selectable_value(
                pointer_action.as_mut(),
                PointerAction::DespawnWorkerAnt,
                "Remove Worker Ant",
            );

            ui.selectable_value(pointer_action.as_mut(), PointerAction::KillAnt, "Kill Ant");

            ui.add_enabled_ui(!food_disabled, |ui| {
                if ui.button("Breathe for Food").clicked() {
                    is_showing_breath_dialog.0 = true;
                }
            });

            if let Some(visible_grid_entity) = visible_grid.0 {
                let is_nest_visible = nest_query.get(visible_grid_entity).is_ok();
                let is_crater_visible = crater_query.get(visible_grid_entity).is_ok();

                if is_nest_visible {
                    if ui.button("View Crater").clicked() {
                        external_simulation_event_writer.send(ExternalSimulationEvent::ShowCrater);
                    }
                } else if is_crater_visible {
                    if ui.button("View Nest").clicked() {
                        external_simulation_event_writer.send(ExternalSimulationEvent::ShowNest);
                    }
                }
            } else {
                if ui.button("View Nest").clicked() {
                    external_simulation_event_writer.send(ExternalSimulationEvent::ShowNest);
                }
            }
        });
}
