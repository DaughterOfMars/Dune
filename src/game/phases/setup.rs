use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_mod_picking::{PickableBundle, PickingEvent};
use iyes_loopless::{
    prelude::{ConditionHelpers, IntoConditionalSystem},
    state::{CurrentState, NextState},
};

use crate::{
    components::{Card, Faction, FactionPredictionCard, Player},
    game::Phase,
    lerper::{Lerp, UITransform},
    resources::Data,
    Active, Screen, MAX_PLAYERS,
};

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
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

#[derive(Default)]
struct ChooseFactionsStep {
    faction_cards: Vec<Entity>,
    turn: u8,
}

fn pick_factions_step(
    mut commands: Commands,
    data: Res<Data>,
    mut active: ResMut<Active>,
    phase: Res<CurrentState<Phase>>,
    to_pick: Query<(Entity, &Player), Without<Faction>>,
    picked: Query<(&Faction, &Player)>,
    cameras: Query<(Entity, &Player), With<Camera>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: Local<ChooseFactionsStep>,
) {
    let factions = vec![
        Faction::Atreides,
        Faction::BeneGesserit,
        Faction::Emperor,
        Faction::Fremen,
        Faction::Harkonnen,
        Faction::SpacingGuild,
    ];
    if state.faction_cards.is_empty() {
        for turn in state.turn + 1..MAX_PLAYERS + 1 {
            if let Some((cam_entity, _)) = cameras.iter().find(|(_, player)| turn == player.turn_order) {
                for (i, faction) in factions.into_iter().enumerate() {
                    let card_face = asset_server.get_handle("card.gltf#Mesh0/Primitive0");
                    let card_back = asset_server.get_handle("card.gltf#Mesh0/Primitive1");
                    let prediction_back_texture = asset_server.get_handle("treachery/treachery_back.png");
                    let prediction_front_texture =
                        asset_server.get_handle(format!("predictions/prediction_{}.png", faction.code()).as_str());

                    let node = data.prediction_nodes.factions[i];
                    info!("Spawning faction cards");
                    state.faction_cards.push(
                        commands
                            .spawn_bundle((Card, FactionPredictionCard { faction }))
                            .insert(Lerp::ui_from_to(
                                UITransform::default().with_rotation(Quat::from_rotation_x(PI / 2.0)),
                                UITransform::from(node).with_rotation(Quat::from_rotation_x(PI / 2.0)),
                                cam_entity,
                                1.0,
                                0.05 * i as f32,
                            ))
                            .insert_bundle(SpatialBundle::default())
                            .insert_bundle(PickableBundle::default())
                            .with_children(|parent| {
                                parent.spawn_bundle(PbrBundle {
                                    mesh: card_face.clone(),
                                    material: materials.add(StandardMaterial::from(prediction_front_texture)),
                                    ..default()
                                });
                                parent.spawn_bundle(PbrBundle {
                                    mesh: card_back.clone(),
                                    material: materials.add(StandardMaterial::from(prediction_back_texture)),
                                    ..default()
                                });
                            })
                            .id(),
                    );
                }
                state.turn = turn;
                break;
            }
        }
    } else {
        if picked
            .iter()
            .find(|(_, player)| state.turn == player.turn_order)
            .is_some()
        {
            // TODO: animate out cards
            state.faction_cards.clear();
            if state.turn >= MAX_PLAYERS {
                let next_phase = phase.0.next();
                commands.insert_resource(NextState(next_phase));
            }
        } else if let Some((entity, _)) = to_pick.iter().find(|(_, player)| state.turn == player.turn_order) {
            active.entity = entity;
        }
    }
}

fn faction_card_picker(
    mut commands: Commands,
    active: Res<Active>,
    faction_cards: Query<(&FactionPredictionCard, Option<&Lerp>)>,
    players: Query<Entity, (With<Player>, With<Camera>, Without<Faction>)>,
    mut events: EventReader<PickingEvent>,
) {
    for event in events.iter() {
        if let PickingEvent::Clicked(clicked) = event {
            if let Ok((faction_card, lerp)) = faction_cards.get(*clicked) {
                if lerp.is_none() {
                    if let Ok(active_player) = players.get(active.entity) {
                        commands.entity(active_player).insert(faction_card.faction.clone());
                        break;
                    }
                }
            }
        }
    }
}
