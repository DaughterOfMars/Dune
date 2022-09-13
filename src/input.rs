use bevy::{prelude::*, render::camera::Camera};
use bevy_mod_picking::PickingEvent;
use iyes_loopless::prelude::IntoConditionalSystem;
use renet::RenetClient;

use crate::{
    data::{CameraNode, Data},
    game::state::{GameEvent, PlayerId},
    lerper::{Lerp, Lerper},
    network::SendEvent,
    Screen,
};

pub struct GameInputPlugin;

impl Plugin for GameInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(lookaround.run_in_state(Screen::Game))
            .add_system(camera_reset.run_in_state(Screen::Game))
            .add_system(pass.run_in_state(Screen::Game));

        #[cfg(feature = "debug")]
        app.add_system(debug_restart.run_in_state(Screen::Game));
    }
}

fn debug_restart(keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::F1) {
        // TODO: Disconnect from server
    }
}

fn lookaround(
    mut camera: Query<&mut Lerper, With<Camera>>,
    nodes: Query<&CameraNode>,
    parents: Query<&Parent>,
    mut events: EventReader<PickingEvent>,
) {
    for event in events.iter() {
        match event {
            PickingEvent::Selection(_) => (),
            PickingEvent::Hover(_) => (),
            PickingEvent::Clicked(clicked) => {
                if let Some(mut lerper) = camera.iter_mut().next() {
                    let mut clicked = *clicked;
                    loop {
                        if let Ok(camera_node) = nodes.get(clicked) {
                            lerper.set_if_empty(Lerp::move_camera(camera_node.clone(), 1.0));
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

fn camera_reset(data: Res<Data>, keyboard_input: Res<Input<KeyCode>>, mut camera: Query<&mut Lerper, With<Camera>>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        if let Some(mut lerper) = camera.iter_mut().next() {
            lerper.set_if_empty(Lerp::move_camera(data.camera_nodes.main, 1.0));
        }
    }
}

// Temporary pass input, TODO replace with a button or something
fn pass(keyboard_input: Res<Input<KeyCode>>, mut client: ResMut<RenetClient>, my_id: Res<PlayerId>) {
    if keyboard_input.just_pressed(KeyCode::P) {
        client.send_event(GameEvent::Pass { player_id: *my_id });
    }
}
