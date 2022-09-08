use std::f32::consts::PI;

use bevy::{math::vec3, prelude::*};
use bevy_mod_picking::PickableBundle;
use rand::seq::SliceRandom;

use super::{
    state::{DeckType, GameEvent, GameState, PlayerId, SpawnType},
    ActivePlayerText, Object, ObjectEntityMap, Phase, PhaseText, SetupPhase, Shuffling,
};
use crate::{
    components::{Card, Deck, TraitorCard, TraitorDeck},
    util::card_jitter,
};

pub fn spawn_object(
    mut commands: Commands,
    game_state: Res<GameState>,
    mut game_events: EventReader<GameEvent>,
    mut object_entity: ResMut<ObjectEntityMap>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    my_id: Res<PlayerId>,
) {
    for event in game_events.iter() {
        if let GameEvent::SpawnObject { spawn_type } = event {
            match spawn_type {
                SpawnType::Leader {
                    player_id,
                    leader:
                        Object {
                            id: object_id,
                            inner: leader,
                        },
                } => {
                    if *my_id == *player_id {
                        let big_token = asset_server.get_handle("big_token.gltf#Mesh0/Primitive0");
                        let texture = asset_server
                            .get_handle(format!("leaders/{}.png", game_state.data.leaders[leader].texture).as_str());
                        let entity = commands
                            .spawn_bundle(SpatialBundle::from_transform(Transform::from_translation(
                                // TODO: Stack them somehow
                                game_state.data.token_nodes.leaders[0],
                            )))
                            .insert_bundle(PickableBundle::default())
                            .insert_bundle((*leader, *object_id))
                            .insert_bundle(PbrBundle {
                                mesh: big_token.clone(),
                                material: materials.add(StandardMaterial::from(texture)),
                                ..Default::default()
                            })
                            .id();
                        object_entity.map.insert(*object_id, entity);
                    } else {
                        // TODO: represent other player objects
                    }
                }
                SpawnType::Troop {
                    player_id,
                    unit:
                        Object {
                            id: object_id,
                            inner: unit,
                        },
                } => {
                    if *my_id == *player_id {
                        let faction = game_state.players.get(player_id).unwrap().faction;
                        let little_token = asset_server.get_handle("little_token.gltf#Mesh0/Primitive0");
                        let troop_texture =
                            asset_server.get_handle(format!("tokens/{}_troop.png", faction.code()).as_str());
                        let entity = commands
                            .spawn_bundle(SpatialBundle::from_transform(Transform::from_translation(
                                // TODO: Stack them somehow
                                game_state.data.token_nodes.fighters[0], // + (i as f32 * 0.0036 * Vec3::Y)
                            )))
                            .insert_bundle(PickableBundle::default())
                            .insert_bundle((*unit, *object_id))
                            .insert_bundle(PbrBundle {
                                mesh: little_token.clone(),
                                material: materials.add(StandardMaterial::from(troop_texture)),
                                ..Default::default()
                            })
                            .id();
                        object_entity.map.insert(*object_id, entity);
                    } else {
                        // TODO: represent other player objects
                    }
                }
                SpawnType::TraitorCard(_) => todo!(),
                SpawnType::TreacheryCard(Object {
                    id: object_id,
                    inner: card,
                }) => {
                    let card_face = asset_server.get_handle("card.gltf#Mesh0/Primitive0");
                    let card_back = asset_server.get_handle("card.gltf#Mesh0/Primitive1");

                    let treachery_front_texture = asset_server.get_handle(
                        format!(
                            "treachery/treachery_{}.png",
                            // TODO: somehow use variants consistently
                            game_state.data.treachery_cards[card]
                                .textures
                                .choose(&mut rand::thread_rng())
                                .unwrap()
                        )
                        .as_str(),
                    );

                    let treachery_back_texture = asset_server.get_handle("treachery/treachery_back.png");

                    let entity = commands
                        .spawn_bundle((*card, *object_id))
                        .insert_bundle(SpatialBundle::from_transform(
                            // TODO: stack them
                            Transform::from_translation(vec3(1.23, 0.0049, -0.87))
                                * Transform::from_rotation(Quat::from_rotation_z(PI)),
                        ))
                        .with_children(|parent| {
                            parent.spawn_bundle(PbrBundle {
                                mesh: card_face.clone(),
                                material: materials.add(StandardMaterial::from(treachery_front_texture)),
                                ..default()
                            });
                            parent.spawn_bundle(PbrBundle {
                                mesh: card_back.clone(),
                                material: materials.add(StandardMaterial::from(treachery_back_texture)),
                                ..default()
                            });
                        })
                        .id();
                    object_entity.map.insert(*object_id, entity);
                }
                SpawnType::SpiceCard(Object {
                    id: object_id,
                    inner: card,
                }) => {
                    let card_face = asset_server.get_handle("card.gltf#Mesh0/Primitive0");
                    let card_back = asset_server.get_handle("card.gltf#Mesh0/Primitive1");

                    let spice_front_texture = asset_server
                        .get_handle(format!("spice/spice_{}.png", game_state.data.spice_cards[card].texture).as_str());
                    let spice_back_texture = asset_server.get_handle("spice/spice_back.png");

                    let entity = commands
                        .spawn_bundle((*card, *object_id))
                        .insert_bundle(SpatialBundle {
                            transform: Transform::from_translation(vec3(1.23, 0.0049, 0.3))
                                * Transform::from_rotation(Quat::from_rotation_z(PI)),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn_bundle(PbrBundle {
                                mesh: card_face.clone(),
                                material: materials.add(StandardMaterial::from(spice_front_texture)),
                                ..default()
                            });
                            parent.spawn_bundle(PbrBundle {
                                mesh: card_back.clone(),
                                material: materials.add(StandardMaterial::from(spice_back_texture)),
                                ..default()
                            });
                        })
                        .id();
                    object_entity.map.insert(*object_id, entity);
                }
                SpawnType::StormCard(Object {
                    id: object_id,
                    inner: card,
                }) => {
                    let card_face = asset_server.get_handle("card.gltf#Mesh0/Primitive0");
                    let card_back = asset_server.get_handle("card.gltf#Mesh0/Primitive1");

                    let storm_front_texture = asset_server.get_handle(format!("storm/storm_{}.png", card.val).as_str());
                    let storm_back_texture = asset_server.get_handle("storm/storm_back.png");

                    let entity = commands
                        .spawn_bundle((*card, *object_id))
                        .insert_bundle(SpatialBundle {
                            transform: Transform::from_translation(vec3(1.23, 0.0049, 0.87))
                                * Transform::from_rotation(Quat::from_rotation_z(PI)),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn_bundle(PbrBundle {
                                mesh: card_face.clone(),
                                material: materials.add(StandardMaterial::from(storm_front_texture)),
                                ..default()
                            });
                            parent.spawn_bundle(PbrBundle {
                                mesh: card_back.clone(),
                                material: materials.add(StandardMaterial::from(storm_back_texture)),
                                ..default()
                            });
                        })
                        .id();
                    object_entity.map.insert(*object_id, entity);
                }
                SpawnType::Worm { location, id } => todo!(),
            }
        }
    }
}

