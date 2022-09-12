use std::f32::consts::PI;

use bevy::{
    math::{vec2, vec3},
    prelude::*,
};
use bevy_mod_picking::PickableBundle;
use derive_more::Display;
use iyes_loopless::prelude::ConditionSet;
use renet::RenetClient;
use serde::{Deserialize, Serialize};

use super::Phase;
use crate::{
    components::{FactionChoiceCard, FactionPredictionCard, Spice, TraitorCard, TurnPredictionCard},
    game::{
        state::{GameEvent, GameState, PlayerId, Prompt},
        GameEventStage, ObjectEntityMap, ObjectId, PickedEvent, PlayerFactionText,
    },
    lerper::{Lerp, Lerper, UITransform},
    network::{GameEvents, SendEvent},
    util::divide_spice,
    Screen,
};

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(Screen::Game)
                .with_system(faction_pick)
                .with_system(faction_prediction)
                .with_system(turn_prediction)
                .with_system(pick_traitor)
                .into(),
        );

        app.stage(GameEventStage, |stage: &mut SystemStage| {
            stage
                .add_system(prompt_factions)
                .add_system(faction_init)
                .add_system(prompt_predictions)
                .add_system(positions)
                .add_system(prompt_traitors);
            stage
        });
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Display, Serialize, Deserialize)]
pub enum SetupPhase {
    ChooseFactions,
    Prediction,
    AtStart,
    DealTraitors,
    PlaceForces,
    DealTreachery,
}

fn prompt_factions(
    game_events: Res<GameEvents>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_state: Res<GameState>,
    my_id: Res<PlayerId>,
) {
    if let Some(GameEvent::ShowPrompt {
        player_id,
        prompt: Prompt::Faction { remaining },
    }) = game_events.peek()
    {
        if *my_id == *player_id {
            let card_face = asset_server.get_handle("card.gltf#Mesh0/Primitive0");
            let card_back = asset_server.get_handle("card.gltf#Mesh0/Primitive1");
            let prediction_back_texture = asset_server.get_handle("predictions/prediction_back.png");
            for (i, faction) in remaining.iter().enumerate() {
                let prediction_front_texture =
                    asset_server.get_handle(format!("predictions/prediction_{}.png", faction.code()).as_str());

                let node = game_state.data.prediction_nodes.factions[i];

                commands
                    .spawn_bundle((FactionChoiceCard { faction: *faction },))
                    .insert(Lerper::from(Lerp::ui_from_to(
                        UITransform::default().with_rotation(Quat::from_rotation_x(PI / 2.0)),
                        UITransform::from(node).with_rotation(Quat::from_rotation_x(PI / 2.0)),
                        0.5,
                        0.03 * i as f32,
                    )))
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
                    });
            }
        }
    }
}

fn faction_pick(
    mut commands: Commands,
    mut picked_events: EventReader<PickedEvent<FactionChoiceCard>>,
    mut client: ResMut<RenetClient>,
    faction_cards: Query<Entity, With<FactionChoiceCard>>,
    my_id: Res<PlayerId>,
) {
    for PickedEvent {
        picked: _,
        inner: FactionChoiceCard { faction },
    } in picked_events.iter()
    {
        for entity in faction_cards.iter() {
            // TODO: animate them away~
            commands.entity(entity).despawn_recursive();
        }
        client.send_event(GameEvent::ChooseFaction {
            player_id: *my_id,
            faction: *faction,
        });
    }
}

