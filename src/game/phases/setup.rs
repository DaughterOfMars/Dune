use std::{
    collections::{HashSet, VecDeque},
    f32::consts::PI,
};

use bevy::{math::vec3, prelude::*};
use bevy_mod_picking::{PickableBundle, PickingEvent};
use iyes_loopless::{
    prelude::{ConditionHelpers, IntoConditionalSystem},
    state::{CurrentState, NextState},
};
use maplit::hashset;

use crate::{
    components::{Card, Faction, FactionPredictionCard, Player, Spice, Troop, TurnPredictionCard, Unique},
    game::Phase,
    lerper::{InterpolationFunction, Lerp, UITransform},
    resources::{Data, Info},
    util::divide_spice,
    Active, GameEntity, Screen,
};

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FactionPickedEvent>();
        app.add_system(
            pick_factions_step
                .run_in_state(Screen::Game)
                .run_in_state(Phase::Setup(SetupPhase::ChooseFactions)),
        )
        .add_system(
            faction_card_picker
                .run_in_state(Screen::Game)
                .run_in_state(Phase::Setup(SetupPhase::ChooseFactions)),
        );
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SetupPhase {
    ChooseFactions,
    Prediction,
    AtStart,
    DealTraitors,
    PickTraitors,
    DealTreachery,
}

pub struct FactionPickedEvent {
    entity: Entity,
    faction: Faction,
}

struct ChooseFactionsStep {
    faction_cards: Vec<Entity>,
    pick_queue: VecDeque<Entity>,
    remaining_factions: HashSet<Faction>,
}

impl FromWorld for ChooseFactionsStep {
    fn from_world(world: &mut World) -> Self {
        let info = world.get_resource::<Info>().unwrap();
        Self {
            faction_cards: Default::default(),
            pick_queue: info.turn_order.iter().copied().collect(),
            remaining_factions: hashset![
                Faction::Atreides,
                Faction::BeneGesserit,
                Faction::Emperor,
                Faction::Fremen,
                Faction::Harkonnen,
                Faction::SpacingGuild,
            ],
        }
    }
}