pub fn deal_cards(mut commands: Commands, game_state: Res<GameState>, mut game_events: EventReader<GameEvent>) {
    todo!()
}

pub fn phase_text(game_state: Res<GameState>, mut text: Query<&mut Text, With<PhaseText>>) {
    if game_state.is_changed() {
        let s = match game_state.phase {
            Phase::Setup(subphase) => match subphase {
                SetupPhase::ChooseFactions => "Choosing Factions...".to_string(),
                SetupPhase::Prediction => "Bene Gesserit are making a prediction...".to_string(),
                SetupPhase::AtStart => "Initial Placement...".to_string(),
                SetupPhase::DealTraitors => "Picking Traitor Cards...".to_string(),
                SetupPhase::PlaceForces => "Placing Forces...".to_string(),
                SetupPhase::DealTreachery => "Dealing Treachery Cards...".to_string(),
            },
            Phase::Storm(_) => "Storm Phase".to_string(),
            Phase::SpiceBlow => "Spice Blow Phase".to_string(),
            Phase::Nexus => "Nexus Phase".to_string(),
            Phase::Bidding => "Bidding Phase".to_string(),
            Phase::Revival => "Revival Phase".to_string(),
            Phase::Movement => "Movement Phase".to_string(),
            Phase::Battle => "Battle Phase".to_string(),
            Phase::Collection => "Collection Phase".to_string(),
            Phase::Control => "Control Phase".to_string(),
            Phase::EndGame => "".to_string(),
        };

        text.single_mut().sections[0].value = s;
    }
}

