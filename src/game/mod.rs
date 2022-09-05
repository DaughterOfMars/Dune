mod phases;
mod systems;

use std::{collections::HashMap, hash::Hash};

use bevy::{
    math::vec3,
    prelude::*,
    render::camera::{Camera, OrthographicProjection},
};
use bevy_mod_picking::PickableBundle;
use iyes_loopless::prelude::{AppLooplessStateExt, IntoConditionalSystem};
use rand::prelude::SliceRandom;

use self::{
    phases::{
        setup::{SetupPhase, SetupPlugin},
        storm::StormPhase,
    },
    systems::*,
};
use crate::{
    components::{
        Deck, Disorganized, Faction, FactionPredictionCard, LocationSector, Player, Spice, Troop, TurnPredictionCard,
        Unique,
    },
    lerper::Lerp,
    resources::{Data, Info},
    util::{card_jitter, divide_spice},
    GameEntity, Screen,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(Phase::Setup(SetupPhase::ChooseFactions));

        app.add_plugin(SetupPlugin);

        app.add_enter_system(Screen::Game, init_factions);

        app.add_system(phase_text_system.run_in_state(Screen::Game));
        app.add_system(public_troop_system.run_in_state(Screen::Game));
        app.add_system(trigger_stack_troops.run_in_state(Screen::Game));
        app.add_system(shuffle_system.run_in_state(Screen::Game));
        app.add_system(render_unique.run_in_state(Screen::Game));

        app.add_exit_system(Screen::Game, reset_system);
    }
}

#[derive(Component)]
pub struct PhaseText;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Phase {
    Setup(SetupPhase),
    Storm(StormPhase),
    SpiceBlow,
    Nexus,
    Bidding,
    Revival,
    Movement,
    Battle,
    Collection,
    Control,
    EndGame,
}

impl Phase {
    pub fn next(&self) -> Self {
        match self {
            Phase::Setup(subphase) => match subphase {
                SetupPhase::ChooseFactions => Phase::Setup(SetupPhase::Prediction),
                SetupPhase::Prediction => Phase::Setup(SetupPhase::AtStart),
                SetupPhase::AtStart => Phase::Setup(SetupPhase::DealTraitors),
                SetupPhase::DealTraitors => Phase::Setup(SetupPhase::PickTraitors),
                SetupPhase::PickTraitors => Phase::Setup(SetupPhase::DealTreachery),
                SetupPhase::DealTreachery => Phase::Storm(StormPhase::Reveal),
            },
            Phase::Storm(subphase) => match subphase {
                StormPhase::Reveal => Phase::Storm(StormPhase::WeatherControl),
                StormPhase::WeatherControl => Phase::Storm(StormPhase::FamilyAtomics),
                StormPhase::FamilyAtomics => Phase::Storm(StormPhase::MoveStorm),
                StormPhase::MoveStorm => Phase::SpiceBlow,
            },
            Phase::SpiceBlow => Phase::Nexus,
            Phase::Nexus => Phase::Bidding,
            Phase::Bidding => Phase::Revival,
            Phase::Revival => Phase::Movement,
            Phase::Movement => Phase::Battle,
            Phase::Battle => Phase::Collection,
            Phase::Collection => Phase::Control,
            Phase::Control => Phase::Storm(StormPhase::Reveal),
            Phase::EndGame => Phase::EndGame,
        }
    }
}

fn reset_system() {
    todo!()
}

#[derive(Component)]
pub struct Shuffling(pub usize);

pub fn init_shuffle_decks(mut commands: Commands, decks: Query<Entity, With<Deck>>) {
    for deck in decks.iter() {
        commands.entity(deck).insert(Shuffling(5));
    }
}

pub fn shuffle_system(
    mut commands: Commands,
    mut decks: Query<(Entity, &mut Deck, &Children, &mut Shuffling)>,
    lerps: Query<&Lerp>,
) {
    let mut rng = rand::thread_rng();
    for (e, mut deck, children, mut shuffling) in decks.iter_mut() {
        if children.iter().any(|c| lerps.get(*c).is_ok()) {
            shuffling.0 -= 1;
            if shuffling.0 == 0 {
                commands.entity(e).remove::<Shuffling>();
            }
            continue;
        }
        let mut cards = children.iter().enumerate().collect::<Vec<_>>();
        cards.shuffle(&mut rng);
        deck.0 = cards.iter().map(|(_, e)| **e).collect();
        for (i, card) in cards {
            let transform = Transform::from_translation(Vec3::Y * 0.001 * (i as f32)) * card_jitter();
            commands.entity(*card).insert(Lerp::world_to(transform, 0.2, 0.0));
        }
    }
}

