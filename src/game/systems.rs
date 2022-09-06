use bevy::render::view::RenderLayers;
use iyes_loopless::state::CurrentState;

use super::*;
use crate::{
    components::{Faction, Location},
    Active,
};

pub fn trigger_stack_troops(
    data: Res<Data>,
    mut commands: Commands,
    troops: Query<(Entity, &Unique, &Troop, Option<&Location>)>,
    locations: Query<(Entity, &LocationSector), With<Disorganized>>,
) {
    for (loc_entity, loc_sec) in locations.iter() {
        let mut map = HashMap::new();
        for (entity, faction) in troops.iter().filter_map(|(entity, unique, troop, location)| {
            location.and_then(|location| {
                if *location == loc_sec.location {
                    Some((entity, unique.entity))
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

pub fn public_troop_system(mut troops: Query<(&Troop, &mut Unique, Option<&Location>), Changed<Troop>>) {
    for (troop, mut unique, location) in troops.iter_mut() {
        unique.public = location.is_some();
    }
}

pub fn phase_text_system(phase: Res<CurrentState<Phase>>, mut text: Query<&mut Text, With<PhaseText>>) {
    if phase.is_changed() {
        let s = match phase.0 {
            Phase::Setup(subphase) => match subphase {
                SetupPhase::ChooseFactions => "Choosing Factions...".to_string(),
                SetupPhase::Prediction => "Bene Gesserit are making a prediction...".to_string(),
                SetupPhase::AtStart => "Initial Placement...".to_string(),
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

pub fn active_player_text_system(
    data: Res<Data>,
    active: Res<Active>,
    players: Query<&Faction, With<Player>>,
    mut text: Query<&mut Text, With<ActivePlayerText>>,
) {
    if active.is_changed() {
        text.single_mut().sections[0].value = players
            .get(active.entity)
            .ok()
            .and_then(|e| data.factions.get(e))
            .map(|f| f.name.clone())
            .unwrap_or(format!("{:?}", active.entity));
    }
}

pub fn get_initial_spice() {
    todo!();
}

pub fn render_unique(
    mut commands: Commands,
    mut uniques: Query<(Entity, &Unique), Changed<Unique>>,
    players: Query<&RenderLayers, (With<Player>, With<Camera>)>,
) {
    for (entity, unique) in uniques.iter_mut() {
        if unique.public {
            commands.entity(entity).insert(RenderLayers::default());
        } else {
            if let Ok(layer) = players.get(unique.entity) {
                commands.entity(entity).insert(layer.without(0));
            }
        }
    }
}