fn pick_factions_step(
    mut commands: Commands,
    data: Res<Data>,
    mut active: ResMut<Active>,
    phase: Res<CurrentState<Phase>>,
    mut picked_events: EventReader<FactionPickedEvent>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: Local<ChooseFactionsStep>,
) {
    let card_face = asset_server.get_handle("card.gltf#Mesh0/Primitive0");
    let card_back = asset_server.get_handle("card.gltf#Mesh0/Primitive1");

    let prediction_back_texture = asset_server.get_handle("predictions/prediction_back.png");

    for FactionPickedEvent { entity, faction } in picked_events.iter() {
        if *entity == active.entity {
            state.remaining_factions.remove(faction);
            for entity in state.faction_cards.drain(..) {
                // TODO: animate them away~
                commands.entity(entity).despawn_recursive();
            }

            let shield_face = asset_server.get_handle("shield.gltf#Mesh0/Primitive1");
            let shield_back = asset_server.get_handle("shield.gltf#Mesh0/Primitive2");

            let little_token = asset_server.get_handle("little_token.gltf#Mesh0/Primitive0");
            let big_token = asset_server.get_handle("big_token.gltf#Mesh0/Primitive0");
            let spice_token = asset_server.get_handle("spice_token.gltf#Mesh0/Primitive0");

            let shield_front_texture =
                asset_server.get_handle(format!("shields/{}_shield_front.png", faction.code()).as_str());
            let shield_back_texture =
                asset_server.get_handle(format!("shields/{}_shield_back.png", faction.code()).as_str());

            commands
                .spawn_bundle(SpatialBundle::from_transform(Transform::from_translation(vec3(
                    0.0, 0.27, 1.34,
                ))))
                .insert(Unique::new(*faction))
                .insert(GameEntity)
                .insert(data.camera_nodes.shield)
                .with_children(|parent| {
                    parent
                        .spawn_bundle(PbrBundle {
                            mesh: shield_face.clone(),
                            material: materials.add(StandardMaterial::from(shield_front_texture)),
                            ..Default::default()
                        })
                        .insert_bundle(PickableBundle::default());
                    parent
                        .spawn_bundle(PbrBundle {
                            mesh: shield_back.clone(),
                            material: materials.add(StandardMaterial::from(shield_back_texture)),
                            ..Default::default()
                        })
                        .insert_bundle(PickableBundle::default());
                });

            let prediction_front_texture =
                asset_server.get_handle(format!("predictions/prediction_{}.png", faction.code()).as_str());

            commands
                .spawn_bundle(SpatialBundle::default())
                .insert(Unique::new(Faction::BeneGesserit))
                .insert_bundle(PickableBundle::default())
                .insert(GameEntity)
                .insert(FactionPredictionCard { faction: *faction })
                .with_children(|parent| {
                    parent.spawn_bundle(PbrBundle {
                        mesh: card_face.clone(),
                        material: materials.add(StandardMaterial::from(prediction_front_texture)),
                        ..Default::default()
                    });
                    parent.spawn_bundle(PbrBundle {
                        mesh: card_back.clone(),
                        material: materials.add(StandardMaterial::from(prediction_back_texture.clone())),
                        ..Default::default()
                    });
                });

            for (i, (_, leader_data)) in data.leaders.iter().filter(|(_, l)| l.faction == *faction).enumerate() {
                let texture = asset_server.get_handle(format!("leaders/{}.png", leader_data.texture).as_str());
                let material = materials.add(StandardMaterial::from(texture));
                commands
                    .spawn_bundle(SpatialBundle::from_transform(Transform::from_translation(
                        data.token_nodes.leaders[i],
                    )))
                    .insert(Unique::new(*faction))
                    .insert_bundle(PickableBundle::default())
                    .insert(GameEntity)
                    .insert_bundle(PbrBundle {
                        mesh: big_token.clone(),
                        material,
                        ..Default::default()
                    });
            }

            let troop_texture = asset_server.get_handle(format!("tokens/{}_troop.png", faction.code()).as_str());
            let troop_material = materials.add(StandardMaterial::from(troop_texture));

            for i in 0..20 {
                commands
                    .spawn_bundle(SpatialBundle::from_transform(Transform::from_translation(
                        data.token_nodes.fighters[0] + (i as f32 * 0.0036 * Vec3::Y),
                    )))
                    .insert(Unique::new(*faction))
                    .insert_bundle(PickableBundle::default())
                    .insert(GameEntity)
                    .insert(Troop {
                        value: 1,
                        location: None,
                    })
                    .insert_bundle(PbrBundle {
                        mesh: little_token.clone(),
                        material: troop_material.clone(),
                        ..Default::default()
                    });
            }

            let spice_1_texture = asset_server.get_handle("tokens/spice_1.png");
            let spice_1_material = materials.add(StandardMaterial::from(spice_1_texture));
            let spice_2_texture = asset_server.get_handle("tokens/spice_2.png");
            let spice_2_material = materials.add(StandardMaterial::from(spice_2_texture));
            let spice_5_texture = asset_server.get_handle("tokens/spice_5.png");
            let spice_5_material = materials.add(StandardMaterial::from(spice_5_texture));
            let spice_10_texture = asset_server.get_handle("tokens/spice_10.png");
            let spice_10_material = materials.add(StandardMaterial::from(spice_10_texture));

            let spice = data.factions.get(&faction).unwrap().starting_values.spice;

            let (tens, fives, twos, ones) = divide_spice(spice as i32);
            for (i, (value, s)) in (0..tens)
                .zip(std::iter::repeat((10, 0)))
                .chain((0..fives).zip(std::iter::repeat((5, 1))))
                .chain((0..twos).zip(std::iter::repeat((2, 2))))
                .chain((0..ones).zip(std::iter::repeat((1, 3))))
            {
                let material = match value {
                    1 => spice_1_material.clone(),
                    2 => spice_2_material.clone(),
                    5 => spice_5_material.clone(),
                    _ => spice_10_material.clone(),
                };
                commands
                    .spawn_bundle(SpatialBundle::from_transform(Transform::from_translation(
                        data.token_nodes.spice[s] + (i as f32 * 0.0036 * Vec3::Y),
                    )))
                    .insert(Unique::new(*faction))
                    .insert_bundle(PickableBundle::default())
                    .insert(GameEntity)
                    .insert(Spice { value })
                    .insert_bundle(PbrBundle {
                        mesh: spice_token.clone(),
                        material,
                        ..Default::default()
                    });
            }
            break;
        }
    }
    if state.faction_cards.is_empty() {
        if let Some(player_entity) = state.pick_queue.pop_front() {
            active.entity = player_entity;
            for (i, faction) in state.remaining_factions.clone().into_iter().enumerate() {
                let prediction_front_texture =
                    asset_server.get_handle(format!("predictions/prediction_{}.png", faction.code()).as_str());

                let node = data.prediction_nodes.factions[i];
                state.faction_cards.push(
                    commands
                        .spawn_bundle((Card, FactionPredictionCard { faction }))
                        .insert(
                            Lerp::ui_from_to(
                                UITransform::default().with_rotation(Quat::from_rotation_x(PI / 2.0)),
                                UITransform::from(node).with_rotation(Quat::from_rotation_x(PI / 2.0)),
                                player_entity,
                                0.5,
                                0.03 * i as f32,
                            )
                            .with_interpolation(InterpolationFunction::Easing),
                        )
                        .insert(Unique::new(faction))
                        .insert_bundle(SpatialBundle::default())
                        .with_children(|parent| {
                            parent
                                .spawn_bundle(PbrBundle {
                                    mesh: card_face.clone(),
                                    material: materials.add(StandardMaterial::from(prediction_front_texture)),
                                    ..default()
                                })
                                .insert_bundle(PickableBundle::default());
                            parent
                                .spawn_bundle(PbrBundle {
                                    mesh: card_back.clone(),
                                    material: materials.add(StandardMaterial::from(prediction_back_texture.clone())),
                                    ..default()
                                })
                                .insert_bundle(PickableBundle::default());
                        })
                        .id(),
                );
            }
        } else {
            commands.insert_resource(NextState(phase.0.next()));
        }
    }
}

