mod object;
pub mod phase;
pub mod state;

use std::f32::consts::PI;

use bevy::{math::vec3, prelude::*};
use bevy_mod_picking::{PickableBundle, PickingEvent};
use iyes_loopless::prelude::{AppLooplessStateExt, ConditionSet};
use maplit::hashset;
use renet::RenetClient;

pub use self::object::*;
use self::{
    phase::PhasePlugin,
    state::{DeckType, GameEvent, GameState, PlayerId, SpawnType},
};
use crate::{
    components::{FactionChoiceCard, FactionPredictionCard, LocationSector, TraitorCard, TurnPredictionCard},
    lerper::Lerp,
    network::SendEvent,
    Screen,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ObjectEntityMap>();

        app.add_event::<PickedEvent<FactionChoiceCard>>()
            .add_event::<PickedEvent<FactionPredictionCard>>()
            .add_event::<PickedEvent<TurnPredictionCard>>()
            .add_event::<PickedEvent<TraitorCard>>()
            .add_event::<PickedEvent<LocationSector>>();

        app.add_system_set_to_stage(
            CoreStage::PreUpdate,
            ConditionSet::new()
                .run_in_state(Screen::Game)
                .with_system(spawn_object)
                .into(),
        );

        app.add_system_set(
            ConditionSet::new()
                .run_in_state(Screen::Game)
                .with_system(hiararchy_picker::<FactionChoiceCard>)
                .with_system(hiararchy_picker::<FactionPredictionCard>)
                .with_system(hiararchy_picker::<TurnPredictionCard>)
                .with_system(hiararchy_picker::<TraitorCard>)
                .with_system(hiararchy_picker::<LocationSector>)
                .with_system(active_player_text)
                .with_system(shuffle_traitors)
                .with_system(ship_troop_input)
                .with_system(ship_forces)
                .with_system(discard_card)
                .into(),
        );

        app.add_plugin(PhasePlugin);

        app.add_exit_system(Screen::Game, reset);
    }
}

#[derive(Component)]
pub struct ActivePlayerText;

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
                        object_entity.world.insert(*object_id, entity);
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
                        let faction = game_state.players[player_id].faction;
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
}

pub fn deal_cards(mut commands: Commands, game_state: Res<GameState>, mut game_events: EventReader<GameEvent>) {
    todo!()
}

fn active_player_text(game_state: Res<GameState>, mut text: Query<&mut Text, With<ActivePlayerText>>) {
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

fn shuffle_traitors(mut commands: Commands, game_state: Res<GameState>, mut game_events: EventReader<GameEvent>) {
    for event in game_events.iter() {
        if matches!(
            event,
            GameEvent::SetDeckOrder {
                deck_order,
                deck_type: DeckType::Traitor
            }
        ) {
            // TODO
        }
    }
}

fn ship_troop_input(
    game_state: Res<GameState>,
    mut picked_events: EventReader<PickedEvent<LocationSector>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut client: ResMut<RenetClient>,
    my_id: Res<PlayerId>,
) {
    for PickedEvent { inner, .. } in picked_events.iter() {
        if Some(*my_id) == game_state.active_player {
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
}

fn ship_forces(
    mut commands: Commands,
    game_state: Res<GameState>,
    mut game_events: EventReader<GameEvent>,
    object_entity: Res<ObjectEntityMap>,
) {
    for event in game_events.iter() {
        if let GameEvent::ShipForces { to, forces } = event {
            let idx = game_state.board[&to.location].sectors[&to.sector].forces.len();
            let node = game_state.data.locations[&to.location].sectors[&to.sector].fighters[idx];
            for entity in forces.iter().filter_map(|id| object_entity.world.get(id)) {
                // TODO: stack
                commands.entity(*entity).insert(Lerp::world_to(
                    Transform::from_translation(Vec3::new(node.x, node.z, -node.y)),
                    0.1,
                    0.0,
                ));
            }
        }
    }
}

fn discard_card(
    mut commands: Commands,
    mut game_events: EventReader<GameEvent>,
    object_entity: Res<ObjectEntityMap>,
    my_id: Res<PlayerId>,
) {
    for event in game_events.iter() {
        if let GameEvent::DiscardCard { player_id, card_id, to } = event {
            if *my_id == *player_id {
                let entity = object_entity.world[card_id];
                let transform = match to {
                    DeckType::Traitor => {
                        Transform::from_translation(vec3(1.5, 0.0049, -0.3))
                            * Transform::from_rotation(Quat::from_rotation_z(PI))
                            * Transform::from_rotation(Quat::from_rotation_y(PI))
                    }
                    DeckType::Treachery => {
                        Transform::from_translation(vec3(1.5, 0.0049, -0.87))
                            * Transform::from_rotation(Quat::from_rotation_z(PI))
                            * Transform::from_rotation(Quat::from_rotation_y(PI))
                    }
                    DeckType::Storm => {
                        Transform::from_translation(vec3(1.5, 0.0049, 0.87))
                            * Transform::from_rotation(Quat::from_rotation_z(PI))
                            * Transform::from_rotation(Quat::from_rotation_y(PI))
                    }
                    DeckType::Spice => {
                        Transform::from_translation(vec3(1.5, 0.0049, 0.3))
                            * Transform::from_rotation(Quat::from_rotation_z(PI))
                            * Transform::from_rotation(Quat::from_rotation_y(PI))
                    }
                };
                commands.entity(entity).insert(Lerp::world_to(transform, 0.1, 0.0));
            } else {
                // TODO: do something else for other players
            }
        }
    }
}