fn faction_init(
    game_events: Res<GameEvents>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_state: Res<GameState>,
    my_id: Res<PlayerId>,
    mut text: Query<&mut Text, With<PlayerFactionText>>,
) {
    if let Some(GameEvent::ChooseFaction { player_id, faction }) = game_events.peek() {
        if *my_id == *player_id {
            text.single_mut().sections[0].value = format!("Player: {}", game_state.players[&my_id].faction);
            let shield_face = asset_server.get_handle("shield.gltf#Mesh0/Primitive1");
            let shield_back = asset_server.get_handle("shield.gltf#Mesh0/Primitive2");

            let spice_token = asset_server.get_handle("spice_token.gltf#Mesh0/Primitive0");

            let shield_front_texture =
                asset_server.get_handle(format!("shields/{}_shield_front.png", faction.code()).as_str());
            let shield_back_texture =
                asset_server.get_handle(format!("shields/{}_shield_back.png", faction.code()).as_str());

            commands
                .spawn_bundle(SpatialBundle::from_transform(Transform::from_translation(vec3(
                    0.0, 0.27, 1.34,
                ))))
                .insert(game_state.data.camera_nodes.shield)
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

            let spice_1_texture = asset_server.get_handle("tokens/spice_1.png");
            let spice_1_material = materials.add(StandardMaterial::from(spice_1_texture));
            let spice_2_texture = asset_server.get_handle("tokens/spice_2.png");
            let spice_2_material = materials.add(StandardMaterial::from(spice_2_texture));
            let spice_5_texture = asset_server.get_handle("tokens/spice_5.png");
            let spice_5_material = materials.add(StandardMaterial::from(spice_5_texture));
            let spice_10_texture = asset_server.get_handle("tokens/spice_10.png");
            let spice_10_material = materials.add(StandardMaterial::from(spice_10_texture));

            let spice = game_state.data.factions.get(&faction).unwrap().starting_values.spice;

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
                        game_state.data.token_nodes.spice[s] + (i as f32 * 0.0036 * Vec3::Y),
                    )))
                    .insert_bundle(PickableBundle::default())
                    .insert(Spice { value })
                    .insert_bundle(PbrBundle {
                        mesh: spice_token.clone(),
                        material,
                        ..Default::default()
                    });
            }
        } else {
            // TODO: display other player's faction picks
        }
    }
}

fn prompt_predictions(
    game_events: Res<GameEvents>,
    mut commands: Commands,
    game_state: Res<GameState>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    my_id: Res<PlayerId>,
) {
    if let Some(GameEvent::ShowPrompt { prompt, player_id }) = game_events.peek() {
        match prompt {
            Prompt::FactionPrediction => {
                if *my_id == *player_id {
                    let card_face = asset_server.get_handle("card.gltf#Mesh0/Primitive0");
                    let card_back = asset_server.get_handle("card.gltf#Mesh0/Primitive1");

                    let prediction_back_texture = asset_server.get_handle("predictions/prediction_back.png");

                    for (i, faction) in game_state.players.values().map(|player| player.faction).enumerate() {
                        let prediction_front_texture =
                            asset_server.get_handle(format!("predictions/prediction_{}.png", faction.code()).as_str());

                        let node = game_state.data.prediction_nodes.factions[i];

                        commands
                            .spawn_bundle((FactionPredictionCard { faction },))
                            .insert(Lerper::from(Lerp::ui_from_to(
                                UITransform::default().with_rotation(Quat::from_rotation_x(PI / 2.0)),
                                UITransform::from(node).with_rotation(Quat::from_rotation_x(PI / 2.0)),
                                0.5,
                                0.03 * i as f32,
                            )))
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
                                        material: materials
                                            .add(StandardMaterial::from(prediction_back_texture.clone())),
                                        ..default()
                                    })
                                    .insert_bundle(PickableBundle::default());
                            });
                    }
                }
            }
            Prompt::TurnPrediction => {
                if *my_id == *player_id {
                    let card_face = asset_server.get_handle("card.gltf#Mesh0/Primitive0");
                    let card_back = asset_server.get_handle("card.gltf#Mesh0/Primitive1");

                    let prediction_back_texture = asset_server.get_handle("predictions/prediction_back.png");

                    (1..=15).for_each(|turn| {
                        let prediction_front_texture =
                            asset_server.get_handle(format!("predictions/prediction_t{}.png", turn).as_str());

                        let i = turn as usize - 1;
                        let node = game_state.data.prediction_nodes.turns[i];

                        commands
                            .spawn_bundle(SpatialBundle::default())
                            .insert(Lerper::from(Lerp::ui_from_to(
                                UITransform::default()
                                    .with_rotation(Quat::from_rotation_x(PI / 2.0))
                                    .with_scale(0.6),
                                UITransform::from(node)
                                    .with_rotation(Quat::from_rotation_x(PI / 2.0))
                                    .with_scale(0.6),
                                0.5,
                                0.01 * i as f32,
                            )))
                            .insert(TurnPredictionCard { turn })
                            .with_children(|parent| {
                                parent
                                    .spawn_bundle(PbrBundle {
                                        mesh: card_face.clone(),
                                        material: materials.add(StandardMaterial::from(prediction_front_texture)),
                                        ..Default::default()
                                    })
                                    .insert_bundle(PickableBundle::default());
                                parent
                                    .spawn_bundle(PbrBundle {
                                        mesh: card_back.clone(),
                                        material: materials
                                            .add(StandardMaterial::from(prediction_back_texture.clone())),
                                        ..Default::default()
                                    })
                                    .insert_bundle(PickableBundle::default());
                            });
                    });
                }
            }
            _ => (),
        }
    }
}