fn faction_card_picker(
    mut commands: Commands,
    active: Res<Active>,
    faction_cards: Query<(&FactionPredictionCard, Option<&Lerp>)>,
    free_cams: Query<Entity, (With<Camera>, With<Player>, Without<Faction>)>,
    parents: Query<&Parent>,
    mut picking_events: EventReader<PickingEvent>,
    mut picked_events: EventWriter<FactionPickedEvent>,
) {
    for event in picking_events.iter() {
        if let PickingEvent::Clicked(clicked) = event {
            let mut clicked = *clicked;
            loop {
                if let Ok((faction_card, lerp)) = faction_cards.get(clicked) {
                    if lerp.is_none() {
                        if let Ok(active_cam) = free_cams.get(active.entity) {
                            commands.entity(active_cam).insert(*&faction_card.faction);
                            picked_events.send(FactionPickedEvent {
                                entity: active_cam,
                                faction: *&faction_card.faction,
                            });
                            return;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
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

fn predict_winner_step(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let card_face = asset_server.get_handle("card.gltf#Mesh0/Primitive0");
    let card_back = asset_server.get_handle("card.gltf#Mesh0/Primitive1");

    let prediction_back_texture = asset_server.get_handle("predictions/prediction_back.png");
    let prediction_back_material = materials.add(StandardMaterial::from(prediction_back_texture));

    (1..=15).for_each(|turn| {
        let prediction_front_texture =
            asset_server.get_handle(format!("predictions/prediction_t{}.png", turn).as_str());
        let prediction_front_material = materials.add(StandardMaterial::from(prediction_front_texture));
        commands
            .spawn_bundle(SpatialBundle::default())
            .insert(Unique::new(Faction::BeneGesserit))
            .insert(GameEntity)
            .insert(TurnPredictionCard { turn })
            .with_children(|parent| {
                parent
                    .spawn_bundle(PbrBundle {
                        mesh: card_face.clone(),
                        material: prediction_front_material,
                        ..Default::default()
                    })
                    .insert_bundle(PickableBundle::default());
                parent
                    .spawn_bundle(PbrBundle {
                        mesh: card_back.clone(),
                        material: prediction_back_material.clone(),
                        ..Default::default()
                    })
                    .insert_bundle(PickableBundle::default());
            });
    });
}
