use std::{collections::HashSet, f32::consts::PI};

use bevy::{math::vec3, prelude::*};
use bevy_mod_picking::PickableBundle;
use iyes_loopless::{
    prelude::{AppLooplessStateExt, ConditionHelpers, IntoConditionalSystem},
    state::{CurrentState, NextState},
};
use maplit::hashset;

use super::SetupPhase;
use crate::{
    active::AdvanceActive,
    components::{Card, Faction, FactionPredictionCard, Player, Spice, Troop, Unique},
    game::{Phase, PickedEvent},
    lerper::{InterpolationFunction, Lerp, UITransform},
    resources::Data,
    util::divide_spice,
    Active, GameEntity, Screen,
};

pub struct ChooseFactionsPlugin;

impl Plugin for ChooseFactionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(ChooseFactionsState::PresentFactions);
        app.add_system(
            present_factions
                .run_in_state(Screen::Game)
                .run_in_state(Phase::Setup(SetupPhase::ChooseFactions))
                .run_in_state(ChooseFactionsState::PresentFactions),
        )
        .add_system(
            await_pick
                .run_in_state(Screen::Game)
                .run_in_state(Phase::Setup(SetupPhase::ChooseFactions))
                .run_in_state(ChooseFactionsState::AwaitPick),
        );
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum ChooseFactionsState {
    PresentFactions,
    AwaitPick,
}

fn present_factions(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    active: Res<Active>,
    data: Res<Data>,
    phase: Res<CurrentState<Phase>>,
    picked_factions: Query<&Faction, With<Player>>,
) {
    let picked_factions = picked_factions.iter().copied().collect::<HashSet<_>>();
    let remaining_factions = hashset![
        Faction::Atreides,
        Faction::BeneGesserit,
        Faction::Emperor,
        Faction::Fremen,
        Faction::Harkonnen,
        Faction::SpacingGuild,
    ];
    let remaining_factions = remaining_factions
        .difference(&picked_factions)
        .copied()
        .collect::<Vec<_>>();
    if !remaining_factions.is_empty() {
        let card_face = asset_server.get_handle("card.gltf#Mesh0/Primitive0");
        let card_back = asset_server.get_handle("card.gltf#Mesh0/Primitive1");
        let prediction_back_texture = asset_server.get_handle("predictions/prediction_back.png");
        for (i, faction) in remaining_factions.clone().into_iter().enumerate() {
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
                        .insert(Unique::new(active.entity))
                        .insert_bundle(PickableBundle::default());
                    parent
                        .spawn_bundle(PbrBundle {
                            mesh: card_back.clone(),
                            material: materials.add(StandardMaterial::from(prediction_back_texture.clone())),
                            ..default()
                        })
                        .insert(Unique::new(active.entity))
                        .insert_bundle(PickableBundle::default());
                });
        }
        commands.insert_resource(NextState(ChooseFactionsState::AwaitPick));
    } else {
        commands.insert_resource(NextState(ChooseFactionsState::PresentFactions));
        commands.insert_resource(NextState(phase.0.next()))
    }
}

