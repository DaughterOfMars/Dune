use bevy::{
    math::Vec4Swizzles,
    prelude::*,
    render::camera::{Camera, PerspectiveProjection},
};

mod data;
use cgmath::{MetricSpace, Point3, Vector3};
use data::*;

use std::{collections::HashMap, fs::File};

use rand::{seq::SliceRandom, Rng};

use std::f32::consts::PI;

use collision::{prelude::*, primitive::Primitive3, Aabb3, Ray};

fn main() {
    App::build()
        .add_resource(Msaa { samples: 4 })
        .add_resource(ClearColor(Color::BLACK))
        .add_resource(Data::init())
        .add_resource(Info::new())
        .add_resource(Resources::new())
        .add_resource(State {
            phase: Phase::Setup {
                subphase: SetupSubPhase::ChooseFactions,
            },
        })
        .add_resource(ActionStack(Vec::new()))
        .add_plugins(DefaultPlugins)
        .add_startup_system(init.system())
        .add_system(handle_actions.system())
        .add_system(input.system())
        .add_system(handle_phase.system())
        .add_system(propagate_visibility.system())
        //.add_system(handle_phase.system())
        .run();
}

#[derive(Copy, Clone)]
enum Phase {
    Setup { subphase: SetupSubPhase },
    Storm { subphase: StormSubPhase },
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
    fn next(&self) -> Self {
        match self {
            Phase::Setup { subphase } => match subphase {
                SetupSubPhase::ChooseFactions => Phase::Setup {
                    subphase: SetupSubPhase::Prediction,
                },
                SetupSubPhase::Prediction => Phase::Setup {
                    subphase: SetupSubPhase::AtStart,
                },
                SetupSubPhase::AtStart => Phase::Setup {
                    subphase: SetupSubPhase::DealTraitors,
                },
                SetupSubPhase::DealTraitors => Phase::Setup {
                    subphase: SetupSubPhase::PickTraitors,
                },
                SetupSubPhase::PickTraitors => Phase::Setup {
                    subphase: SetupSubPhase::DealTreachery,
                },
                SetupSubPhase::DealTreachery => Phase::Storm {
                    subphase: StormSubPhase::Reveal,
                },
            },
            Phase::Storm { subphase } => match subphase {
                StormSubPhase::Reveal => Phase::Storm {
                    subphase: StormSubPhase::WeatherControl,
                },
                StormSubPhase::WeatherControl => Phase::Storm {
                    subphase: StormSubPhase::FamilyAtomics,
                },
                StormSubPhase::FamilyAtomics => Phase::Storm {
                    subphase: StormSubPhase::MoveStorm,
                },
                StormSubPhase::MoveStorm => Phase::SpiceBlow,
            },
            Phase::SpiceBlow => Phase::Nexus,
            Phase::Nexus => Phase::Bidding,
            Phase::Bidding => Phase::Revival,
            Phase::Revival => Phase::Movement,
            Phase::Movement => Phase::Battle,
            Phase::Battle => Phase::Collection,
            Phase::Collection => Phase::Control,
            Phase::Control => Phase::Storm {
                subphase: StormSubPhase::Reveal,
            },
            Phase::EndGame => Phase::EndGame,
        }
    }

    fn advance(&mut self) {
        *self = self.next();
    }
}

#[derive(Copy, Clone)]
enum SetupSubPhase {
    ChooseFactions,
    Prediction,
    AtStart,
    DealTraitors,
    PickTraitors,
    DealTreachery,
}

#[derive(Copy, Clone)]
enum StormSubPhase {
    Reveal,
    WeatherControl,
    FamilyAtomics,
    MoveStorm,
}

#[derive(Copy, Clone)]
struct Spice {
    value: i32,
}

#[derive(Copy, Clone)]
struct Troop {
    value: i32,
    location: Option<Entity>,
}

#[derive(Default)]
struct Storm {
    sector: i32,
}

// Something that is uniquely visible to one faction
struct Unique {
    faction: Faction,
}

struct Collider {
    aabb: Aabb3<f32>,
    primitive: Option<Primitive3<f32>>,
}

#[derive(Clone)]
struct ClickAction {
    action: Action,
}

struct Prediction {
    faction: Option<Faction>,
    turn: Option<i32>,
}

#[derive(Clone)]
enum Action {
    // Allows a player to choose between multiple actions
    Choice {
        player: Entity,
        options: Vec<Action>,
    },
    // Allows multiple actions to occur at once
    Simultaneous {
        actions: Vec<Action>,
    },
    MakePrediction {
        factions_shown: bool,
        faction_picked: bool,
        turns_shown: bool,
        turn_picked: bool,
        animation_time: f32,
        current_time: f32,
        src: Vec2,
        faction_dests: Vec<Vec2>,
        turn_dests: Vec<Vec2>,
    },
    PlaceFreeTroops {
        player: Entity,
        num: i32,
        locations: Option<Vec<String>>,
    },
    PlaceTroops {
        player: Entity,
        num: i32,
        locations: Option<Vec<String>>,
    },
    PickTraitors,
    PlayPrompt {
        treachery_card: String,
    },
    CameraMotion {
        src: Option<Transform>,
        dest: CameraNode,
        remaining_time: f32,
        total_time: f32,
    },
    ButtonPress,
}

impl Action {
    fn move_camera(dest: CameraNode, time: f32) -> Self {
        Action::CameraMotion {
            src: None,
            dest,
            remaining_time: time,
            total_time: time,
        }
    }

