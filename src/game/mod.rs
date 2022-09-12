mod object;
pub mod phase;
pub mod state;

use std::f32::consts::PI;

use bevy::{ecs::schedule::ShouldRun, math::vec3, prelude::*};
use bevy_mod_picking::{PickableBundle, PickingEvent};
use iyes_loopless::prelude::{AppLooplessStateExt, ConditionSet};
use maplit::hashset;
use renet::RenetClient;

pub use self::object::*;
use self::{
    phase::PhasePlugin,
    state::{DeckType, EventReduce, GameEvent, GameState, PlayerId, SpawnType},
};
use crate::{
    components::{FactionChoiceCard, FactionPredictionCard, LocationSector, TraitorCard, Troop, TurnPredictionCard},
    lerper::{Lerp, Lerper, UITransform},
    network::{GameEvents, SendEvent},
    util::hand_positions,
    Screen,
};

#[derive(StageLabel)]
pub struct GameEventStage;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ObjectEntityMap>();

        app.add_event::<PickedEvent<FactionChoiceCard>>()
            .add_event::<PickedEvent<FactionPredictionCard>>()
            .add_event::<PickedEvent<TurnPredictionCard>>()
            .add_event::<PickedEvent<TraitorCard>>()
            .add_event::<PickedEvent<LocationSector>>();

        app.add_system_set(
            ConditionSet::new()
                .run_in_state(Screen::Game)
                .with_system(hiararchy_picker::<FactionChoiceCard>)
                .with_system(hiararchy_picker::<FactionPredictionCard>)
                .with_system(hiararchy_picker::<TurnPredictionCard>)
                .with_system(hiararchy_picker::<TraitorCard>)
                .with_system(hiararchy_picker::<LocationSector>)
                .with_system(ship_troop_input)
                .into(),
        );

        app.add_stage_before(
            CoreStage::Update,
            GameEventStage,
            SystemStage::parallel()
                .with_run_criteria(check_for_event)
                .with_system(consume_events.exclusive_system().at_start())
                .with_system(pull_events.exclusive_system().at_end())
                .with_system(spawn_object)
                .with_system(ship_forces)
                .with_system(discard_card)
                .with_system(hand),
        );

        app.add_plugin(PhasePlugin);

        app.add_exit_system(Screen::Game, reset);
    }
}

fn consume_events(game_events: Res<GameEvents>, mut game_state: ResMut<GameState>) {
    if let Some(event) = game_events.peek().cloned() {
        game_state.consume(event);
    }
}

fn pull_events(mut game_events: ResMut<GameEvents>) {
    game_events.next();
}

pub fn check_for_event(game_events: Res<GameEvents>) -> ShouldRun {
    if game_events.peek().is_some() {
        ShouldRun::YesAndCheckAgain
    } else {
        ShouldRun::No
    }
}

#[derive(Component)]
pub struct PlayerFactionText;

fn reset() {
    todo!()
}

pub struct PickedEvent<T> {
    pub picked: Entity,
    pub inner: T,
}