fn faction_prediction(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    mut picked_events: EventReader<PickedEvent<FactionPredictionCard>>,
    cards: Query<Entity, With<FactionPredictionCard>>,
) {
    for PickedEvent {
        picked: _,
        inner: FactionPredictionCard { faction },
    } in picked_events.iter()
    {
        for entity in cards.iter() {
            // TODO: animate them away~
            commands.entity(entity).despawn_recursive();
        }
        client.send_event(GameEvent::MakeFactionPrediction { faction: *faction });
    }
}

fn turn_prediction(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    mut picked_events: EventReader<PickedEvent<TurnPredictionCard>>,
    cards: Query<Entity, With<TurnPredictionCard>>,
) {
    for PickedEvent {
        picked: _,
        inner: TurnPredictionCard { turn },
    } in picked_events.iter()
    {
        for entity in cards.iter() {
            // TODO: animate them away~
            commands.entity(entity).despawn_recursive();
        }
        client.send_event(GameEvent::MakeTurnPrediction { turn: *turn });
    }
}

fn positions(
    game_events: Res<GameEvents>,
    mut commands: Commands,
    game_state: Res<GameState>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if matches!(game_events.peek(), Some(GameEvent::AdvancePhase))
        && matches!(game_state.phase, Phase::Setup(SetupPhase::AtStart))
    {
        for (i, turn) in game_state.play_order.iter().enumerate() {
            let faction = game_state.players[turn].faction;
            let little_token = asset_server.get_handle("little_token.gltf#Mesh0/Primitive0");
            let logo_texture = asset_server.get_handle(format!("tokens/{}_logo.png", faction.code()).as_str());
            commands
                .spawn_bundle(SpatialBundle::from_transform(Transform::from_translation(
                    game_state.data.token_nodes.factions[i],
                )))
                .insert_bundle(PbrBundle {
                    mesh: little_token.clone(),
                    material: materials.add(StandardMaterial::from(logo_texture)),
                    ..Default::default()
                });
        }
    }
}

fn prompt_traitors(
    game_events: Res<GameEvents>,
    game_state: Res<GameState>,
    my_id: Res<PlayerId>,
    object_entity: Res<ObjectEntityMap>,
    mut traitors: Query<&mut Lerper>,
) {
    if let Some(GameEvent::ShowPrompt {
        player_id,
        prompt: Prompt::Traitor,
    }) = game_events.peek()
    {
        if *my_id == *player_id {
            let nodes = [vec2(-0.6, 0.0), vec2(-0.2, 0.0), vec2(0.2, 0.0), vec2(0.6, 0.0)];
            for (i, (card, node)) in game_state
                .players
                .get(player_id)
                .unwrap()
                .traitor_cards
                .iter()
                .zip(nodes)
                .enumerate()
            {
                if let Ok(mut lerper) = traitors.get_mut(object_entity.world[&card.id]) {
                    lerper.push(Lerp::world_to_ui(
                        UITransform::from(node).with_rotation(Quat::from_rotation_x(PI / 2.0)),
                        0.5,
                        0.03 * i as f32,
                    ));
                }
            }
        }
    }
}

fn pick_traitor(
    mut client: ResMut<RenetClient>,
    mut picked_events: EventReader<PickedEvent<TraitorCard>>,
    mut cards: Query<&ObjectId, With<TraitorCard>>,
    my_id: Res<PlayerId>,
) {
    for PickedEvent { picked, inner: _ } in picked_events.iter() {
        if let Ok(card_id) = cards.get_mut(*picked) {
            client.send_event(GameEvent::ChooseTraitor {
                player_id: *my_id,
                card_id: *card_id,
            });
        }
    }
}
