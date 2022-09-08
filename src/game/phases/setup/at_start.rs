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
    components::{
        Card, Deck, Disorganized, Faction, Location, LocationSector, TraitorCard, TraitorDeck, TreacheryCard,
        TreacheryDeck, Troop,
    },
    game::{Phase, PickedEvent, Shuffling},
    lerper::{Lerp, UITransform},
    util::card_jitter,
    GameEntity, Screen,
};

pub struct AtStartPlugin;

impl Plugin for AtStartPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(positions.run_in_state(Screen::Game))
            .add_system(shuffle_traitors.run_in_state(Screen::Game))
            .add_system(deal_traitors.run_in_state(Screen::Game))
            .add_system(await_traitor_picks.run_in_state(Screen::Game))
            .add_system(enable_force_positions.run_in_state(Screen::Game))
            .add_system(await_force_placement.run_in_state(Screen::Game))
            .add_system(deal_treachery.run_in_state(Screen::Game));
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
    game_state: Res<GameState>,
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
    game_state: Res<GameState>,
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