pub fn init_factions(
    mut commands: Commands,
    data: Res<Data>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    free_cams: Query<Entity, (With<Camera>, Without<Player>)>,
) {
    info!("Enter: init_factions");
    let card_face = asset_server.get_handle("card.gltf#Mesh0/Primitive0");
    let card_back = asset_server.get_handle("card.gltf#Mesh0/Primitive1");

    let shield_face = asset_server.get_handle("shield.gltf#Mesh0/Primitive1");
    let shield_back = asset_server.get_handle("shield.gltf#Mesh0/Primitive2");

    let prediction_back_texture = asset_server.get_handle("treachery/treachery_back.png");
    let prediction_back_material = materials.add(StandardMaterial::from(prediction_back_texture));

    let little_token = asset_server.get_handle("little_token.gltf#Mesh0/Primitive0");
    let big_token = asset_server.get_handle("big_token.gltf#Mesh0/Primitive0");
    let spice_token = asset_server.get_handle("spice_token.gltf#Mesh0/Primitive0");

    let factions = vec![
        Faction::Atreides,
        Faction::Harkonnen,
        Faction::Emperor,
        Faction::SpacingGuild,
        Faction::Fremen,
        Faction::BeneGesserit,
    ];

    for (i, &faction) in factions.iter().enumerate() {
        let faction_data = data.factions.get(&faction).unwrap();
        commands
            .entity(free_cams.iter().next().unwrap())
            .insert_bundle((Player::new(faction_data.name.clone(), (i + 1) as _), faction));

        // let logo_texture: Handle<Image> = asset_server.get_handle(format!("tokens/{}_logo.png",
        // faction_code).as_str());

        let shield_front_texture =
            asset_server.get_handle(format!("shields/{}_shield_front.png", faction.code()).as_str());
        let shield_back_texture =
            asset_server.get_handle(format!("shields/{}_shield_back.png", faction.code()).as_str());
        let shield_front_material = materials.add(StandardMaterial::from(shield_front_texture));
        let shield_back_material = materials.add(StandardMaterial::from(shield_back_texture));

        commands
            .spawn_bundle(SpatialBundle::from_transform(Transform::from_translation(vec3(
                0.0, 0.27, 1.34,
            ))))
            .insert(Unique::new(faction))
            .insert_bundle(PickableBundle::default())
            .insert(GameEntity)
            .insert(data.camera_nodes.shield)
            .with_children(|parent| {
                parent.spawn_bundle(PbrBundle {
                    mesh: shield_face.clone(),
                    material: shield_front_material,
                    ..Default::default()
                });
                parent.spawn_bundle(PbrBundle {
                    mesh: shield_back.clone(),
                    material: shield_back_material,
                    ..Default::default()
                });
            });

        let prediction_front_texture =
            asset_server.get_handle(format!("predictions/prediction_{}.png", faction.code()).as_str());
        let prediction_front_material = materials.add(StandardMaterial::from(prediction_front_texture));

        commands
            .spawn_bundle(SpatialBundle::default())
            .insert(Unique::new(Faction::BeneGesserit))
            .insert_bundle(PickableBundle::default())
            .insert(GameEntity)
            .insert(FactionPredictionCard { faction })
            .with_children(|parent| {
                parent.spawn_bundle(PbrBundle {
                    mesh: card_face.clone(),
                    material: prediction_front_material,
                    ..Default::default()
                });
                parent.spawn_bundle(PbrBundle {
                    mesh: card_back.clone(),
                    material: prediction_back_material.clone(),
                    ..Default::default()
                });
            });

        for (i, (_, leader_data)) in data.leaders.iter().filter(|(_, l)| l.faction == faction).enumerate() {
            let texture = asset_server.get_handle(format!("leaders/{}.png", leader_data.texture).as_str());
            let material = materials.add(StandardMaterial::from(texture));
            commands
                .spawn_bundle(SpatialBundle::from_transform(Transform::from_translation(
                    data.token_nodes.leaders[i],
                )))
                .insert(Unique::new(faction))
                .insert_bundle(PickableBundle::default())
                .insert(GameEntity)
                .with_children(|parent| {
                    parent.spawn_bundle(PbrBundle {
                        mesh: big_token.clone(),
                        material,
                        ..Default::default()
                    });
                });
        }

        let troop_texture = asset_server.get_handle(format!("tokens/{}_troop.png", faction.code()).as_str());
        let troop_material = materials.add(StandardMaterial::from(troop_texture));

        for i in 0..20 {
            commands
                .spawn_bundle(SpatialBundle::from_transform(Transform::from_translation(
                    data.token_nodes.fighters[0] + (i as f32 * 0.0036 * Vec3::Y),
                )))
                .insert(Unique::new(faction))
                .insert_bundle(PickableBundle::default())
                .insert(GameEntity)
                .insert(Troop {
                    value: 1,
                    location: None,
                })
                .with_children(|parent| {
                    parent.spawn_bundle(PbrBundle {
                        mesh: little_token.clone(),
                        material: troop_material.clone(),
                        ..Default::default()
                    });
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
                .insert(Unique::new(faction))
                .insert_bundle(PickableBundle::default())
                .insert(GameEntity)
                .insert(Spice { value })
                .with_children(|parent| {
                    parent.spawn_bundle(PbrBundle {
                        mesh: spice_token.clone(),
                        material,
                        ..Default::default()
                    });
                });
        }
    }

    (1..=15).for_each(|turn| {
        let prediction_front_texture =
            asset_server.get_handle(format!("predictions/prediction_t{}.png", turn).as_str());
        let prediction_front_material = materials.add(StandardMaterial::from(prediction_front_texture));
        commands
            .spawn_bundle(SpatialBundle::default())
            .insert(Unique::new(Faction::BeneGesserit))
            .insert_bundle(PickableBundle::default())
            .insert(GameEntity)
            .insert(TurnPredictionCard { turn })
            .with_children(|parent| {
                parent.spawn_bundle(PbrBundle {
                    mesh: card_face.clone(),
                    material: prediction_front_material,
                    ..Default::default()
                });
                parent.spawn_bundle(PbrBundle {
                    mesh: card_back.clone(),
                    material: prediction_back_material.clone(),
                    ..Default::default()
                });
            });
    });
}
