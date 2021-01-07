use bevy::{prelude::*, render::camera::Camera};

use crate::{
    components::{Collider, LocationSector, Prediction},
    data::{CameraNode, FactionPredictionCard, TurnPredictionCard},
    lerper::{Lerp, LerpType},
    phase::{Action, ActionAggregation, ActionQueue, Context},
    resources::{Data, Info},
    util::closest,
};

const STAGE: &str = "input";

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_stage(STAGE, SystemStage::parallel())
            .add_system_to_stage(STAGE, camera_system.system())
            .add_system_to_stage(STAGE, sector_context_system.system())
            .add_system_to_stage(STAGE, prediction_context_system.system());
    }
}

pub fn camera_system(
    commands: &mut Commands,
    data: Res<Data>,
    windows: Res<Windows>,
    mouse_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    cameras: Query<(&Camera, &Transform)>,
    mut camera: Query<Entity, (With<Camera>, Without<Lerp>)>,
    colliders: Query<(Entity, &Collider, &Transform, &CameraNode)>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Some(camera) = camera.iter_mut().next() {
            if let Some((_, &cam_node)) = closest(&windows, &cameras, &colliders) {
                commands.insert_one(
                    camera,
                    Lerp {
                        lerp_type: LerpType::Camera {
                            src: None,
                            dest: cam_node,
                        },
                        time: 1.0,
                        delay: 0.0,
                    },
                );
            }
        }
    } else if keyboard_input.just_pressed(KeyCode::Escape) {
        if let Some(mut camera) = camera.iter_mut().next() {
            commands.insert_one(
                camera,
                Lerp {
                    lerp_type: LerpType::Camera {
                        src: None,
                        dest: data.camera_nodes.main,
                    },
                    time: 1.0,
                    delay: 0.0,
                },
            );
        }
    }
}

fn sector_context_system(
    mut info: ResMut<Info>,
    windows: Res<Windows>,
    mouse_input: Res<Input<MouseButton>>,
    cameras: Query<(&Camera, &Transform)>,
    colliders: Query<(Entity, &Collider, &Transform, &LocationSector)>,
) {
    if info.context != Context::None {
        if mouse_input.just_pressed(MouseButton::Left) {
            if let Some((_, sector)) = closest(&windows, &cameras, &colliders) {
                match info.context {
                    Context::PlacingTroops => {
                        todo!();
                        info.context = Context::None;
                    }
                    _ => (),
                }
            }
        }
    }
}

fn prediction_context_system(
    info: Res<Info>,
    data: Res<Data>,
    mut queue: ResMut<ActionQueue>,
    windows: Res<Windows>,
    mouse_input: Res<Input<MouseButton>>,
    cameras: Query<(&Camera, &Transform)>,
    colliders: QuerySet<(
        Query<(Entity, &Collider, &Transform, &FactionPredictionCard)>,
        Query<(Entity, &Collider, &Transform, &TurnPredictionCard)>,
    )>,
    mut predictions: Query<&mut Prediction>,
) {
    if info.context != Context::None {
        if mouse_input.just_pressed(MouseButton::Left) {
            match info.context {
                Context::Predicting => {
                    if let Some((element, faction_card)) =
                        closest(&windows, &cameras, colliders.q0())
                    {
                        if let Some(mut player_prediction) = predictions.iter_mut().next() {
                            player_prediction.faction = Some(faction_card.faction);
                        }
                        let num_factions = info.factions_in_play.len();
                        let animation_time = 1.5;
                        let delay = animation_time / (2.0 * num_factions as f32);
                        let indiv_anim_time = animation_time - (delay * (num_factions - 1) as f32);
                        // Animate selected card
                        let chosen_action = Action::add_lerp(
                            element,
                            Lerp {
                                lerp_type: LerpType::UI {
                                    src: None,
                                    dest: data.prediction_nodes.chosen_faction,
                                },
                                time: 1.0,
                                delay: 0.0,
                            },
                        );

                        // Animate out faction cards
                        let mut out_actions = colliders
                            .q0()
                            .iter()
                            .filter(|(_, _, _, card)| card.faction != faction_card.faction)
                            .enumerate()
                            .map(|(i, (element, _, _, _))| {
                                Action::add_lerp(
                                    element,
                                    Lerp {
                                        lerp_type: LerpType::UI {
                                            src: None,
                                            dest: data.prediction_nodes.src,
                                        },
                                        time: indiv_anim_time,
                                        delay: 1.0 + (delay * i as f32),
                                    },
                                )
                            })
                            .collect::<Vec<_>>();
                        out_actions.push(chosen_action);
                        queue.push_front(ActionAggregation::Multiple(out_actions));
                    }
                    if let Some((element, turn_card)) = closest(&windows, &cameras, &colliders.q1())
                    {
                        if let Some(mut player_prediction) = predictions.iter_mut().next() {
                            player_prediction.turn = Some(turn_card.turn);
                        }
                        let animation_time = 1.5;
                        let delay = animation_time / 30.0;
                        let indiv_anim_time = animation_time - (delay * 14.0);
                        // Animate selected card
                        let chosen_action = Action::add_lerp(
                            element,
                            Lerp {
                                lerp_type: LerpType::UI {
                                    src: None,
                                    dest: data.prediction_nodes.chosen_turn,
                                },
                                time: 1.0,
                                delay: 0.0,
                            },
                        );
                        // Animate out turn cards
                        let mut out_actions = colliders
                            .q1()
                            .iter()
                            .filter(|(_, _, _, card)| card.turn != turn_card.turn)
                            .enumerate()
                            .map(|(i, (element, _, _, _))| {
                                Action::add_lerp(
                                    element,
                                    Lerp {
                                        lerp_type: LerpType::UI {
                                            src: None,
                                            dest: data.prediction_nodes.src,
                                        },
                                        time: indiv_anim_time,
                                        delay: 1.0 + (delay * i as f32),
                                    },
                                )
                            })
                            .collect::<Vec<_>>();
                        out_actions.push(chosen_action);
                        queue.push_front(ActionAggregation::Multiple(out_actions));
                    }
                }
                _ => (),
            }
        }
    }
}
