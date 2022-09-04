use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;
use rand::prelude::IteratorRandom;

use crate::{
    components::{
        Active, Faction, FactionPredictionCard, Player, Prediction, Spice, Troop, TurnPredictionCard, UniqueBundle,
    },
    resources::{Data, Info},
    util::divide_spice,
    ScreenEntity,
};

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(State::<SetupPhase>::get_driver());
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

pub fn init_factions(
    mut commands: Commands,
    data: Res<Data>,
    mut info: ResMut<Info>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_factions: Query<&Faction, With<Player>>,
) {
    println!("Enter: init_factions");
    let card_face = asset_server.get_handle("card.gltf#Mesh0/Primitive0");
    let card_back = asset_server.get_handle("card.gltf#Mesh0/Primitive1");

    let shield_face = asset_server.get_handle("shield.gltf#Mesh0/Primitive1");
    let shield_back = asset_server.get_handle("shield.gltf#Mesh0/Primitive2");

    let prediction_back_texture = asset_server.get_handle("treachery/treachery_back.png");
    let prediction_back_material = materials.add(StandardMaterial::from(prediction_back_texture));

    let little_token = asset_server.get_handle("little_token.gltf#Mesh0/Primitive0");
    let big_token = asset_server.get_handle("big_token.gltf#Mesh0/Primitive0");
    let spice_token = asset_server.get_handle("spice_token.gltf#Mesh0/Primitive0");

    for (i, &faction) in player_factions.iter().enumerate() {
        let faction_code = match faction {
            Faction::Atreides => "at",
            Faction::Harkonnen => "hk",
            Faction::Emperor => "em",
            Faction::SpacingGuild => "sg",
            Faction::Fremen => "fr",
            Faction::BeneGesserit => "bg",
        };

        let logo_texture: Handle<Image> = asset_server.get_handle(format!("tokens/{}_logo.png", faction_code).as_str());

        let shield_front_texture =
            asset_server.get_handle(format!("shields/{}_shield_front.png", faction_code).as_str());
        let shield_back_texture = asset_server.get_handle(format!("shields/{}_shield_back.png", faction_code).as_str());
        let shield_front_material = materials.add(StandardMaterial::from(shield_front_texture));
        let shield_back_material = materials.add(StandardMaterial::from(shield_back_texture));

        commands
            .spawn_bundle(UniqueBundle::new(faction))
            .insert_bundle(PickableBundle::default())
            .insert(ScreenEntity)
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
            asset_server.get_handle(format!("predictions/prediction_{}.png", faction_code).as_str());
        let prediction_front_material = materials.add(StandardMaterial::from(prediction_front_texture));

        commands
            .spawn_bundle(UniqueBundle::new(Faction::BeneGesserit))
            .insert_bundle(PickableBundle::default())
            .insert(ScreenEntity)
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

        for (i, (leader, leader_data)) in data.leaders.iter().filter(|(_, l)| l.faction == faction).enumerate() {
            let texture = asset_server.get_handle(format!("leaders/{}.png", leader_data.texture).as_str());
            let material = materials.add(StandardMaterial::from(texture));
            commands
                .spawn_bundle(UniqueBundle::new(faction))
                .insert_bundle(PickableBundle::default())
                .insert(ScreenEntity)
                .with_children(|parent| {
                    parent.spawn_bundle(PbrBundle {
                        mesh: big_token.clone(),
                        material,
                        ..Default::default()
                    });
                });
        }

        let troop_texture = asset_server.get_handle(format!("tokens/{}_troop.png", faction_code).as_str());
        let troop_material = materials.add(StandardMaterial::from(troop_texture));

        for i in 0..20 {
            commands
                .spawn_bundle(UniqueBundle::new(faction))
                .insert_bundle(PickableBundle::default())
                .insert(ScreenEntity)
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
                .spawn_bundle(UniqueBundle::new(faction))
                .insert_bundle(PickableBundle::default())
                .insert(ScreenEntity)
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
            .spawn_bundle(UniqueBundle::new(Faction::BeneGesserit))
            .insert_bundle(PickableBundle::default())
            .insert(ScreenEntity)
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

// TODO:
// - Trigger animate-in for faction cards
// - At end of animation, add Pickable component to faction cards
// - Create event when Pickable faction card is clicked
// - Add system to detect these events and add Faction component to player, pass to next player
// (maybe use Active component? Or store it in a resource since there will only ever be one active player?
// what about players that aren't "Active" but can still take actions? Active player system?)

pub fn pick_factions(
    mut commands: Commands,
    mut info: ResMut<Info>,
    data: Res<Data>,
    mut phase: ResMut<State<SetupPhase>>,
    to_pick: Query<(Entity, &Player), (With<Active>, Without<Faction>)>,
    picked: Query<&Faction, With<Player>>,
) {
    // TODO: pick using events
    let factions = vec![
        Faction::Atreides,
        Faction::BeneGesserit,
        Faction::Emperor,
        Faction::Fremen,
        Faction::Harkonnen,
        Faction::SpacingGuild,
    ]
    .into_iter()
    .filter(|f| !picked.iter().any(|p| p == f));

    let mut rng = rand::thread_rng();

    let faction = factions.choose(&mut rng).unwrap();
    let mut e = commands.spawn_bundle((faction, ScreenEntity));

    if faction == Faction::BeneGesserit {
        e.insert(Prediction {
            faction: None,
            turn: None,
        });
    }
}
