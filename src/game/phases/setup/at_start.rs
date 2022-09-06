use std::{
    collections::{HashMap, HashSet},
    f32::consts::PI,
};

use bevy::{
    math::{vec2, vec3},
    prelude::*,
};
use bevy_mod_picking::PickableBundle;
use iyes_loopless::{
    prelude::{AppLooplessStateExt, ConditionHelpers, IntoConditionalSystem},
    state::{CurrentState, NextState},
};

use super::SetupPhase;
use crate::{
    active::{Active, AdvanceActive},
    components::{
        Card, Deck, Disorganized, Faction, Location, LocationSector, Player, TraitorCard, TraitorDeck, TreacheryCard,
        TreacheryDeck, Troop, Unique,
    },
    game::{Phase, PickedEvent, Shuffling},
    lerper::{Lerp, UITransform},
    resources::{Data, Info},
    util::card_jitter,
    GameEntity, Screen,
};

pub struct AtStartPlugin;

impl Plugin for AtStartPlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(AtStartState::Positions);
        app.add_system(
            positions
                .run_in_state(Screen::Game)
                .run_in_state(Phase::Setup(SetupPhase::AtStart))
                .run_in_state(AtStartState::Positions),
        )
        .add_system(
            shuffle_traitors
                .run_in_state(Screen::Game)
                .run_in_state(Phase::Setup(SetupPhase::AtStart))
                .run_in_state(AtStartState::ShuffleTraitors),
        )
        .add_system(
            deal_traitors
                .run_in_state(Screen::Game)
                .run_in_state(Phase::Setup(SetupPhase::AtStart))
                .run_in_state(AtStartState::DealTraitors),
        )
        .add_system(
            await_traitor_picks
                .run_in_state(Screen::Game)
                .run_in_state(Phase::Setup(SetupPhase::AtStart))
                .run_in_state(AtStartState::AwaitTraitorPicks),
        )
        .add_system(
            enable_force_positions
                .run_in_state(Screen::Game)
                .run_in_state(Phase::Setup(SetupPhase::AtStart))
                .run_in_state(AtStartState::EnableForcePositions),
        )
        .add_system(
            await_force_placement
                .run_in_state(Screen::Game)
                .run_in_state(Phase::Setup(SetupPhase::AtStart))
                .run_in_state(AtStartState::AwaitForcePlacement),
        )
        .add_system(
            deal_treachery
                .run_in_state(Screen::Game)
                .run_in_state(Phase::Setup(SetupPhase::AtStart))
                .run_in_state(AtStartState::DealTreachery),
        );
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum AtStartState {
    Positions,
    ShuffleTraitors,
    DealTraitors,
    AwaitTraitorPicks,
    EnableForcePositions,
    AwaitForcePlacement,
    DealTreachery,
}

fn positions(
    mut commands: Commands,
    data: Res<Data>,
    info: Res<Info>,
    players: Query<&Faction, With<Player>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (i, turn) in info.turn_order.iter().enumerate() {
        let faction = players.get(*turn).unwrap();
        let little_token = asset_server.get_handle("little_token.gltf#Mesh0/Primitive0");
        let logo_texture = asset_server.get_handle(format!("tokens/{}_logo.png", faction.code()).as_str());
        commands
            .spawn_bundle(SpatialBundle::from_transform(Transform::from_translation(
                data.token_nodes.factions[i],
            )))
            .insert(GameEntity)
            .insert_bundle(PbrBundle {
                mesh: little_token.clone(),
                material: materials.add(StandardMaterial::from(logo_texture)),
                ..Default::default()
            });
    }
    commands.insert_resource(NextState(AtStartState::ShuffleTraitors));
}

