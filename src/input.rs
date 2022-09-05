use bevy::{prelude::*, render::camera::Camera};
use bevy_mod_picking::PickingEvent;
use iyes_loopless::prelude::IntoConditionalSystem;

use crate::{data::CameraNode, lerper::Lerp, resources::Data, Screen};

pub struct GameInputPlugin;

impl Plugin for GameInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(picker_system.run_in_state(Screen::Game));
        app.add_system(camera_reset_system.run_in_state(Screen::Game));

        #[cfg(feature = "debug")]
        app.add_system(debug_restart_system.run_in_state(Screen::Game));
    }
}

fn debug_restart_system(keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::F1) {
        // TODO: Go back to main menu state
    }
}

fn picker_system(
    mut commands: Commands,
    camera: Query<Entity, (With<Camera>, Without<Lerp>)>,
    nodes: Query<&CameraNode>,
    parents: Query<&Parent>,
    mut events: EventReader<PickingEvent>,
) {
    for event in events.iter() {
        match event {
            PickingEvent::Selection(_) => (),
            PickingEvent::Hover(_) => (),
            PickingEvent::Clicked(clicked) => {
                if let Some(camera) = camera.iter().next() {
                    let mut clicked = *clicked;
                    loop {
                        if let Ok(camera_node) = nodes.get(clicked) {
                            commands
                                .entity(camera)
                                .insert(Lerp::move_camera(camera_node.clone(), 1.0));
                            return;
                        } else {
                            if let Ok(parent) = parents.get(clicked).map(|p| p.get()) {
                                clicked = parent;
                            } else {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

fn camera_reset_system(
    mut commands: Commands,
    data: Res<Data>,
    keyboard_input: Res<Input<KeyCode>>,
    camera: Query<Entity, (With<Camera>, Without<Lerp>)>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        if let Some(camera) = camera.iter().next() {
            commands
                .entity(camera)
                .insert(Lerp::move_camera(data.camera_nodes.main, 1.0));
        }
    }
}
