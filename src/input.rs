use bevy::{prelude::*, render::camera::Camera};

use crate::{
    components::{Collider, LocationSector, Player, Prediction, Troop},
    data::{CameraNode, FactionPredictionCard, TurnPredictionCard},
    lerper::{Lerp, LerpType},
    phase::{Action, ActionAggregation, ActionQueue, Context},
    resources::{Data, Info},
    util::{closest, closest_mut, MutRayCastResult, RayCastResult},
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
    camera: Query<Entity, (With<Camera>, Without<Lerp>)>,
    colliders: Query<(Entity, &Collider, &Transform, &CameraNode)>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Some(camera) = camera.iter().next() {
            if let Some(RayCastResult {
                intersection: _,
                entity: _,
                component: &cam_node,
            }) = closest(&windows, &cameras, &colliders)
            {
                commands.insert_one(
                    camera,
                    Lerp::new(
                        LerpType::Camera {
                            src: None,
                            dest: cam_node,
                        },
                        1.0,
                        0.0,
                    ),
                );
            }
        }
    } else if keyboard_input.just_pressed(KeyCode::Escape) {
        if let Some(camera) = camera.iter().next() {
            commands.insert_one(
                camera,
                Lerp::new(
                    LerpType::Camera {
                        src: None,
                        dest: data.camera_nodes.main,
                    },
                    1.0,
                    0.0,
                ),
            );
        }
    }
}

fn sector_context_system(
    mut info: ResMut<Info>,
    mut queue: ResMut<ActionQueue>,
    windows: Res<Windows>,
    mouse_input: Res<Input<MouseButton>>,
    cameras: Query<(&Camera, &Transform)>,
    colliders: Query<(Entity, &Collider, &Transform, &LocationSector)>,
    players: Query<&Player>,
    mut troops: Query<(Entity, &Collider, &Transform, &mut Troop)>,
) {
    match info.context {
        Context::PlacingTroops => {
            if mouse_input.just_pressed(MouseButton::Left) {
                if let Some(RayCastResult {
                    intersection,
                    entity: location_entity,
                    component: location_sector,
                }) = closest(&windows, &cameras, &colliders)
                {
                    println!(
                        "Clicked on {}-{}",
                        location_sector.location.name, location_sector.sector
                    );
                    match info.context {
                        Context::PlacingTroops => {
                            if let Ok(active_player) = players.get(info.get_active_player()) {
                                println!("Active player: {:?}", active_player.faction);
                                let (num_troops, locations, _) =
                                    active_player.faction.initial_values();

                                let mut place = false;
                                println!("Valid Locations: {:?}", locations);
                                if let Some(locations) = locations {
                                    if locations
                                        .iter()
                                        .find(|name| {
                                            name.as_str() == location_sector.location.name.as_str()
                                        })
                                        .is_some()
                                    {
                                        place = true;
                                    }
                                } else {
                                    place = true;
                                }
                                println!("Place: {}", place);
                                if place {
                                    let (lerp_entity, _, _, mut new_troop) = troops
                                        .iter_mut()
                                        .max_by(|(_, _, transform1, _), (_, _, transform2, _)| {
                                            transform1
                                                .translation
                                                .y
                                                .partial_cmp(&transform2.translation.y)
                                                .unwrap()
                                        })
                                        .unwrap();
                                    new_troop.location = Some(location_entity);
                                    let lerp = if let Some(MutRayCastResult {
                                        intersection: _,
                                        entity,
                                        component: _,
                                    }) = closest_mut(&windows, &cameras, &mut troops)
                                    {
                                        let troop_transform =
                                            troops.get_component::<Transform>(entity).unwrap();
                                        Lerp::new(
                                            LerpType::World {
                                                src: None,
                                                dest: *troop_transform
                                                    * Transform::from_translation(
                                                        0.0036 * Vec3::unit_y(),
                                                    ),
                                            },
                                            1.0,
                                            0.0,
                                        )
                                    } else {
                                        Lerp::new(
                                            LerpType::World {
                                                src: None,
                                                dest: Transform::from_translation(intersection)
                                                    * Transform::from_translation(
                                                        0.0018 * Vec3::unit_y(),
                                                    ),
                                            },
                                            1.0,
                                            0.0,
                                        )
                                    };
                                    queue.push_single_front(Action::add_lerp(lerp_entity, lerp));
                                    if troops
                                        .iter_mut()
                                        .filter(|(_, _, _, troop)| troop.location.is_some())
                                        .count()
                                        == num_troops as usize
                                    {
                                        info.context = Context::None;
                                    }
                                } else {
                                    println!("Tried to place troop in an invalid location!");
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }
        }

        Context::None => {}
        Context::Predicting => {}
        Context::PickingTraitors => {}
        Context::Prompting => {}
        Context::StackResolving => {}
    }
}

fn prediction_context_system(
    mut info: ResMut<Info>,
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
    if info.context == Context::Predicting {
        if mouse_input.just_pressed(MouseButton::Left) {
            if let Some(RayCastResult {
                intersection: _,
                entity: element,
                component: faction_card,
            }) = closest(&windows, &cameras, colliders.q0())
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
                    Lerp::new(
                        LerpType::UI {
                            src: None,
                            dest: data.prediction_nodes.chosen_faction,
                        },
                        1.0,
                        0.0,
                    ),
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
                            Lerp::new(
                                LerpType::UI {
                                    src: None,
                                    dest: data.prediction_nodes.src,
                                },
                                indiv_anim_time,
                                1.0 + (delay * i as f32),
                            ),
                        )
                    })
                    .collect::<Vec<_>>();
                out_actions.push(chosen_action);
                queue.push_front(ActionAggregation::Multiple(out_actions));
                info.context = Context::None;
            }
            if let Some(RayCastResult {
                intersection: _,
                entity: element,
                component: turn_card,
            }) = closest(&windows, &cameras, &colliders.q1())
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
                    Lerp::new(
                        LerpType::UI {
                            src: None,
                            dest: data.prediction_nodes.chosen_turn,
                        },
                        1.0,
                        0.0,
                    ),
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
                            Lerp::new(
                                LerpType::UI {
                                    src: None,
                                    dest: data.prediction_nodes.src,
                                },
                                indiv_anim_time,
                                1.0 + (delay * i as f32),
                            ),
                        )
                    })
                    .collect::<Vec<_>>();
                out_actions.push(chosen_action);
                queue.push_front(ActionAggregation::Multiple(out_actions));
                info.context = Context::None;
            }
        }
    }
}