    fn make_prediction(time: f32) -> Self {
        Action::MakePrediction {
            factions_shown: false,
            faction_picked: false,
            turns_shown: false,
            turn_picked: false,
            animation_time: time,
            current_time: 0.0,
            src: (1.5, -1.5).into(),
            faction_dests: vec![
                (-0.5, 0.5).into(),
                (0.0, 0.5).into(),
                (0.5, 0.5).into(),
                (-0.5, -0.5).into(),
                (0.0, -0.5).into(),
                (0.5, -0.5).into(),
            ],
            turn_dests: vec![
                (-0.8, 0.6).into(),
                (-0.4, 0.6).into(),
                (0.0, 0.6).into(),
                (0.4, 0.6).into(),
                (0.8, 0.6).into(),
                (-0.8, 0.0).into(),
                (-0.4, 0.0).into(),
                (0.0, 0.0).into(),
                (0.4, 0.0).into(),
                (0.8, 0.0).into(),
                (-0.8, -0.6).into(),
                (-0.4, -0.6).into(),
                (0.0, -0.6).into(),
                (0.4, -0.6).into(),
                (0.8, -0.6).into(),
            ],
        }
    }
}

struct Data {
    leaders: Vec<Leader>,
    locations: Vec<Location>,
    treachery_cards: Vec<TreacheryCard>,
    spice_cards: Vec<SpiceCard>,
    camera_nodes: CameraNodes,
}

impl Data {
    fn init() -> Self {
        let locations = ron::de::from_reader(File::open("src/locations.ron").unwrap()).unwrap();
        let leaders = ron::de::from_reader(File::open("src/leaders.ron").unwrap()).unwrap();
        let treachery_cards =
            ron::de::from_reader(File::open("src/treachery.ron").unwrap()).unwrap();
        let spice_cards = ron::de::from_reader(File::open("src/spice.ron").unwrap()).unwrap();
        let camera_nodes =
            ron::de::from_reader(File::open("src/camera_nodes.ron").unwrap()).unwrap();
        Data {
            locations,
            leaders,
            treachery_cards,
            spice_cards,
            camera_nodes,
        }
    }
}

struct State {
    phase: Phase,
}

struct ActionStack(Vec<Action>);

impl ActionStack {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn push(&mut self, action: Action) {
        self.0.push(action);
    }

    fn peek(&self) -> Option<&Action> {
        self.0.last()
    }

    fn peek_mut(&mut self) -> Option<&mut Action> {
        self.0.last_mut()
    }

