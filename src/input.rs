use bevy::{prelude::*, render::camera::Camera};
use bevy_mod_picking::PickingEvent;

use crate::{
    components::Active,
    data::CameraNode,
    lerper::{Lerp, LerpType},
    resources::Data,
    Screen,
};

pub struct GameInputPlugin;

impl Plugin for GameInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(Screen::Game)
                .with_system(picker_system)
                .with_system(camera_reset_system),
        );

        #[cfg(feature = "debug")]
        app.add_system_set(SystemSet::on_update(Screen::Game).with_system(debug_restart_system));
    }
}

fn debug_restart_system(mut state: ResMut<State<Screen>>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::F1) {
        state.overwrite_replace(Screen::MainMenu).unwrap();
    }
}

fn picker_system(
    mut commands: Commands,
    camera: Query<Entity, (With<Camera>, Without<Lerp>)>,
    nodes: Query<&CameraNode>,
    mut events: EventReader<PickingEvent>,
) {
    for event in events.iter() {
        match event {
            PickingEvent::Selection(_) => (),
            PickingEvent::Hover(_) => (),
            PickingEvent::Clicked(clicked) => {
                if let Some(camera) = camera.iter().next() {
                    if let Ok(camera_node) = nodes.get(*clicked) {
                        commands
                            .entity(camera)
                            .insert(Lerp::move_camera(camera_node.clone(), 1.0));
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
            commands.entity(camera).insert(Lerp::new(
                LerpType::Camera {
                    src: None,
                    dest: data.camera_nodes.main,
                },
                1.0,
                0.0,
            ));
        }
    }
}
