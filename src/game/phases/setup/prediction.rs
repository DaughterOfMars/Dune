use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;
use iyes_loopless::prelude::IntoConditionalSystem;
use renet::RenetClient;

use super::SetupPhase;
use crate::{
    components::{Card, Faction, FactionPredictionCard, TurnPredictionCard},
    game::{
        state::{GameEvent, GameState, PlayerId, Prompt},
        Phase, PickedEvent,
    },
    lerper::{InterpolationFunction, Lerp, UITransform},
    network::SendGameEvent,
    GameEntity, Screen,
};

pub struct PredictionPlugin;

impl Plugin for PredictionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(present_factions.run_in_state(Screen::Game))
            .add_system(await_faction_pick.run_in_state(Screen::Game))
            .add_system(present_turns.run_in_state(Screen::Game))
            .add_system(await_turn_pick.run_in_state(Screen::Game));
    }
}

fn present_factions(
    mut commands: Commands,
    game_state: Res<GameState>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_id: Res<PlayerId>,
    mut client: ResMut<RenetClient>,
) {
    if matches!(game_state.phase, Phase::Setup(SetupPhase::Prediction))
        && game_state.active_player == Some(*player_id)
        && game_state
            .players
            .get(&player_id)
            .map(|p| p.prompt.is_none() && p.faction == Faction::BeneGesserit)
            .unwrap_or_default()
    {
        let card_face = asset_server.get_handle("card.gltf#Mesh0/Primitive0");
        let card_back = asset_server.get_handle("card.gltf#Mesh0/Primitive1");

        let prediction_back_texture = asset_server.get_handle("predictions/prediction_back.png");

        for (i, faction) in game_state.players.values().map(|player| player.faction).enumerate() {
            let prediction_front_texture =
                asset_server.get_handle(format!("predictions/prediction_{}.png", faction.code()).as_str());

            let node = game_state.data.prediction_nodes.factions[i];

            commands
                .spawn_bundle((Card, FactionPredictionCard { faction }))
                .insert(
                    Lerp::ui_from_to(
                        UITransform::default().with_rotation(Quat::from_rotation_x(PI / 2.0)),
                        UITransform::from(node).with_rotation(Quat::from_rotation_x(PI / 2.0)),
                        0.5,
                        0.03 * i as f32,
                    )
                    .with_interpolation(InterpolationFunction::Easing),
                )
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
        client.send_game_event(GameEvent::ShowPrompt {
            player_id: None,
            prompt: Prompt::FactionPrediction,
        });
    }
}

fn await_faction_pick(
    mut commands: Commands,
    game_state: Res<GameState>,
    mut picked_events: EventReader<PickedEvent<FactionPredictionCard>>,
    faction_cards: Query<Entity, With<FactionPredictionCard>>,
    player_id: Res<PlayerId>,
    mut client: ResMut<RenetClient>,
) {
    if matches!(game_state.phase, Phase::Setup(SetupPhase::Prediction))
        && game_state.active_player == Some(*player_id)
        && game_state
            .players
            .get(&player_id)
            .map(|p| matches!(p.prompt, Some(Prompt::FactionPrediction)) && p.faction == Faction::BeneGesserit)
            .unwrap_or_default()
    {
        for PickedEvent {
            picked: _,
            inner: FactionPredictionCard { faction },
        } in picked_events.iter()
        {
            for entity in faction_cards.iter() {
                // TODO: animate them away~
                commands.entity(entity).despawn_recursive();
            }
            client.send_game_event(GameEvent::MakeFactionPrediction { faction: *faction });
        }
    }
}

fn present_turns(
    mut commands: Commands,
    game_state: Res<GameState>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_id: Res<PlayerId>,
    mut client: ResMut<RenetClient>,
) {
    if matches!(game_state.phase, Phase::Setup(SetupPhase::Prediction))
        && game_state.active_player == Some(*player_id)
        && game_state
            .players
            .get(&player_id)
            .map(|p| p.prompt.is_none() && p.faction == Faction::BeneGesserit)
            .unwrap_or_default()
    {
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
                .insert(
                    Lerp::ui_from_to(
                        UITransform::default()
                            .with_rotation(Quat::from_rotation_x(PI / 2.0))
                            .with_scale(0.6),
                        UITransform::from(node)
                            .with_rotation(Quat::from_rotation_x(PI / 2.0))
                            .with_scale(0.6),
                        0.5,
                        0.01 * i as f32,
                    )
                    .with_interpolation(InterpolationFunction::Easing),
                )
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
                            material: materials.add(StandardMaterial::from(prediction_back_texture.clone())),
                            ..Default::default()
                        })
                        .insert_bundle(PickableBundle::default());
                });
        });
        client.send_game_event(GameEvent::ShowPrompt {
            player_id: None,
            prompt: Prompt::TurnPrediction,
        });
    }
}

fn await_turn_pick(
    mut commands: Commands,
    game_state: Res<GameState>,
    mut picked_events: EventReader<PickedEvent<TurnPredictionCard>>,
    turn_cards: Query<Entity, With<TurnPredictionCard>>,
    player_id: Res<PlayerId>,
    mut client: ResMut<RenetClient>,
) {
    if matches!(game_state.phase, Phase::Setup(SetupPhase::Prediction))
        && game_state.active_player == Some(*player_id)
        && game_state
            .players
            .get(&player_id)
            .map(|p| matches!(p.prompt, Some(Prompt::TurnPrediction)) && p.faction == Faction::BeneGesserit)
            .unwrap_or_default()
    {
        for PickedEvent {
            picked: _,
            inner: TurnPredictionCard { turn },
        } in picked_events.iter()
        {
            for entity in turn_cards.iter() {
                // TODO: animate them away~
                commands.entity(entity).despawn_recursive();
            }
            client.send_game_event(GameEvent::MakeTurnPrediction { turn: *turn });
        }
    }
}