// Converts PickingEvents to typed PickedEvents by looking up the hierarchy if needed
fn hiararchy_picker<T: Component + Clone>(
    pickables: Query<&T>,
    parents: Query<&Parent>,
    mut picking_events: EventReader<PickingEvent>,
    mut picked_events: EventWriter<PickedEvent<T>>,
) {
    if !pickables.is_empty() {
        for event in picking_events.iter() {
            if let PickingEvent::Clicked(clicked) = event {
                let mut clicked = *clicked;
                loop {
                    if let Ok(inner) = pickables.get(clicked) {
                        picked_events.send(PickedEvent {
                            picked: clicked,
                            inner: inner.clone(),
                        });
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

fn spawn_object(
    game_events: Res<GameEvents>,
    mut commands: Commands,
    game_state: Res<GameState>,
    mut object_entity: ResMut<ObjectEntityMap>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    my_id: Res<PlayerId>,
) {
    if let Some(GameEvent::SpawnObject { spawn_type }) = game_events.peek() {
        match spawn_type {
            SpawnType::Leader {
                player_id,
                leader: Object {
                    id: object_id,
                    inner: leader,
                },
            } => {
                if *my_id == *player_id {
                    let big_token = asset_server.get_handle("big_token.gltf#Mesh0/Primitive0");
                    let texture = asset_server
                        .get_handle(format!("leaders/{}.png", game_state.data.leaders[&leader].texture).as_str());
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
                        .insert(Lerper::default())
                        .id();
                    object_entity.world.insert(*object_id, entity);
                } else {
                    // TODO: represent other player objects
                }
            }
            SpawnType::Troop {
                player_id,
                unit: Object {
                    id: object_id,
                    inner: unit,
                },
            } => {
                if *my_id == *player_id {
                    let faction = game_state.players[&player_id].faction;
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
                        .insert(Lerper::default())
                        .id();
                    object_entity.world.insert(*object_id, entity);
                } else {
                    // TODO: represent other player objects
                }
            }
            SpawnType::TraitorCard(Object {
                id: object_id,
                inner: card,
            }) => {
                let card_face = asset_server.get_handle("card.gltf#Mesh0/Primitive0");
                let card_back = asset_server.get_handle("card.gltf#Mesh0/Primitive1");

                let traitor_front_texture = asset_server.get_handle(
                    format!(
                        "traitor/traitor_{}.png",
                        game_state.data.leaders[&card.leader].texture.as_str()
                    )
                    .as_str(),
                );

                let traitor_back_texture = asset_server.get_handle("traitor/traitor_back.png");

                let entity = commands
                    .spawn_bundle((*card, *object_id))
                    .insert_bundle(SpatialBundle::from_transform(
                        // TODO: stack them
                        Transform::from_translation(vec3(1.23, 0.0049, -0.3))
                            * Transform::from_rotation(Quat::from_rotation_z(PI)),
                    ))
                    .insert(Lerper::default())
                    .with_children(|parent| {
                        parent
                            .spawn_bundle(PbrBundle {
                                mesh: card_face.clone(),
                                material: materials.add(StandardMaterial::from(traitor_front_texture)),
                                ..default()
                            })
                            .insert_bundle(PickableBundle::default());
                        parent
                            .spawn_bundle(PbrBundle {
                                mesh: card_back.clone(),
                                material: materials.add(StandardMaterial::from(traitor_back_texture)),
                                ..default()
                            })
                            .insert_bundle(PickableBundle::default());
                    })
                    .id();
                object_entity.world.insert(*object_id, entity);
            }
            SpawnType::TreacheryCard(Object {
                id: object_id,
                inner: card,
            }) => {
                let card_face = asset_server.get_handle("card.gltf#Mesh0/Primitive0");
                let card_back = asset_server.get_handle("card.gltf#Mesh0/Primitive1");

                let treachery_front_texture = asset_server.get_handle(
                    format!(
                        "treachery/treachery_{}.png",
                        game_state.data.treachery_cards[&card.kind].textures[card.variant]
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
                    .insert(Lerper::default())
                    .with_children(|parent| {
                        parent
                            .spawn_bundle(PbrBundle {
                                mesh: card_face.clone(),
                                material: materials.add(StandardMaterial::from(treachery_front_texture)),
                                ..default()
                            })
                            .insert_bundle(PickableBundle::default());
                        parent
                            .spawn_bundle(PbrBundle {
                                mesh: card_back.clone(),
                                material: materials.add(StandardMaterial::from(treachery_back_texture)),
                                ..default()
                            })
                            .insert_bundle(PickableBundle::default());
                    })
                    .id();
                object_entity.world.insert(*object_id, entity);
            }
            SpawnType::SpiceCard(Object {
                id: object_id,
                inner: card,
            }) => {
                let card_face = asset_server.get_handle("card.gltf#Mesh0/Primitive0");
                let card_back = asset_server.get_handle("card.gltf#Mesh0/Primitive1");

                let spice_front_texture = asset_server
                    .get_handle(format!("spice/spice_{}.png", game_state.data.spice_cards[&card].texture).as_str());
                let spice_back_texture = asset_server.get_handle("spice/spice_back.png");

                let entity = commands
                    .spawn_bundle((*card, *object_id))
                    .insert_bundle(SpatialBundle {
                        transform: Transform::from_translation(vec3(1.23, 0.0049, 0.3))
                            * Transform::from_rotation(Quat::from_rotation_z(PI)),
                        ..default()
                    })
                    .insert(Lerper::default())
                    .with_children(|parent| {
                        parent
                            .spawn_bundle(PbrBundle {
                                mesh: card_face.clone(),
                                material: materials.add(StandardMaterial::from(spice_front_texture)),
                                ..default()
                            })
                            .insert_bundle(PickableBundle::default());
                        parent
                            .spawn_bundle(PbrBundle {
                                mesh: card_back.clone(),
                                material: materials.add(StandardMaterial::from(spice_back_texture)),
                                ..default()
                            })
                            .insert_bundle(PickableBundle::default());
                    })
                    .id();
                object_entity.world.insert(*object_id, entity);
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
                    .insert(Lerper::default())
                    .with_children(|parent| {
                        parent
                            .spawn_bundle(PbrBundle {
                                mesh: card_face.clone(),
                                material: materials.add(StandardMaterial::from(storm_front_texture)),
                                ..default()
                            })
                            .insert_bundle(PickableBundle::default());
                        parent
                            .spawn_bundle(PbrBundle {
                                mesh: card_back.clone(),
                                material: materials.add(StandardMaterial::from(storm_back_texture)),
                                ..default()
                            })
                            .insert_bundle(PickableBundle::default());
                    })
                    .id();
                object_entity.world.insert(*object_id, entity);
            }
            SpawnType::Worm { location, id } => todo!(),
        }
    }
}

fn hand(
    game_events: Res<GameEvents>,
    game_state: Res<GameState>,
    mut hand_cards: Query<&mut Lerper>,
    object_entity: Res<ObjectEntityMap>,
    my_id: Res<PlayerId>,
) {
    if let Some(GameEvent::DealCard { player_id, .. } | GameEvent::DiscardCard { player_id, .. }) = game_events.peek() {
        if *my_id == *player_id {
            if let Some(player) = game_state.players.get(&my_id) {
                let hand = player
                    .traitor_cards
                    .iter()
                    .map(|o| o.id)
                    .chain(player.treachery_cards.iter().map(|o| o.id))
                    .collect::<Vec<_>>();
                let hand_positions = hand_positions(hand.len());
                for (id, pos) in hand.into_iter().zip(hand_positions.into_iter()) {
                    if let Some(entity) = object_entity.world.get(&id) {
                        if let Some(mut lerper) = hand_cards.get_mut(*entity).ok() {
                            lerper.replace(Lerp::ui_to(
                                UITransform::from(pos).with_rotation(Quat::from_rotation_x(PI / 2.0)),
                                0.1,
                                0.0,
                            ));
                        }
                    }
                }
            } else {
                // TODO
            }
        }
    }
}

fn shuffle_traitors(game_events: Res<GameEvents>, mut commands: Commands, game_state: Res<GameState>) {
    // TODO
}

fn ship_troop_input(
    game_state: Res<GameState>,
    mut picked_events: EventReader<PickedEvent<LocationSector>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut client: ResMut<RenetClient>,
    my_id: Res<PlayerId>,
) {
    for PickedEvent { inner, .. } in picked_events.iter() {
        if let Some(player) = game_state.players.get(&my_id) {
            if !player.offworld_forces.is_empty() {
                // TODO: Maybe add modifiers to the PickedEvents somehow?
                if keyboard_input.pressed(KeyCode::LShift) {
                    if let Some(force) = player.offworld_forces.iter().find(|t| t.inner.is_special) {
                        let event = GameEvent::ShipForces {
                            to: *inner,
                            forces: hashset!(force.id),
                        };
                        client.send_event(event);
                    }
                } else if let Some(force) = player.offworld_forces.iter().find(|t| !t.inner.is_special) {
                    let event = GameEvent::ShipForces {
                        to: *inner,
                        forces: hashset!(force.id),
                    };
                    client.send_event(event);
                }
            }
        }
    }
}

fn ship_forces(
    game_events: Res<GameEvents>,
    game_state: Res<GameState>,
    object_entity: Res<ObjectEntityMap>,
    mut troops: Query<&mut Lerper, With<Troop>>,
) {
    if let Some(GameEvent::ShipForces { to, forces }) = game_events.peek() {
        let idx = game_state.board[&to.location].sectors[&to.sector].forces.len();
        let node = game_state.data.locations[&to.location].sectors[&to.sector].fighters[idx];
        for entity in forces.iter().filter_map(|id| object_entity.world.get(id)) {
            if let Ok(mut lerper) = troops.get_mut(*entity) {
                // TODO: stack
                lerper.replace(Lerp::world_to(
                    Transform::from_translation(Vec3::new(node.x, node.z, -node.y)),
                    0.1,
                    0.0,
                ));
            }
        }
    }
}

fn discard_card(
    game_events: Res<GameEvents>,
    mut commands: Commands,
    object_entity: Res<ObjectEntityMap>,
    mut cards: Query<&mut Lerper>,
    my_id: Res<PlayerId>,
) {
    if let Some(GameEvent::DiscardCard { player_id, card_id, to }) = game_events.peek() {
        if *my_id == *player_id {
            let entity = object_entity.world[&card_id];
            let transform = match to {
                DeckType::Traitor => Transform::from_translation(vec3(1.5, 0.0049, -0.3)),
                DeckType::Treachery => Transform::from_translation(vec3(1.5, 0.0049, -0.87)),
                DeckType::Storm => Transform::from_translation(vec3(1.5, 0.0049, 0.87)),
                DeckType::Spice => Transform::from_translation(vec3(1.5, 0.0049, 0.3)),
            };
            if let Ok(mut lerper) = cards.get_mut(entity) {
                lerper.replace(Lerp::world_to(transform, 0.1, 0.0));
            }
        } else {
            // TODO: do something else for other players
        }
    }
}
