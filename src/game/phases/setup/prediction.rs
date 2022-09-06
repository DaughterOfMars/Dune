use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;
use iyes_loopless::{
    prelude::{AppLooplessStateExt, ConditionHelpers, IntoConditionalSystem},
    state::{CurrentState, NextState},
};

use super::SetupPhase;
use crate::{
    active::{Active, NextActive},
    components::{Card, Faction, FactionPredictionCard, Player, TurnPredictionCard, Unique},
    game::{Phase, PickedEvent},
    lerper::{InterpolationFunction, Lerp, UITransform},
    resources::{Data, Info},
    GameEntity, Screen,
};

pub struct PredictionPlugin;

impl Plugin for PredictionPlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(PredictionState::CheckBGPlaying);
        app.add_system(
            check_bg_playing
                .run_in_state(Screen::Game)
                .run_in_state(Phase::Setup(SetupPhase::Prediction))
                .run_in_state(PredictionState::CheckBGPlaying),
        )
        .add_system(
            present_factions
                .run_in_state(Screen::Game)
                .run_in_state(Phase::Setup(SetupPhase::Prediction))
                .run_in_state(PredictionState::PresentFactions),
        )
        .add_system(
            await_faction_pick
                .run_in_state(Screen::Game)
                .run_in_state(Phase::Setup(SetupPhase::Prediction))
                .run_in_state(PredictionState::AwaitFactionPick),
        )
        .add_system(
            present_turns
                .run_in_state(Screen::Game)
                .run_in_state(Phase::Setup(SetupPhase::Prediction))
                .run_in_state(PredictionState::PresentTurns),
        )
        .add_system(
            await_turn_pick
                .run_in_state(Screen::Game)
                .run_in_state(Phase::Setup(SetupPhase::Prediction))
                .run_in_state(PredictionState::AwaitTurnPick),
        );
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum PredictionState {
    CheckBGPlaying,
    PresentFactions,
    AwaitFactionPick,
    PresentTurns,
    AwaitTurnPick,
}

fn check_bg_playing(
    mut commands: Commands,
    players: Query<(Entity, &Faction), With<Player>>,
    phase: Res<CurrentState<Phase>>,
) {
    if let Some((bg_player, _)) = players.iter().find(|(_, faction)| **faction == Faction::BeneGesserit) {
        commands.insert_resource(NextActive { entity: bg_player });
        commands.insert_resource(NextState(PredictionState::PresentFactions));
    } else {
        commands.insert_resource(NextState(phase.0.next()));
    }
}

fn present_factions(
    mut commands: Commands,
    data: Res<Data>,
    active: Res<Active>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    players: Query<(Entity, &Faction), With<Player>>,
) {
    let card_face = asset_server.get_handle("card.gltf#Mesh0/Primitive0");
    let card_back = asset_server.get_handle("card.gltf#Mesh0/Primitive1");

    let prediction_back_texture = asset_server.get_handle("predictions/prediction_back.png");

    for (i, faction) in players.iter().map(|(_, faction)| *faction).enumerate() {
        let prediction_front_texture =
            asset_server.get_handle(format!("predictions/prediction_{}.png", faction.code()).as_str());

        let node = data.prediction_nodes.factions[i];

        commands
            .spawn_bundle((Card, FactionPredictionCard { faction }))
            .insert(
                Lerp::ui_from_to(
                    UITransform::default().with_rotation(Quat::from_rotation_x(PI / 2.0)),
                    UITransform::from(node).with_rotation(Quat::from_rotation_x(PI / 2.0)),
                    active.entity,
                    0.5,
                    0.03 * i as f32,
                )
                .with_interpolation(InterpolationFunction::Easing),
            )
            .insert(Unique::new(active.entity))
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
    commands.insert_resource(NextState(PredictionState::AwaitFactionPick));
}

fn await_faction_pick(
    mut commands: Commands,
    mut picked_events: EventReader<PickedEvent<FactionPredictionCard>>,
    faction_cards: Query<(Entity, &Unique), With<FactionPredictionCard>>,
) {
    for PickedEvent {
        picker,
        picked: _,
        inner: FactionPredictionCard { faction },
    } in picked_events.iter()
    {
        commands
            .entity(*picker)
            .insert(FactionPredictionCard { faction: *faction });
        for (entity, _) in faction_cards.iter().filter(|(_, unique)| unique.entity == *picker) {
            // TODO: animate them away~
            commands.entity(entity).despawn_recursive();
        }
        commands.insert_resource(NextState(PredictionState::PresentTurns));
    }
}

fn present_turns(
    mut commands: Commands,
    data: Res<Data>,
    active: Res<Active>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let card_face = asset_server.get_handle("card.gltf#Mesh0/Primitive0");
    let card_back = asset_server.get_handle("card.gltf#Mesh0/Primitive1");

    let prediction_back_texture = asset_server.get_handle("predictions/prediction_back.png");

    (1..=15).for_each(|turn| {
        let prediction_front_texture =
            asset_server.get_handle(format!("predictions/prediction_t{}.png", turn).as_str());

        let i = turn as usize - 1;
        let node = data.prediction_nodes.turns[i];

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
                    active.entity,
                    0.5,
                    0.01 * i as f32,
                )
                .with_interpolation(InterpolationFunction::Easing),
            )
            .insert(Unique::new(active.entity))
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
    commands.insert_resource(NextState(PredictionState::AwaitTurnPick));
}

fn await_turn_pick(
    mut commands: Commands,
    mut picked_events: EventReader<PickedEvent<TurnPredictionCard>>,
    phase: Res<CurrentState<Phase>>,
    turn_cards: Query<(Entity, &Unique), With<TurnPredictionCard>>,
    info: Res<Info>,
) {
    for PickedEvent {
        picker,
        picked: _,
        inner: TurnPredictionCard { turn },
    } in picked_events.iter()
    {
        commands.entity(*picker).insert(TurnPredictionCard { turn: *turn });
        for (entity, _) in turn_cards.iter().filter(|(_, unique)| unique.entity == *picker) {
            // TODO: animate them away~
            commands.entity(entity).despawn_recursive();
        }
        commands.insert_resource(NextState(PredictionState::CheckBGPlaying));
        commands.insert_resource(NextActive {
            entity: info.turn_order[0],
        });
        commands.insert_resource(NextState(phase.0.next()))
    }
}
