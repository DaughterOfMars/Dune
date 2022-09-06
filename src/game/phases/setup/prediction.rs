use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;
use iyes_loopless::{
    prelude::{ConditionHelpers, IntoConditionalSystem},
    state::{CurrentState, NextState},
};

use super::{FactionPickedEvent, SetupPhase, TurnPickedEvent};
use crate::{
    components::{Card, Faction, FactionPredictionCard, Player, TurnPredictionCard, Unique},
    game::Phase,
    lerper::{InterpolationFunction, Lerp, UITransform},
    resources::Data,
    GameEntity, NextActive, Screen,
};

pub struct PredictionPlugin;

impl Plugin for PredictionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            prediction_step
                .run_in_state(Screen::Game)
                .run_in_state(Phase::Setup(SetupPhase::Prediction)),
        );
    }
}

#[derive(Default)]
struct PredictionStep {
    predicted_faction: Option<Faction>,
    faction_cards: Vec<Entity>,
    predicted_turn: Option<u8>,
    turn_cards: Vec<Entity>,
}

fn prediction_step(
    mut commands: Commands,
    data: Res<Data>,
    phase: Res<CurrentState<Phase>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    players: Query<(Entity, &Faction), With<Player>>,
    mut faction_picked_events: EventReader<FactionPickedEvent>,
    mut turn_picked_events: EventReader<TurnPickedEvent>,
    mut state: Local<PredictionStep>,
) {
    if let Some((bg_player, _)) = players.iter().find(|(_, faction)| **faction == Faction::BeneGesserit) {
        let card_face = asset_server.get_handle("card.gltf#Mesh0/Primitive0");
        let card_back = asset_server.get_handle("card.gltf#Mesh0/Primitive1");

        let prediction_back_texture = asset_server.get_handle("predictions/prediction_back.png");

        if state.predicted_faction.is_none() {
            if state.faction_cards.is_empty() {
                commands.insert_resource(NextActive { entity: bg_player });
                let factions_in_play = players.iter().map(|(_, faction)| *faction).collect::<Vec<_>>();
                for (i, faction) in factions_in_play.into_iter().enumerate() {
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
                                    bg_player,
                                    0.5,
                                    0.03 * i as f32,
                                )
                                .with_interpolation(InterpolationFunction::Easing),
                            )
                            .insert(Unique::new(Faction::BeneGesserit))
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
                            })
                            .id(),
                    );
                }
            } else {
                for FactionPickedEvent { entity, faction } in faction_picked_events.iter() {
                    if bg_player == *entity {
                        state.predicted_faction.replace(*faction);
                        for entity in state.faction_cards.drain(..) {
                            // TODO: animate them away~
                            commands.entity(entity).despawn_recursive();
                        }
                        break;
                    }
                }
            }
        } else if state.predicted_turn.is_none() {
            if state.turn_cards.is_empty() {
                (1..=15).for_each(|turn| {
                    let prediction_front_texture =
                        asset_server.get_handle(format!("predictions/prediction_t{}.png", turn).as_str());

                    let i = turn as usize - 1;
                    let node = data.prediction_nodes.turns[i];
                    state.turn_cards.push(
                        commands
                            .spawn_bundle(SpatialBundle::default())
                            .insert(
                                Lerp::ui_from_to(
                                    UITransform::default()
                                        .with_rotation(Quat::from_rotation_x(PI / 2.0))
                                        .with_scale(0.6),
                                    UITransform::from(node)
                                        .with_rotation(Quat::from_rotation_x(PI / 2.0))
                                        .with_scale(0.6),
                                    bg_player,
                                    0.5,
                                    0.01 * i as f32,
                                )
                                .with_interpolation(InterpolationFunction::Easing),
                            )
                            .insert(Unique::new(Faction::BeneGesserit))
                            .insert(GameEntity)
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
                            })
                            .id(),
                    );
                });
            } else {
                for TurnPickedEvent { entity, turn } in turn_picked_events.iter() {
                    if bg_player == *entity {
                        state.predicted_turn.replace(*turn);
                        for entity in state.turn_cards.drain(..) {
                            // TODO: animate them away~
                            commands.entity(entity).despawn_recursive();
                        }
                        break;
                    }
                }
            }
        } else {
            commands.insert_resource(NextState(phase.0.next()));
        }
    } else {
        commands.insert_resource(NextState(phase.0.next()));
    }
}
