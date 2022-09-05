use bevy::render::view::RenderLayers;
use iyes_loopless::state::CurrentState;

use super::*;
use crate::{components::Faction, data::FactionStartingValues, Active};

pub fn trigger_stack_troops(
    data: Res<Data>,
    mut commands: Commands,
    troops: Query<(Entity, &Unique, &Troop)>,
    locations: Query<(Entity, &LocationSector), With<Disorganized>>,
) {
    for (loc_entity, loc_sec) in locations.iter() {
        let mut map = HashMap::new();
        for (entity, faction) in troops.iter().filter_map(|(entity, unique, troop)| {
            troop.location.and_then(|location| {
                if location == loc_entity {
                    Some((entity, unique.faction))
                } else {
                    None
                }
            })
        }) {
            map.entry(faction).or_insert(Vec::new()).push(entity);
        }
        for (node_ind, troops) in map.values().enumerate() {
            let location_data = data.locations.get(&loc_sec.location).unwrap();
            let node = location_data.sectors[&loc_sec.sector].fighters[node_ind];
            for (i, entity) in troops.iter().enumerate() {
                commands.entity(*entity).insert(Lerp::world_to(
                    Transform::from_translation(Vec3::new(node.x, node.z, -node.y))
                        * Transform::from_translation(i as f32 * 0.0018 * Vec3::Y),
                    0.1,
                    0.0,
                ));
            }
        }
        commands.entity(loc_entity).remove::<Disorganized>();
    }
}

pub fn public_troop_system(mut troops: Query<(&Troop, &mut Unique), Changed<Troop>>) {
    for (troop, mut unique) in troops.iter_mut() {
        unique.public = troop.location.is_some();
    }
}

// pub fn active_player(
// info: Res<Info>,
// players: Query<&Player>,
// mut uniques: Query<(&mut Visible, &Unique)>,
// ) {
// let entity = info.get_active_player();
// let active_player_faction = players.get(entity).unwrap().faction;
// for (mut visible, unique) in uniques.iter_mut() {
// if visible.is_visible != (unique.public || unique.faction == active_player_faction) {
// visible.is_visible = unique.public || unique.faction == active_player_faction;
// }
// }
// }

pub fn phase_text_system(
    phase: Res<CurrentState<Phase>>,
    data: Res<Data>,
    active: Res<Active>,
    players: Query<&Faction, With<Player>>,
    mut text: Query<&mut Text, With<PhaseText>>,
) {
    if phase.is_changed() {
        let s = match phase.0 {
            Phase::Setup(subphase) => match subphase {
                SetupPhase::ChooseFactions => "Choosing Factions...".to_string(),
                SetupPhase::Prediction => "Bene Gesserit are making a prediction...".to_string(),
                SetupPhase::AtStart => format!(
                    "{:?} Initial Placement...",
                    data.factions.get(players.get(active.entity).unwrap()).unwrap().name
                ),
                SetupPhase::DealTraitors => "Dealing Traitor Cards...".to_string(),
                SetupPhase::PickTraitors => "Picking Traitors...".to_string(),
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

pub fn get_initial_spice() {
    todo!();
}

pub fn place_troops(
    info: Res<Info>,
    players: Query<(Entity, &Faction), With<Player>>,
    data: Res<Data>,
    clickable_locations: Query<(Entity, &LocationSector)>,
    mut troops: Query<(Entity, &mut Troop, &Unique, &Transform)>,
    cameras: Query<Entity, (With<Camera>, Without<OrthographicProjection>)>,
) {
    println!("Enter: place_troops");
    let clickables = clickable_locations.iter().map(|(entity, _)| entity).collect::<Vec<_>>();

    for (entity, faction) in players.iter() {
        let FactionStartingValues {
            units,
            possible_locations,
            spice,
        } = &data.factions.get(faction).unwrap().starting_values;

        if *units > 0 {
            if let Some(locations) = possible_locations {
                if locations.len() == 0 {
                    // Do nothing
                } else if locations.len() == 1 {
                    // Auto place
                    let (location, loc_sec) = clickable_locations
                        .iter()
                        .find(|(_, loc_sec)| loc_sec.location == locations[0])
                        .unwrap();
                    let mut troop_stack = troops
                        .iter_mut()
                        .filter(|(_, troop, unique, _)| &unique.faction == faction && troop.location.is_none())
                        .collect::<Vec<_>>();
                    troop_stack.sort_by(|(_, _, _, transform1), (_, _, _, transform2)| {
                        transform1.translation.y.partial_cmp(&transform2.translation.y).unwrap()
                    });
                    todo!("lerp in the units")
                } else {
                    // Let the player pick
                    // Highlight the possible locations
                    todo!()
                };
            } else {
                todo!("go to next player")
            }
        } else {
            todo!("go to next player")
        }
    }

    // Move the camera so we can see the board good
    // enable clickables
}

pub fn render_unique(
    mut commands: Commands,
    mut uniques: Query<(Entity, &Unique), Changed<Unique>>,
    players: Query<(&Player, &Faction)>,
) {
    for (entity, unique) in uniques.iter_mut() {
        if unique.public {
            commands.entity(entity).insert(RenderLayers::default());
        } else {
            if let Some((player, _)) = players.iter().find(|(_, faction)| *faction == &unique.faction) {
                commands.entity(entity).insert(RenderLayers::layer(player.turn_order));
            }
        }
    }
}