pub fn active_player_text(game_state: Res<GameState>, mut text: Query<&mut Text, With<ActivePlayerText>>) {
    if game_state.is_changed() {
        text.single_mut().sections[0].value = game_state
            .active_player
            .as_ref()
            .map(|id| {
                format!(
                    "Active player: {}",
                    game_state
                        .players
                        .get(id)
                        .map(|p| game_state.data.factions[&p.faction].name.clone())
                        .unwrap_or(id.to_string())
                )
            })
            .unwrap_or_default();
    }
}

pub fn shuffle_traitors(
    mut commands: Commands,
    game_state: Res<GameState>,
    mut game_events: EventReader<GameEvent>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    traitor_deck: Query<Entity, With<TraitorDeck>>,
) {
    for event in game_events.iter() {
        if matches!(
            event,
            GameEvent::ShuffleDeck {
                deck_type: DeckType::Traitor
            }
        ) {
            if let Ok(traitor_deck) = traitor_deck.get_single() {
                commands.entity(traitor_deck).insert(Shuffling(5));
            } else {
                let card_face = asset_server.get_handle("card.gltf#Mesh0/Primitive0");
                let card_back = asset_server.get_handle("card.gltf#Mesh0/Primitive1");

                let traitor_back_texture = asset_server.get_handle("traitor/traitor_back.png");

                commands
                    .spawn_bundle((Deck, TraitorDeck, Shuffling(5)))
                    .insert_bundle(SpatialBundle::from_transform(
                        Transform::from_translation(vec3(1.23, 0.0049, -0.3))
                            * Transform::from_rotation(Quat::from_rotation_z(PI)),
                    ))
                    .with_children(|parent| {
                        for (i, (leader, leader_data)) in game_state
                            .decks
                            .traitor
                            .cards
                            .iter()
                            .map(|traitor_card| {
                                (
                                    traitor_card.inner.leader,
                                    &game_state.data.leaders[&traitor_card.inner.leader],
                                )
                            })
                            .enumerate()
                        {
                            let traitor_front_texture = asset_server
                                .get_handle(format!("traitor/traitor_{}.png", leader_data.texture.as_str()).as_str());
                            let traitor_front_material = materials.add(StandardMaterial::from(traitor_front_texture));

                            parent
                                .spawn_bundle((Card, TraitorCard { leader: leader.clone() }))
                                .insert_bundle(SpatialBundle::from_transform(
                                    Transform::from_translation(Vec3::Y * 0.001 * (i as f32)) * card_jitter(),
                                ))
                                .with_children(|parent| {
                                    parent.spawn_bundle(PbrBundle {
                                        mesh: card_face.clone(),
                                        material: traitor_front_material,
                                        ..default()
                                    });
                                    parent.spawn_bundle(PbrBundle {
                                        mesh: card_back.clone(),
                                        material: materials.add(StandardMaterial::from(traitor_back_texture.clone())),
                                        ..default()
                                    });
                                });
                        }
                    });
            }
        }
    }
}