fn await_pick(
    mut commands: Commands,
    mut picked_events: EventReader<PickedEvent<FactionPredictionCard>>,
    faction_cards: Query<(Entity, &Unique), With<FactionPredictionCard>>,
    data: Res<Data>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for PickedEvent {
        picker,
        picked: _,
        inner: FactionPredictionCard { faction },
    } in picked_events.iter()
    {
        commands.entity(*picker).insert(*faction);
        for (entity, _) in faction_cards.iter().filter(|(_, unique)| unique.entity == *picker) {
            // TODO: animate them away~
            commands.entity(entity).despawn_recursive();
        }

        let shield_face = asset_server.get_handle("shield.gltf#Mesh0/Primitive1");
        let shield_back = asset_server.get_handle("shield.gltf#Mesh0/Primitive2");

        let little_token = asset_server.get_handle("little_token.gltf#Mesh0/Primitive0");
        let big_token = asset_server.get_handle("big_token.gltf#Mesh0/Primitive0");
        let spice_token = asset_server.get_handle("spice_token.gltf#Mesh0/Primitive0");

        let shield_front_texture =
            asset_server.get_handle(format!("shields/{}_shield_front.png", faction.code()).as_str());
        let shield_back_texture =
            asset_server.get_handle(format!("shields/{}_shield_back.png", faction.code()).as_str());

        commands
            .spawn_bundle(SpatialBundle::from_transform(Transform::from_translation(vec3(
                0.0, 0.27, 1.34,
            ))))
            .insert(Unique::new(*picker))
            .insert(GameEntity)
            .insert(data.camera_nodes.shield)
            .with_children(|parent| {
                parent
                    .spawn_bundle(PbrBundle {
                        mesh: shield_face.clone(),
                        material: materials.add(StandardMaterial::from(shield_front_texture)),
                        ..Default::default()
                    })
                    .insert_bundle(PickableBundle::default());
                parent
                    .spawn_bundle(PbrBundle {
                        mesh: shield_back.clone(),
                        material: materials.add(StandardMaterial::from(shield_back_texture)),
                        ..Default::default()
                    })
                    .insert_bundle(PickableBundle::default());
            });

        for (i, (_, leader_data)) in data.leaders.iter().filter(|(_, l)| l.faction == *faction).enumerate() {
            let texture = asset_server.get_handle(format!("leaders/{}.png", leader_data.texture).as_str());
            commands
                .spawn_bundle(SpatialBundle::from_transform(Transform::from_translation(
                    data.token_nodes.leaders[i],
                )))
                .insert(Unique::new(*picker))
                .insert_bundle(PickableBundle::default())
                .insert(GameEntity)
                .insert_bundle(PbrBundle {
                    mesh: big_token.clone(),
                    material: materials.add(StandardMaterial::from(texture)),
                    ..Default::default()
                });
        }

        let troop_texture = asset_server.get_handle(format!("tokens/{}_troop.png", faction.code()).as_str());

        for i in 0..20 {
            commands
                .spawn_bundle(SpatialBundle::from_transform(Transform::from_translation(
                    data.token_nodes.fighters[0] + (i as f32 * 0.0036 * Vec3::Y),
                )))
                .insert(Unique::new(*picker))
                .insert_bundle(PickableBundle::default())
                .insert(GameEntity)
                .insert(Troop { value: 1 })
                .insert_bundle(PbrBundle {
                    mesh: little_token.clone(),
                    material: materials.add(StandardMaterial::from(troop_texture.clone())),
                    ..Default::default()
                });
        }

        let spice_1_texture = asset_server.get_handle("tokens/spice_1.png");
        let spice_1_material = materials.add(StandardMaterial::from(spice_1_texture));
        let spice_2_texture = asset_server.get_handle("tokens/spice_2.png");
        let spice_2_material = materials.add(StandardMaterial::from(spice_2_texture));
        let spice_5_texture = asset_server.get_handle("tokens/spice_5.png");
        let spice_5_material = materials.add(StandardMaterial::from(spice_5_texture));
        let spice_10_texture = asset_server.get_handle("tokens/spice_10.png");
        let spice_10_material = materials.add(StandardMaterial::from(spice_10_texture));

        let spice = data.factions.get(&faction).unwrap().starting_values.spice;

        let (tens, fives, twos, ones) = divide_spice(spice as i32);
        for (i, (value, s)) in (0..tens)
            .zip(std::iter::repeat((10, 0)))
            .chain((0..fives).zip(std::iter::repeat((5, 1))))
            .chain((0..twos).zip(std::iter::repeat((2, 2))))
            .chain((0..ones).zip(std::iter::repeat((1, 3))))
        {
            let material = match value {
                1 => spice_1_material.clone(),
                2 => spice_2_material.clone(),
                5 => spice_5_material.clone(),
                _ => spice_10_material.clone(),
            };
            commands
                .spawn_bundle(SpatialBundle::from_transform(Transform::from_translation(
                    data.token_nodes.spice[s] + (i as f32 * 0.0036 * Vec3::Y),
                )))
                .insert(Unique::new(*picker))
                .insert_bundle(PickableBundle::default())
                .insert(GameEntity)
                .insert(Spice { value })
                .insert_bundle(PbrBundle {
                    mesh: spice_token.clone(),
                    material,
                    ..Default::default()
                });
        }
        commands.insert_resource(AdvanceActive);
        commands.insert_resource(NextState(ChooseFactionsState::PresentFactions));
    }
}