    fn pop(&mut self) -> Option<Action> {
        self.0.pop()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn extend<T: IntoIterator<Item = Action>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

struct Info {
    turn: i32,
    factions_in_play: Vec<Faction>,
    active_player: usize,
    play_order: Vec<Entity>,
}

impl Info {
    fn new() -> Self {
        Self {
            turn: 0,
            factions_in_play: Vec::new(),
            active_player: 0,
            play_order: Vec::new(),
        }
    }
}

struct Resources {
    spice_bank: Vec<Spice>,
    treachery_deck: Vec<Entity>,
    treachery_discard: Vec<Entity>,
    traitor_deck: Vec<Entity>,
    traitor_discard: Vec<Entity>,
    spice_deck: Vec<Entity>,
    spice_discard: Vec<Entity>,
    storm_deck: Vec<Entity>,
}

impl Resources {
    fn new() -> Self {
        Self {
            spice_bank: Vec::new(),
            treachery_deck: Vec::new(),
            treachery_discard: Vec::new(),
            traitor_deck: Vec::new(),
            traitor_discard: Vec::new(),
            spice_deck: Vec::new(),
            spice_discard: Vec::new(),
            storm_deck: Vec::new(),
        }
    }
}

struct Player {
    faction: Faction,
    traitor_cards: Vec<Entity>,
    treachery_cards: Vec<Entity>,
    spice: Vec<Spice>,
    total_spice: i32,
    reserve_troops: i32,
    leaders: Vec<Leader>,
}

impl Player {
    fn new(faction: Faction, all_leaders: &Vec<Leader>) -> Self {
        let (troops, _, spice) = faction.initial_values();
        Player {
            faction,
            traitor_cards: Vec::new(),
            treachery_cards: Vec::new(),
            spice: Vec::new(),
            total_spice: spice,
            reserve_troops: 20 - troops,
            leaders: all_leaders
                .iter()
                .filter(|&leader| leader.faction == faction)
                .cloned()
                .collect::<Vec<Leader>>(),
        }
    }
}

fn init(
    commands: &mut Commands,
    data: Res<Data>,
    mut info: ResMut<Info>,
    mut resources: ResMut<Resources>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    asset_server.load_folder(".").unwrap();
    // Board
    commands
        .spawn((
            Collider {
                aabb: collision::Aabb3::new((1.0, -0.007, 1.1).into(), (-1.0, 0.007, -1.1).into()),
                primitive: None,
            },
            ClickAction {
                action: Action::move_camera(data.camera_nodes.board, 1.5),
            },
            Transform::default(),
            GlobalTransform::default(),
        ))
        .with_children(|parent| {
            parent.spawn_scene(asset_server.get_handle("board.gltf"));
        });

    //Camera
    commands.spawn(Camera3dBundle {
        perspective_projection: PerspectiveProjection {
            near: 0.01,
            far: 100.0,
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 2.5, 2.0))
            .looking_at(Vec3::zero(), Vec3::unit_y())
            * Transform::from_translation(Vec3::new(0.0, -0.4, 0.0)),
        ..Default::default()
    });

    // Light
    commands
        .spawn(LightBundle {
            transform: Transform::from_translation(Vec3::new(10.0, 10.0, 10.0)),
            ..Default::default()
        })
        .spawn((Storm::default(),));

    let mut rng = rand::thread_rng();

    info.factions_in_play = vec![
        Faction::Atreides,
        Faction::BeneGesserit,
        Faction::Emperor,
        Faction::Fremen,
        Faction::Harkonnen,
        Faction::SpacingGuild,
    ];

    let shield_face = asset_server.get_handle("shield.gltf#Mesh0/Primitive1");
    let shield_back = asset_server.get_handle("shield.gltf#Mesh0/Primitive2");

    let card_face = asset_server.get_handle("card.gltf#Mesh0/Primitive0");
    let card_back = asset_server.get_handle("card.gltf#Mesh0/Primitive1");

    let prediction_back_texture = asset_server.get_handle("treachery/treachery_back.png");
    let prediction_back_material = materials.add(StandardMaterial {
        albedo_texture: Some(prediction_back_texture),
        ..Default::default()
    });

    info.play_order = info
        .factions_in_play
        .iter()
        .map(|&faction| {
            let faction_code = match faction {
                Faction::Atreides => "at",
                Faction::Harkonnen => "hk",
                Faction::Emperor => "em",
                Faction::SpacingGuild => "sg",
                Faction::Fremen => "fr",
                Faction::BeneGesserit => "bg",
            };
            let shield_front_texture = asset_server
                .get_handle(format!("shields/{}_shield_front.png", faction_code).as_str());
            let shield_back_texture = asset_server
                .get_handle(format!("shields/{}_shield_back.png", faction_code).as_str());
            let shield_front_material = materials.add(StandardMaterial {
                albedo_texture: Some(shield_front_texture),
                ..Default::default()
            });
            let shield_back_material = materials.add(StandardMaterial {
                albedo_texture: Some(shield_back_texture),
                ..Default::default()
            });
            commands
                .spawn((
                    Unique { faction },
                    Transform::from_translation(Vec3::new(0.0, 0.27, 1.34)),
                    GlobalTransform::default(),
                    Visible {
                        is_visible: false,
                        ..Default::default()
                    },
                    Collider {
                        aabb: collision::Aabb3::new(
                            (-0.525, 0.0, 1.542).into(),
                            (0.525, 0.57, 1.421).into(),
                        ),
                        primitive: None,
                    },
                    ClickAction {
                        action: Action::move_camera(data.camera_nodes.shield, 1.5),
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(PbrBundle {
                        mesh: shield_face.clone(),
                        material: shield_front_material,
                        ..Default::default()
                    });
                    parent.spawn(PbrBundle {
                        mesh: shield_back.clone(),
                        material: shield_back_material,
                        ..Default::default()
                    });
                });
            let prediction_front_texture = asset_server
                .get_handle(format!("predictions/prediction_{}.png", faction_code).as_str());
            let prediction_front_material = materials.add(StandardMaterial {
                albedo_texture: Some(prediction_front_texture),
                ..Default::default()
            });
            commands
                .spawn((
                    Unique {
                        faction: Faction::BeneGesserit,
                    },
                    FactionPredictionCard { faction },
                    Transform::default(),
                    GlobalTransform::default(),
                    Visible {
                        is_visible: false,
                        ..Default::default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(PbrBundle {
                        mesh: card_face.clone(),
                        material: prediction_front_material,
                        ..Default::default()
                    });
                    parent.spawn(PbrBundle {
                        mesh: card_back.clone(),
                        material: prediction_back_material.clone(),
                        ..Default::default()
                    });
                });
            commands.spawn((Player::new(faction, &data.leaders),));

            if faction == Faction::BeneGesserit {
                commands.with(Prediction {
                    faction: None,
                    turn: None,
                });
            }

            commands.current_entity().unwrap()
        })
        .collect();

    info.play_order.shuffle(&mut rng);

    (1..=15).for_each(|turn| {
        let prediction_front_texture =
            asset_server.get_handle(format!("predictions/prediction_t{}.png", turn).as_str());
        let prediction_front_material = materials.add(StandardMaterial {
            albedo_texture: Some(prediction_front_texture),
            ..Default::default()
        });
        commands
            .spawn((
                Unique {
                    faction: Faction::BeneGesserit,
                },
                TurnPredictionCard { turn },
                Transform::default(),
                GlobalTransform::default(),
                Visible {
                    is_visible: false,
                    ..Default::default()
                },
            ))
            .with_children(|parent| {
                parent.spawn(PbrBundle {
                    mesh: card_face.clone(),
                    material: prediction_front_material,
                    ..Default::default()
                });
                parent.spawn(PbrBundle {
                    mesh: card_back.clone(),
                    material: prediction_back_material.clone(),
                    ..Default::default()
                });
            });
    });

    resources.spice_bank.extend(
        (0..50)
            .map(|_| Spice { value: 1 })
            .chain((0..50).map(|_| Spice { value: 2 }))
            .chain((0..20).map(|_| Spice { value: 5 }))
            .chain((0..10).map(|_| Spice { value: 10 })),
    );

    commands.spawn(());

    let treachery_back_texture = asset_server.get_handle("treachery/treachery_back.png");
    let treachery_back_material = materials.add(StandardMaterial {
        albedo_texture: Some(treachery_back_texture),
        ..Default::default()
    });

    resources.treachery_deck = data
        .treachery_cards
        .iter()
        .enumerate()
        .map(|(i, card)| {
            let treachery_front_texture = asset_server
                .get_handle(format!("treachery/treachery_{}.png", card.texture.as_str()).as_str());
            let treachery_front_material = materials.add(StandardMaterial {
                albedo_texture: Some(treachery_front_texture),
                ..Default::default()
            });

            commands
                .spawn((
                    card.clone(),
                    Transform::from_translation(Vec3::new(
                        1.23,
                        0.0049 + (i as f32 * 0.001),
                        -0.87,
                    )) * Transform::from_rotation(Quat::from_rotation_z(PI)),
                    GlobalTransform::default(),
                ))
                .with_children(|parent| {
                    parent.spawn(PbrBundle {
                        mesh: card_face.clone(),
                        material: treachery_front_material,
                        ..Default::default()
                    });
                    parent.spawn(PbrBundle {
                        mesh: card_back.clone(),
                        material: treachery_back_material.clone(),
                        ..Default::default()
                    });
                })
                .current_entity()
                .unwrap()
        })
        .collect();
    resources.treachery_deck.shuffle(&mut rng);

    let traitor_back_texture = asset_server.get_handle("traitor/traitor_back.png");
    let traitor_back_material = materials.add(StandardMaterial {
        albedo_texture: Some(traitor_back_texture),
        ..Default::default()
    });

    resources.traitor_deck = data
        .leaders
        .iter()
        .enumerate()
        .map(|(i, card)| {
            let traitor_front_texture = asset_server
                .get_handle(format!("traitor/traitor_{}.png", card.texture.as_str()).as_str());
            let traitor_front_material = materials.add(StandardMaterial {
                albedo_texture: Some(traitor_front_texture),
                ..Default::default()
            });

            commands
                .spawn((
                    card.clone(),
                    Transform::from_translation(Vec3::new(1.23, 0.0049 + (i as f32 * 0.001), -0.3))
                        * Transform::from_rotation(Quat::from_rotation_z(PI)),
                    GlobalTransform::default(),
                ))
                .with_children(|parent| {
                    parent.spawn(PbrBundle {
                        mesh: card_face.clone(),
                        material: traitor_front_material,
                        ..Default::default()
                    });
                    parent.spawn(PbrBundle {
                        mesh: card_back.clone(),
                        material: traitor_back_material.clone(),
                        ..Default::default()
                    });
                })
                .current_entity()
                .unwrap()
        })
        .collect();
    resources.traitor_deck.shuffle(&mut rng);

    let spice_back_texture = asset_server.get_handle("spice/spice_back.png");
    let spice_back_material = materials.add(StandardMaterial {
        albedo_texture: Some(spice_back_texture),
        ..Default::default()
    });

    resources.spice_deck = data
        .spice_cards
        .iter()
        .enumerate()
        .map(|(i, card)| {
            let spice_front_texture = asset_server
                .get_handle(format!("spice/spice_{}.png", card.texture.as_str()).as_str());
            let spice_front_material = materials.add(StandardMaterial {
                albedo_texture: Some(spice_front_texture),
                ..Default::default()
            });

            commands
                .spawn((
                    card.clone(),
                    Transform::from_translation(Vec3::new(1.23, 0.0049 + (i as f32 * 0.001), 0.3))
                        * Transform::from_rotation(Quat::from_rotation_z(PI)),
                    GlobalTransform::default(),
                ))
                .with_children(|parent| {
                    parent.spawn(PbrBundle {
                        mesh: card_face.clone(),
                        material: spice_front_material,
                        ..Default::default()
                    });
                    parent.spawn(PbrBundle {
                        mesh: card_back.clone(),
                        material: spice_back_material.clone(),
                        ..Default::default()
                    });
                })
                .current_entity()
                .unwrap()
        })
        .collect();
    resources.spice_deck.shuffle(&mut rng);

    let storm_back_texture = asset_server.get_handle("storm/storm_back.png");
    let storm_back_material = materials.add(StandardMaterial {
        albedo_texture: Some(storm_back_texture),
        ..Default::default()
    });

    resources.storm_deck = (1..7)
        .map(|val| {
            let storm_front_texture =
                asset_server.get_handle(format!("storm/storm_{}.png", val).as_str());
            let storm_front_material = materials.add(StandardMaterial {
                albedo_texture: Some(storm_front_texture),
                ..Default::default()
            });

            commands
                .spawn((
                    StormCard { val },
                    Transform::from_translation(Vec3::new(
                        1.23,
                        0.0049 + (val as f32 * 0.001),
                        0.87,
                    )) * Transform::from_rotation(Quat::from_rotation_z(PI)),
                    GlobalTransform::default(),
                ))
                .with_children(|parent| {
                    parent.spawn(PbrBundle {
                        mesh: card_face.clone(),
                        material: storm_front_material,
                        ..Default::default()
                    });
                    parent.spawn(PbrBundle {
                        mesh: card_back.clone(),
                        material: storm_back_material.clone(),
                        ..Default::default()
                    });
                })
                .current_entity()
                .unwrap()
        })
        .collect();
    resources.storm_deck.shuffle(&mut rng);

    commands.spawn((
        Collider {
            aabb: collision::Aabb3::new((1.1, -0.009, 0.47).into(), (1.35, 0.05, 0.11).into()),
            primitive: None,
        },
        ClickAction {
            action: Action::move_camera(data.camera_nodes.spice, 1.5),
        },
    ));

    commands.spawn((
        Collider {
            aabb: collision::Aabb3::new((1.1, -0.009, 1.05).into(), (1.35, 0.05, 0.69).into()),
            primitive: None,
        },
        ClickAction {
            action: Action::move_camera(data.camera_nodes.storm, 1.5),
        },
    ));
}

fn input(
    mut stack: ResMut<ActionStack>,
    data: Res<Data>,
    windows: Res<Windows>,
    mouse_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    cameras: Query<(&Transform, &Camera)>,
    colliders: Query<(&Collider, &ClickAction)>,
    mut predictions: Query<&mut Prediction>,
    prediction_cards: QuerySet<(Query<&FactionPredictionCard>, Query<&TurnPredictionCard>)>,
) {
    match stack.peek_mut() {
        Some(Action::MakePrediction {
            factions_shown,
            faction_picked,
            turns_shown,
            turn_picked,
            animation_time: _,
            current_time: _,
            src: _,
            faction_dests,
            turn_dests,
        }) => {
            if !*faction_picked || !*turn_picked {
                if mouse_input.just_pressed(MouseButton::Left) {
                    if let Some((_, camera)) = cameras.iter().next() {
                        if let Some(window) = windows.get_primary() {
                            if let Some(pos) = window.cursor_position() {
                                let ss_pos = Vec2::new(
                                    2.0 * (pos.x / window.physical_width() as f32) - 1.0,
                                    2.0 * (pos.y / window.physical_height() as f32) - 1.0,
                                );
                                let p = camera.projection_matrix.inverse()
                                    * ss_pos.extend(0.0).extend(1.0);
                                let p = p.xyz() / p.w;
                                let card_extents = Vec2::new(0.125, 0.18);
                                if *factions_shown && !*faction_picked {
                                    for (i, &card_pos) in faction_dests.iter().enumerate() {
                                        if let Some(FactionPredictionCard { faction }) =
                                            prediction_cards.q0().iter().nth(i)
                                        {
                                            let card_pos = camera.projection_matrix.inverse()
                                                * card_pos.extend(0.0).extend(1.0);
                                            let card_pos = card_pos.xyz() / card_pos.w;
                                            if p.x >= card_pos.x - card_extents.x
                                                && p.x <= card_pos.x + card_extents.x
                                                && p.y >= card_pos.y - card_extents.y
                                                && p.y <= card_pos.y + card_extents.y
                                            {
                                                predictions.iter_mut().for_each(
                                                    |mut prediction| {
                                                        prediction.faction = Some(*faction);
                                                    },
                                                );
                                                *faction_picked = true;
                                                break;
                                            }
                                        }
                                    }
                                } else if *turns_shown && !*turn_picked {
                                    for (i, &card_pos) in turn_dests.iter().enumerate() {
                                        if let Some(TurnPredictionCard { turn }) =
                                            prediction_cards.q1().iter().nth(i)
                                        {
                                            let card_pos = camera.projection_matrix.inverse()
                                                * card_pos.extend(0.0).extend(1.0);
                                            let card_pos = card_pos.xyz() / card_pos.w;
                                            if p.x >= card_pos.x - card_extents.x
                                                && p.x <= card_pos.x + card_extents.x
                                                && p.y >= card_pos.y - card_extents.y
                                                && p.y <= card_pos.y + card_extents.y
                                            {
                                                predictions.iter_mut().for_each(
                                                    |mut prediction| {
                                                        prediction.turn = Some(*turn);
                                                    },
                                                );
                                                *turn_picked = true;
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {
            if mouse_input.just_pressed(MouseButton::Left) {
                if let Some((&transform, camera)) = cameras.iter().next() {
                    if let Some(window) = windows.get_primary() {
                        if let Some(pos) = window.cursor_position() {
                            let ss_pos = Vec2::new(
                                2.0 * (pos.x / window.physical_width() as f32) - 1.0,
                                2.0 * (pos.y / window.physical_height() as f32) - 1.0,
                            );
                            let p0 = screen_to_world(
                                ss_pos.extend(0.0),
                                transform,
                                camera.projection_matrix,
                            );
                            let p1 = screen_to_world(
                                ss_pos.extend(1.0),
                                transform,
                                camera.projection_matrix,
                            );
                            let dir = (p1 - p0).normalize();
                            let ray = Ray::<f32, Point3<f32>, Vector3<f32>>::new(
                                (p0.x, p0.y, p0.z).into(),
                                (dir.x, dir.y, dir.z).into(),
                            );
                            let (mut closest_intersection, mut closest_action) = (None, None);
                            for (collider, action) in colliders.iter() {
                                if let Some(intersection) = ray.intersection(&collider.aabb) {
                                    if closest_intersection.is_none() {
                                        closest_intersection = Some(intersection);
                                        closest_action = Some(action);
                                    } else {
                                        if ray.origin.distance(closest_intersection.unwrap())
                                            > ray.origin.distance(intersection)
                                        {
                                            closest_intersection = Some(intersection);
                                            closest_action = Some(action);
                                        }
                                    }
                                }
                            }
                            if let Some(ClickAction { action }) = closest_action {
                                stack.push(action.clone());
                            }
                        }
                    }
                }
            } else if keyboard_input.just_pressed(KeyCode::Escape) {
                stack.push(Action::move_camera(data.camera_nodes.main, 1.5));
            }
        }
    }
}

fn handle_phase(
    mut stack: ResMut<ActionStack>,
    mut state: ResMut<State>,
    mut info: ResMut<Info>,
    mut resources: ResMut<Resources>,
    mut player_query: Query<(Entity, &mut Player)>,
    mut storm_query: Query<&mut Storm>,
    storm_cards: Query<&StormCard>,
    mut unique_query: Query<(&mut Visible, &Unique)>,
) {
    // We need to resolve any pending actions first
    if stack.is_empty() {
        match state.phase {
            Phase::Setup { ref mut subphase } => {
                match subphase {
                    SetupSubPhase::ChooseFactions => {
                        // skip for now
                        set_view_to_active_player(&info, &mut player_query, &mut unique_query);
                        state.phase.advance();
                    }
                    SetupSubPhase::Prediction => {
                        for (_, player) in player_query.iter_mut() {
                            if player.faction == Faction::BeneGesserit {
                                // Make a prediction
                                stack.push(Action::make_prediction(1.5));
                            }
                        }
                    }
                    SetupSubPhase::AtStart => {
                        let entity = info.play_order[info.active_player];
                        let mut players = player_query.iter_mut().collect::<HashMap<Entity, _>>();
                        let active_player_faction = players.get_mut(&entity).unwrap().faction;

                        match active_player_faction {
                            Faction::BeneGesserit | Faction::Fremen => {
                                for &first_entity in info.play_order.iter() {
                                    let first_player_faction =
                                        players.get_mut(&first_entity).unwrap().faction;
                                    // BG is first in turn order
                                    if first_player_faction == Faction::BeneGesserit {
                                        if active_player_faction == Faction::BeneGesserit {
                                            // We go first on the stack so we will go after the fremen
                                            let active_player = players.get_mut(&entity).unwrap();
                                            let (troops, locations, spice) =
                                                active_player.faction.initial_values();
                                            active_player.total_spice += spice;
                                            stack.push(Action::PlaceFreeTroops {
                                                player: entity,
                                                num: troops,
                                                locations,
                                            });

                                            // Then fremen go on, so they will go before BG
                                            let first_player = players.get_mut(&entity).unwrap();
                                            let (troops, locations, spice) =
                                                first_player.faction.initial_values();
                                            first_player.total_spice += spice;
                                            stack.push(Action::PlaceFreeTroops {
                                                player: first_entity,
                                                num: troops,
                                                locations,
                                            });
                                        }
                                        break;
                                    // Fremen is first
                                    } else if first_player_faction == Faction::Fremen {
                                        // Both players go in regular order
                                        let active_player = players.get_mut(&entity).unwrap();
                                        let (troops, locations, spice) =
                                            active_player.faction.initial_values();
                                        active_player.total_spice += spice;
                                        stack.push(Action::PlaceFreeTroops {
                                            player: entity,
                                            num: troops,
                                            locations,
                                        });
                                        break;
                                    }
                                }
                            }
                            _ => {
                                let active_player = players.get_mut(&entity).unwrap();
                                let (troops, locations, spice) =
                                    active_player_faction.initial_values();
                                active_player.total_spice += spice;
                                stack.push(Action::PlaceFreeTroops {
                                    player: entity,
                                    num: troops,
                                    locations,
                                });
                            }
                        }

                        info.active_player += 1;
                        info.active_player %= info.play_order.len();
                    }
                    SetupSubPhase::DealTraitors => {
                        for _ in 0..4 {
                            for &entity in info.play_order.iter() {
                                if let Ok((_, mut player)) = player_query.get_mut(entity) {
                                    player
                                        .traitor_cards
                                        .push(resources.traitor_deck.pop().unwrap());
                                }
                            }
                        }

                        *subphase = SetupSubPhase::PickTraitors;
                    }
                    SetupSubPhase::PickTraitors => stack.push(Action::PickTraitors),
                    SetupSubPhase::DealTreachery => {
                        for &entity in info.play_order.iter() {
                            if let Ok((_, mut player)) = player_query.get_mut(entity) {
                                player
                                    .treachery_cards
                                    .push(resources.treachery_deck.pop().unwrap());
                                if player.faction == Faction::Harkonnen {
                                    player
                                        .treachery_cards
                                        .push(resources.treachery_deck.pop().unwrap());
                                }
                            }
                        }
                        state.phase = Phase::Storm {
                            subphase: StormSubPhase::Reveal,
                        };
                    }
                }
            }
            Phase::Storm { ref mut subphase } => {
                match subphase {
                    StormSubPhase::Reveal => {
                        // Make card visible to everyone
                        if info.turn == 0 {
                            *subphase = StormSubPhase::MoveStorm;
                        } else {
                            *subphase = StormSubPhase::WeatherControl;
                        }
                    }
                    StormSubPhase::WeatherControl => {
                        stack.push(Action::PlayPrompt {
                            treachery_card: "Weather Control".to_string(),
                        });

                        info.active_player += 1;
                        info.active_player %= info.play_order.len();
                    }
                    StormSubPhase::FamilyAtomics => {
                        stack.push(Action::PlayPrompt {
                            treachery_card: "Family Atomics".to_string(),
                        });

                        info.active_player += 1;
                        info.active_player %= info.play_order.len();
                    }
                    StormSubPhase::MoveStorm => {
                        let mut rng = rand::thread_rng();
                        if info.turn == 0 {
                            for mut storm in storm_query.iter_mut() {
                                storm.sector = rng.gen_range(0..18);
                            }
                        } else {
                            let &storm_card = resources.storm_deck.last().unwrap();
                            let delta = storm_cards.get(storm_card).unwrap().val;
                            for mut storm in storm_query.iter_mut() {
                                storm.sector += delta;
                                storm.sector %= 18;
                            }
                            // TODO: Kill everything it passed over and wipe spice
                            resources.storm_deck.shuffle(&mut rng)
                            // TODO: Choose a first player
                            // TODO: Assign bonuses
                        }
                    }
                }
            }
            _ => (),
        }
    }
}

enum ActionResult {
    None,
    Remove,
    Replace { actions: Vec<Action> },
    Add { actions: Vec<Action> },
}

fn handle_actions(
    mut stack: ResMut<ActionStack>,
    mut state: ResMut<State>,
    mut info: ResMut<Info>,
    mut resources: ResMut<Resources>,
    mut player_query: Query<(Entity, &mut Player)>,
    storm_cards: Query<&StormCard>,
    mut cameras: Query<(&mut Transform, &Camera)>,
    mut predictions: QuerySet<(
        Query<(&mut Transform, &FactionPredictionCard)>,
        Query<(&mut Transform, &TurnPredictionCard)>,
    )>,
    mut uniques: Query<(&mut Visible, &Unique)>,
    time: Res<Time>,
) {
    if let Some(action) = stack.peek_mut() {
        match action {
            Action::Simultaneous { actions } => {
                let mut new_actions = Vec::new();
                for action in actions.iter_mut() {
                    match handle_single_action(
                        action,
                        &mut state,
                        &mut info,
                        &mut resources,
                        &mut player_query,
                        &storm_cards,
                        &mut cameras,
                        &mut predictions,
                        &mut uniques,
                        &time,
                    ) {
                        ActionResult::None => new_actions.push(action.clone()),
                        ActionResult::Remove => (),
                        ActionResult::Replace {
                            actions: mut replace_actions,
                        } => new_actions.extend(replace_actions.drain(..)),
                        ActionResult::Add {
                            actions: mut add_actions,
                        } => {
                            new_actions.push(action.clone());
                            new_actions.extend(add_actions.drain(..));
                        }
                    };
                }
                *actions = new_actions;
                if actions.is_empty() {
                    stack.pop();
                }
            }
            _ => {
                match handle_single_action(
                    action,
                    &mut state,
                    &mut info,
                    &mut resources,
                    &mut player_query,
                    &storm_cards,
                    &mut cameras,
                    &mut predictions,
                    &mut uniques,
                    &time,
                ) {
                    ActionResult::None => (),
                    ActionResult::Remove => {
                        stack.pop();
                    }
                    ActionResult::Replace {
                        actions: mut replace_actions,
                    } => {
                        stack.pop();
                        stack.extend(replace_actions.drain(..));
                    }
                    ActionResult::Add {
                        actions: mut add_actions,
                    } => {
                        stack.extend(add_actions.drain(..));
                    }
                };
            }
        }
    }
}

fn handle_single_action(
    action: &mut Action,
    state: &mut ResMut<State>,
    info: &mut ResMut<Info>,
    resources: &mut ResMut<Resources>,
    player_query: &mut Query<(Entity, &mut Player)>,
    storm_cards: &Query<&StormCard>,
    cameras: &mut Query<(&mut Transform, &Camera)>,
    predictions: &mut QuerySet<(
        Query<(&mut Transform, &FactionPredictionCard)>,
        Query<(&mut Transform, &TurnPredictionCard)>,
    )>,
    uniques: &mut Query<(&mut Visible, &Unique)>,
    time: &Res<Time>,
) -> ActionResult {
    match action {
        Action::CameraMotion {
            src,
            dest,
            remaining_time,
            total_time,
        } => {
            if let Some((mut cam_transform, camera)) = cameras.iter_mut().next() {
                if *remaining_time <= 0.0 {
                    *cam_transform =
                        Transform::from_translation(dest.pos).looking_at(dest.at, dest.up);
                    return ActionResult::Remove;
                } else {
                    if cam_transform.translation != dest.pos {
                        if let Some(src_transform) = src {
                            let mut lerp_amount =
                                PI * (*total_time - *remaining_time) / *total_time;
                            lerp_amount = -0.5 * lerp_amount.cos() + 0.5;
                            let dest_transform =
                                Transform::from_translation(dest.pos).looking_at(dest.at, dest.up);
                            *cam_transform = Transform::from_translation(
                                src_transform
                                    .translation
                                    .lerp(dest_transform.translation, lerp_amount),
                            ) * Transform::from_rotation(
                                src_transform
                                    .rotation
                                    .lerp(dest_transform.rotation, lerp_amount),
                            );
                        } else {
                            *src = Some(cam_transform.clone())
                        }
                        *remaining_time -= time.delta_seconds();
                        return ActionResult::None;
                    } else {
                        return ActionResult::Remove;
                    }
                }
            } else {
                return ActionResult::Remove;
            }
        }
        Action::MakePrediction {
            factions_shown,
            faction_picked,
            turns_shown,
            turn_picked,
            animation_time,
            current_time,
            src,
            faction_dests,
            turn_dests,
        } => {
            // Animate faction cards
            if let Some((cam_transform, camera)) = cameras.iter_mut().next() {
                if !*factions_shown {
                    for (mut visible, unique) in uniques.iter_mut() {
                        visible.is_visible = unique.faction == Faction::BeneGesserit;
                    }
                    if current_time >= animation_time {
                        for (i, (mut transform, _)) in predictions.q0_mut().iter_mut().enumerate() {
                            *transform = Transform::from_translation(screen_to_world(
                                faction_dests[i].extend(0.1),
                                *cam_transform,
                                camera.projection_matrix,
                            )) * Transform::from_rotation(cam_transform.rotation)
                                * Transform::from_rotation(Quat::from_rotation_x(0.5 * PI))
                                * Transform::from_scale(Vec3::splat(0.01));
                        }
                        *factions_shown = true;
                        *current_time = 0.0;
                    } else {
                        let num_factions = info.factions_in_play.len();
                        let delay = *animation_time / (2.0 * num_factions as f32);
                        let indiv_anim_time = *animation_time - (delay * (num_factions - 1) as f32);
                        for (i, (mut transform, _)) in predictions.q0_mut().iter_mut().enumerate() {
                            let mut lerp_amount = PI
                                * ((*current_time - (i as f32 * delay))
                                    .max(0.0)
                                    .min(indiv_anim_time))
                                / indiv_anim_time;
                            lerp_amount = -0.5 * lerp_amount.cos() + 0.5;
                            *transform = Transform::from_translation(
                                screen_to_world(
                                    src.extend(0.1),
                                    *cam_transform,
                                    camera.projection_matrix,
                                )
                                .lerp(
                                    screen_to_world(
                                        faction_dests[i].extend(0.1),
                                        *cam_transform,
                                        camera.projection_matrix,
                                    ),
                                    lerp_amount,
                                ),
                            ) * Transform::from_rotation(cam_transform.rotation)
                                * Transform::from_rotation(Quat::from_rotation_x(0.5 * PI))
                                * Transform::from_scale(Vec3::splat(0.01));
                        }
                        *current_time += time.delta_seconds();
                    }
                } else if *faction_picked && !*turns_shown {
                    if current_time >= animation_time {
                        for (i, (mut transform, _)) in predictions.q1_mut().iter_mut().enumerate() {
                            *transform = Transform::from_translation(screen_to_world(
                                turn_dests[i].extend(0.1),
                                *cam_transform,
                                camera.projection_matrix,
                            )) * Transform::from_rotation(cam_transform.rotation)
                                * Transform::from_rotation(Quat::from_rotation_x(0.5 * PI))
                                * Transform::from_scale(Vec3::splat(0.006));
                        }
                        *turns_shown = true;
                        *current_time = 0.0;
                    } else {
                        let delay = *animation_time / (2.0 * 15.0);
                        let indiv_anim_time = *animation_time - (delay * 14.0);
                        for (i, (mut transform, _)) in predictions.q1_mut().iter_mut().enumerate() {
                            let mut lerp_amount = PI
                                * ((*current_time - (i as f32 * delay))
                                    .max(0.0)
                                    .min(indiv_anim_time))
                                / indiv_anim_time;
                            lerp_amount = -0.5 * lerp_amount.cos() + 0.5;
                            *transform = Transform::from_translation(
                                screen_to_world(
                                    src.extend(0.1),
                                    *cam_transform,
                                    camera.projection_matrix,
                                )
                                .lerp(
                                    screen_to_world(
                                        turn_dests[i].extend(0.1),
                                        *cam_transform,
                                        camera.projection_matrix,
                                    ),
                                    lerp_amount,
                                ),
                            ) * Transform::from_rotation(cam_transform.rotation)
                                * Transform::from_rotation(Quat::from_rotation_x(0.5 * PI))
                                * Transform::from_scale(Vec3::splat(0.006));
                        }
                        *current_time += time.delta_seconds();
                    }
                } else if *turn_picked {
                    set_view_to_active_player(info, player_query, uniques);
                    state.phase.advance();
                    return ActionResult::Remove;
                }
            }
            return ActionResult::None;
        }
        _ => {
            return ActionResult::Remove;
        }
    }
}

fn propagate_visibility(
    root: Query<(&Visible, Option<&Children>), (Without<Parent>, Changed<Visible>)>,
    mut children: Query<&mut Visible, With<Parent>>,
) {
    for (visible, root_children) in root.iter() {
        if let Some(root_children) = root_children {
            for &child in root_children.iter() {
                if let Ok(mut child_visible) = children.get_mut(child) {
                    child_visible.is_visible = visible.is_visible;
                }
            }
        }
    }
}

fn screen_to_world(ss_pos: Vec3, transform: Transform, v: Mat4) -> Vec3 {
    let p = transform.compute_matrix() * v.inverse() * ss_pos.extend(1.0);
    p.xyz() / p.w
}

fn set_view_to_active_player(
    info: &ResMut<Info>,
    players: &mut Query<(Entity, &mut Player)>,
    uniques: &mut Query<(&mut Visible, &Unique)>,
) {
    let entity = info.play_order[info.active_player];
    let active_player_faction = players.get_mut(entity).unwrap().1.faction;
    for (mut visible, unique) in uniques.iter_mut() {
        visible.is_visible = unique.faction == active_player_faction;
    }
}