fn shuffle_traitors(
    mut commands: Commands,
    data: Res<Data>,
    players: Query<&Faction, With<Player>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let card_face = asset_server.get_handle("card.gltf#Mesh0/Primitive0");
    let card_back = asset_server.get_handle("card.gltf#Mesh0/Primitive1");

    let traitor_back_texture = asset_server.get_handle("traitor/traitor_back.png");

    commands
        .spawn_bundle((Deck, TraitorDeck, Shuffling(5)))
        .insert_bundle(SpatialBundle::from_transform(
            Transform::from_translation(vec3(1.23, 0.0049, -0.3)) * Transform::from_rotation(Quat::from_rotation_z(PI)),
        ))
        .with_children(|parent| {
            let factions_in_play = players.iter().copied().collect::<HashSet<_>>();
            for (i, (leader, leader_data)) in data
                .leaders
                .iter()
                .filter(|(_, l)| factions_in_play.contains(&l.faction))
                .enumerate()
            {
                let traitor_front_texture =
                    asset_server.get_handle(format!("traitor/traitor_{}.png", leader_data.texture.as_str()).as_str());
                let traitor_front_material = materials.add(StandardMaterial::from(traitor_front_texture));

                parent
                    .spawn_bundle((Card, TraitorCard { leader: leader.clone() }))
                    .insert_bundle(SpatialBundle::from_transform(
                        Transform::from_translation(Vec3::Y * 0.001 * (i as f32)) * card_jitter(),
                    ))
                    .insert(GameEntity)
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
    commands.insert_resource(NextState(AtStartState::DealTraitors));
}

fn deal_traitors(
    mut commands: Commands,
    info: Res<Info>,
    traitor_deck: Query<&Children, With<TraitorDeck>>,
    traitor_cards: Query<(Entity, &Transform), With<TraitorCard>>,
) {
    let nodes = [vec2(-0.6, 0.0), vec2(-0.2, 0.0), vec2(0.2, 0.0), vec2(0.6, 0.0)];
    let mut cards = traitor_deck
        .single()
        .iter()
        .filter_map(|e| traitor_cards.get(*e).ok())
        .collect::<Vec<_>>();
    cards.sort_unstable_by(|(_, transform1), (_, transform2)| {
        transform1
            .translation
            .y
            .partial_cmp(&transform2.translation.y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for (i, player) in std::iter::repeat(info.turn_order.iter())
        .take(4)
        .enumerate()
        .map(|(i, iter)| iter.map(move |v| (i, v)))
        .flatten()
    {
        commands
            .entity(cards.pop().unwrap().0)
            .insert(Lerp::world_to_ui(
                UITransform::from(nodes[i]).with_rotation(Quat::from_rotation_x(PI / 2.0)),
                *player,
                0.5,
                0.03 * i as f32,
            ))
            .insert(Unique::new(*player))
            .insert_bundle(PickableBundle::default());
    }
    commands.insert_resource(NextState(AtStartState::AwaitTraitorPicks));
}

fn await_traitor_picks(
    mut commands: Commands,
    mut picked_events: EventReader<PickedEvent<TraitorCard>>,
    mut players: Query<(&mut Player, &Faction)>,
    traitor_cards: Query<(Entity, &Unique), With<TraitorCard>>,
) {
    for PickedEvent {
        picker,
        picked,
        inner: card,
    } in picked_events.iter()
    {
        let (mut player, faction) = players.get_mut(*picker).unwrap();
        // TODO: Probably should just auto pick for harkonnen
        let max_picks = match faction {
            Faction::Harkonnen => 4,
            _ => 1,
        };
        player.traitor_cards.push(*card);
        commands.entity(*picked).despawn_recursive();
        if player.traitor_cards.len() == max_picks {
            for (entity, _) in traitor_cards.iter().filter(|(_, unique)| unique.entity == *picker) {
                // TODO: animate them away~
                commands.entity(entity).despawn_recursive();
            }
            commands.insert_resource(AdvanceActive);
        }
        if players.iter().all(|(player, faction)| {
            player.traitor_cards.len()
                == match faction {
                    Faction::Harkonnen => 4,
                    _ => 1,
                }
        }) {
            commands.insert_resource(NextState(AtStartState::EnableForcePositions));
        }
    }
}

fn enable_force_positions(
    mut commands: Commands,
    active: Res<Active>,
    players: Query<&Faction, With<Player>>,
    locations: Query<(Entity, &LocationSector)>,
    deployed_troops: Query<Entity, (With<Troop>, With<Location>)>,
    data: Res<Data>,
) {
    if deployed_troops.iter().count() as u8
        == data
            .factions
            .iter()
            .filter(|(faction, _)| players.iter().find(|f| f == faction).is_some())
            .fold(0, |mut count, (_, faction_data)| {
                count += faction_data.starting_values.units;
                count
            })
    {
        commands.insert_resource(NextState(AtStartState::DealTreachery));
    } else {
        let faction = players.get(active.entity).unwrap();
        let faction_data = data.factions.get(faction).unwrap();
        if faction_data.starting_values.units > 0 {
            if let Some(possible_locations) = faction_data.starting_values.possible_locations.as_ref() {
                for (entity, _) in locations
                    .iter()
                    .filter(|(_, l)| possible_locations.contains(&l.location))
                {
                    commands.entity(entity).insert_bundle(PickableBundle::default());
                }
            } else {
                for (entity, _) in locations.iter() {
                    commands.entity(entity).insert_bundle(PickableBundle::default());
                }
            }
            commands.insert_resource(NextState(AtStartState::AwaitForcePlacement));
        } else {
            commands.insert_resource(AdvanceActive);
        }
    }
}

fn await_force_placement(
    mut commands: Commands,
    mut picked_events: EventReader<PickedEvent<LocationSector>>,
    players: Query<&Faction, With<Player>>,
    offworld_troops: Query<(Entity, &Unique), (With<Troop>, Without<Location>)>,
    deployed_troops: Query<&Unique, (With<Troop>, With<Location>)>,
    data: Res<Data>,
) {
    let mut troops_map = offworld_troops
        .iter()
        .fold(HashMap::<_, Vec<_>>::new(), |mut map, (entity, unique)| {
            map.entry(unique.entity).or_default().push(entity);
            map
        });
    for PickedEvent { picker, picked, inner } in picked_events.iter() {
        let deployed_count = deployed_troops.iter().filter(|unique| unique.entity == *picker).count() + 1;
        commands
            .entity(troops_map.get_mut(picker).unwrap().pop().unwrap())
            .insert(inner.location);
        commands.entity(*picked).insert(Disorganized);
        if deployed_count as u8
            == data
                .factions
                .get(&players.get(*picker).unwrap())
                .unwrap()
                .starting_values
                .units
        {
            commands.insert_resource(AdvanceActive);
            commands.insert_resource(NextState(AtStartState::EnableForcePositions));
        }
    }
}

fn deal_treachery(
    mut commands: Commands,
    info: Res<Info>,
    treachery_deck: Query<&Children, With<TreacheryDeck>>,
    treachery_cards: Query<(Entity, &Transform, &TreacheryCard)>,
    phase: Res<CurrentState<Phase>>,
) {
    let mut cards = treachery_deck
        .single()
        .iter()
        .map(|e| treachery_cards.get(*e).unwrap())
        .collect::<Vec<_>>();
    cards.sort_unstable_by(|(_, transform1, _), (_, transform2, _)| {
        transform1
            .translation
            .y
            .partial_cmp(&transform2.translation.y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for player in info.turn_order.iter() {
        let (entity, _, card) = cards.pop().unwrap();
        // TODO: animate this or something
        commands.entity(*player).insert(*card);
        commands.entity(entity).despawn_recursive();
    }
    commands.insert_resource(NextState(phase.0.next()));
}
