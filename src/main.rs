use bevy::{
    math::Vec4Swizzles,
    prelude::*,
    render::camera::{Camera, PerspectiveProjection},
};

mod data;
use data::*;
use ncollide3d::{
    na::{Isometry3, Point3, Translation3, UnitQuaternion, Vector3},
    query::{Ray, RayCast},
    shape::{Cuboid, Shape},
};

use std::{
    collections::HashMap,
    fs::File,
    ops::{Deref, DerefMut},
};

use rand::{seq::SliceRandom, Rng};

use std::f32::consts::PI;

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
        .add_system(input.system())
        .add_system(handle_actions.system())
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
    shape: Box<dyn Shape<f32>>,
}

#[derive(Bundle)]
struct ColliderBundle {
    transform: Transform,
    global_transform: GlobalTransform,
    collider: Collider,
    click_action: Option<ClickAction>,
    hover_action: Option<HoverAction>,
}

impl ColliderBundle {
    fn new(
        shape: impl Shape<f32>,
        click_action: Option<Action>,
        hover_action: Option<Action>,
    ) -> Self {
        Self {
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            collider: Collider {
                shape: Box::new(shape),
            },
            click_action: click_action.and_then(|action| Some(ClickAction { action })),
            hover_action: hover_action.and_then(|action| Some(HoverAction { action })),
        }
    }

    fn with_transform(
        shape: impl Shape<f32>,
        transform: Transform,
        click_action: Option<Action>,
        hover_action: Option<Action>,
    ) -> Self {
        Self {
            transform,
            global_transform: GlobalTransform::default(),
            collider: Collider {
                shape: Box::new(shape),
            },
            click_action: click_action.and_then(|action| Some(ClickAction { action })),
            hover_action: hover_action.and_then(|action| Some(HoverAction { action })),
        }
    }
}

#[derive(Bundle)]
struct UniqueBundle {
    unique: Unique,
    visible: Visible,
}

impl UniqueBundle {
    fn new(faction: Faction) -> Self {
        Self {
            unique: Unique { faction },
            visible: Visible {
                is_visible: false,
                ..Default::default()
            },
        }
    }
}

#[derive(Clone)]
struct ClickAction {
    action: Action,
}

#[derive(Clone)]
struct HoverAction {
    action: Action,
}

#[derive(Copy, Clone, Default, Debug)]
struct Prediction {
    faction: Option<Faction>,
    turn: Option<i32>,
}

#[derive(Clone, Debug)]
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
    // Execute a series of actions in order
    Sequence {
        actions: Vec<Action>,
    },
    // Wait for some action to occur before resolving this one
    Await {
        timer: Option<f32>,
    },
    Delay {
        time: f32,
        action: Box<Action>,
    },
    Show {
        element: Entity,
    },
    Hide {
        element: Entity,
    },
    AnimateUIElement {
        element: Entity,
        src: Vec2,
        dest: Vec2,
        animation_time: f32,
        current_time: f32,
    },
    Animate3DElement {
        element: Entity,
        src: Option<Transform>,
        dest: Transform,
        animation_time: f32,
        current_time: f32,
    },
    AnimateUIElementTo3D {
        element: Entity,
        src: Vec2,
        dest: Transform,
        animation_time: f32,
        current_time: f32,
    },
    Animate3DElementToUI {
        element: Entity,
        src: Option<Transform>,
        dest: Vec2,
        animation_time: f32,
        current_time: f32,
    },
    MakePrediction {
        prediction: Prediction,
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
    AdvancePhase,
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

    fn animate_ui(element: Entity, src: Vec2, dest: Vec2, time: f32) -> Self {
        Action::AnimateUIElement {
            element,
            src,
            dest,
            animation_time: time,
            current_time: 0.0,
        }
    }

    fn animate_3d_to_ui(element: Entity, src: Option<Transform>, dest: Vec2, time: f32) -> Self {
        Action::Animate3DElementToUI {
            element,
            src,
            dest,
            animation_time: time,
            current_time: 0.0,
        }
    }

    fn animate_3d(element: Entity, src: Option<Transform>, dest: Transform, time: f32) -> Self {
        Action::Animate3DElement {
            element,
            src,
            dest,
            animation_time: time,
            current_time: 0.0,
        }
    }

    fn await_indefinite() -> Self {
        Action::Await { timer: None }
    }

    fn await_timed(time: f32) -> Self {
        Action::Await { timer: Some(time) }
    }

    fn delay(action: Action, time: f32) -> Self {
        Action::Delay {
            action: Box::new(action),
            time,
        }
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Choice { player, options } => {
                let s = options
                    .iter()
                    .map(|option| option.to_string())
                    .collect::<Vec<String>>()
                    .join("/");
                write!(f, "Choice({})", s)
            }
            Action::Simultaneous { actions } => {
                let s = actions
                    .iter()
                    .map(|action| action.to_string())
                    .collect::<Vec<String>>()
                    .join(" + ");
                write!(f, "Simul({})", s)
            }
            Action::Sequence { actions } => {
                let s = actions
                    .iter()
                    .map(|action| action.to_string())
                    .collect::<Vec<String>>()
                    .join(" -> ");
                write!(f, "Seq({})", s)
            }
            Action::Await { timer } => {
                if let Some(timer) = timer {
                    write!(f, "Await(remaining={})", timer)
                } else {
                    write!(f, "Await(Forever)")
                }
            }
            Action::Delay { time, action } => write!(f, "Delay({}, remaining={})", action, time),
            Action::Show { element } => write!(f, "Show({:?})", *element),
            Action::Hide { element } => write!(f, "Hide({:?})", *element),
            Action::AnimateUIElement {
                element,
                src,
                dest,
                animation_time,
                current_time,
            } => write!(f, "AnimateUIElement"),
            Action::Animate3DElement {
                element,
                src,
                dest,
                animation_time,
                current_time,
            } => write!(f, "Animate3DElement"),
            Action::AnimateUIElementTo3D {
                element,
                src,
                dest,
                animation_time,
                current_time,
            } => write!(f, "AnimateUIElementTo3D"),
            Action::Animate3DElementToUI {
                element,
                src,
                dest,
                animation_time,
                current_time,
            } => write!(f, "Animate3DElementToUI"),
            Action::MakePrediction { prediction } => {
                if let Some(faction) = prediction.faction {
                    write!(f, "MakePrediction({:?})", faction)
                } else {
                    if let Some(turn) = prediction.turn {
                        write!(f, "MakePrediction({})", turn)
                    } else {
                        write!(f, "MakePrediction")
                    }
                }
            }
            Action::PlaceFreeTroops {
                player,
                num,
                locations,
            } => write!(f, "PlaceFreeTroops"),
            Action::PlaceTroops {
                player,
                num,
                locations,
            } => write!(f, "PlaceTroops"),
            Action::PickTraitors => write!(f, "PickTraitors"),
            Action::PlayPrompt { treachery_card } => write!(f, "PlayPrompt"),
            Action::CameraMotion {
                src,
                dest,
                remaining_time,
                total_time,
            } => write!(f, "CameraMotion"),
            Action::AdvancePhase => write!(f, "AdvancePhase"),
        }
    }
}

struct Data {
    leaders: Vec<Leader>,
    locations: Vec<Location>,
    treachery_cards: Vec<TreacheryCard>,
    spice_cards: Vec<SpiceCard>,
    camera_nodes: CameraNodes,
    prediction_nodes: PredictionNodes,
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
        let prediction_nodes =
            ron::de::from_reader(File::open("src/prediction_nodes.ron").unwrap()).unwrap();
        Data {
            locations,
            leaders,
            treachery_cards,
            spice_cards,
            camera_nodes,
            prediction_nodes,
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
        .spawn(ColliderBundle::new(
            Cuboid::new(Vector3::new(1.0, 0.007, 1.1)),
            Some(Action::move_camera(data.camera_nodes.board, 1.5)),
            None,
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
                .spawn(ColliderBundle::with_transform(
                    Cuboid::new(Vector3::new(0.525, 0.285, 0.06)),
                    Transform::from_translation(Vec3::new(0.0, 0.27, 1.34)),
                    Some(Action::move_camera(data.camera_nodes.shield, 1.5)),
                    None,
                ))
                .with_bundle(UniqueBundle::new(faction))
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
                .spawn(ColliderBundle::with_transform(
                    Cuboid::new(Vector3::new(0.125, 0.0005, 0.18) * 0.01),
                    Transform::from_scale(Vec3::splat(0.01))
                        * Transform::from_rotation(Quat::from_rotation_x(0.5 * PI)),
                    Some(Action::MakePrediction {
                        prediction: Prediction {
                            faction: Some(faction),
                            turn: None,
                        },
                    }),
                    None,
                ))
                .with_bundle(UniqueBundle::new(Faction::BeneGesserit))
                .with(FactionPredictionCard { faction })
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
            .spawn(ColliderBundle::with_transform(
                Cuboid::new(Vector3::new(0.125, 0.0005, 0.18) * 0.006),
                Transform::from_scale(Vec3::splat(0.006))
                    * Transform::from_rotation(Quat::from_rotation_x(0.5 * PI)),
                Some(Action::MakePrediction {
                    prediction: Prediction {
                        faction: None,
                        turn: Some(turn),
                    },
                }),
                None,
            ))
            .with_bundle(UniqueBundle::new(Faction::BeneGesserit))
            .with(TurnPredictionCard { turn })
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

    commands.spawn(ColliderBundle::with_transform(
        Cuboid::new(Vector3::new(0.125, 0.03, 0.18)),
        Transform::from_translation(data.camera_nodes.treachery.at),
        Some(Action::move_camera(data.camera_nodes.treachery, 1.5)),
        None,
    ));

    commands.spawn(ColliderBundle::with_transform(
        Cuboid::new(Vector3::new(0.125, 0.03, 0.18)),
        Transform::from_translation(data.camera_nodes.traitor.at),
        Some(Action::move_camera(data.camera_nodes.traitor, 1.5)),
        None,
    ));

    commands.spawn(ColliderBundle::with_transform(
        Cuboid::new(Vector3::new(0.125, 0.03, 0.18)),
        Transform::from_translation(data.camera_nodes.spice.at),
        Some(Action::move_camera(data.camera_nodes.spice, 1.5)),
        None,
    ));

    commands.spawn(ColliderBundle::with_transform(
        Cuboid::new(Vector3::new(0.125, 0.03, 0.18)),
        Transform::from_translation(data.camera_nodes.storm.at),
        Some(Action::move_camera(data.camera_nodes.storm, 1.5)),
        None,
    ));
}

fn input(
    mut stack: ResMut<ActionStack>,
    info: Res<Info>,
    data: Res<Data>,
    windows: Res<Windows>,
    mouse_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    cameras: Query<(&Transform, &Camera)>,
    colliders: Query<(&Collider, &Transform, &Option<ClickAction>)>,
) {
    match stack.peek_mut() {
        Some(Action::Await { .. }) | None => {
            if mouse_input.just_pressed(MouseButton::Left) {
                if let Some((&cam_transform, camera)) = cameras.iter().next() {
                    if let Some(window) = windows.get_primary() {
                        if let Some(pos) = window.cursor_position() {
                            let ss_pos = Vec2::new(
                                2.0 * (pos.x / window.physical_width() as f32) - 1.0,
                                2.0 * (pos.y / window.physical_height() as f32) - 1.0,
                            );
                            let p0 = screen_to_world(
                                ss_pos.extend(0.0),
                                cam_transform,
                                camera.projection_matrix,
                            );
                            let p1 = screen_to_world(
                                ss_pos.extend(1.0),
                                cam_transform,
                                camera.projection_matrix,
                            );
                            let dir = (p1 - p0).normalize();
                            let ray = Ray::new(
                                Point3::new(p0.x, p0.y, p0.z),
                                Vector3::new(dir.x, dir.y, dir.z),
                            );
                            let (mut closest_toi, mut closest_action) = (None, None);
                            for (collider, transform, action) in
                                colliders
                                    .iter()
                                    .filter_map(|(collider, transform, action)| {
                                        if let Some(action) = action {
                                            Some((collider, transform, action))
                                        } else {
                                            None
                                        }
                                    })
                            {
                                let (axis, angle) = transform.rotation.to_axis_angle();
                                let angleaxis = axis * angle;
                                if let Some(toi) = collider.shape.toi_with_ray(
                                    &Isometry3::from_parts(
                                        Translation3::new(
                                            transform.translation.x,
                                            transform.translation.y,
                                            transform.translation.z,
                                        ),
                                        UnitQuaternion::new(Vector3::new(
                                            angleaxis.x,
                                            angleaxis.y,
                                            angleaxis.z,
                                        )),
                                    ),
                                    &ray,
                                    100.0,
                                    true,
                                ) {
                                    if closest_toi.is_none() {
                                        closest_toi = Some(toi);
                                        closest_action = Some(action);
                                    } else {
                                        if toi < closest_toi.unwrap() {
                                            closest_toi = Some(toi);
                                            closest_action = Some(action);
                                        }
                                    }
                                }
                            }
                            if let Some(ClickAction { action }) = closest_action {
                                stack.pop();
                                stack.push(action.clone());
                            }
                        }
                    }
                }
            } else if keyboard_input.just_pressed(KeyCode::Escape) {
                stack.push(Action::move_camera(data.camera_nodes.main, 1.5));
            }
        }
        _ => (),
    }
}

fn handle_phase(
    mut stack: ResMut<ActionStack>,
    mut state: ResMut<State>,
    mut info: ResMut<Info>,
    data: Res<Data>,
    mut resources: ResMut<Resources>,
    mut player_query: Query<(Entity, &mut Player)>,
    mut storm_query: Query<&mut Storm>,
    storm_cards: Query<&StormCard>,
    mut unique_query: Query<(&mut Visible, &Unique)>,
    prediction_cards: Query<(Entity, &FactionPredictionCard)>,
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
                                for (mut visible, unique) in unique_query.iter_mut() {
                                    visible.is_visible = unique.faction == Faction::BeneGesserit;
                                }
                                // Animate in faction prediction cards
                                let num_factions = info.factions_in_play.len();
                                let animation_time = 1.5;
                                let delay = animation_time / (2.0 * num_factions as f32);
                                let indiv_anim_time =
                                    animation_time - (delay * (num_factions - 1) as f32);
                                let in_actions: Vec<Action> = prediction_cards
                                    .iter()
                                    .enumerate()
                                    .map(|(i, (element, _))| Action::Simultaneous {
                                        actions: vec![
                                            Action::animate_3d_to_ui(
                                                element,
                                                None,
                                                data.prediction_nodes.src,
                                                0.0,
                                            ),
                                            Action::delay(
                                                Action::animate_ui(
                                                    element,
                                                    data.prediction_nodes.src,
                                                    data.prediction_nodes.factions[i],
                                                    indiv_anim_time,
                                                ),
                                                delay * i as f32,
                                            ),
                                        ],
                                    })
                                    .collect();
                                let in_action = Action::Simultaneous {
                                    actions: in_actions,
                                };
                                let sequence = Action::Sequence {
                                    actions: vec![in_action, Action::await_indefinite()],
                                };
                                stack.push(sequence);
                            }
                        }
                    }
                    SetupSubPhase::AtStart => {
                        set_view_to_active_player(&info, &mut player_query, &mut unique_query);
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
    Replace { action: Action },
    Add { action: Action },
}

fn handle_actions(
    mut stack: ResMut<ActionStack>,
    mut state: ResMut<State>,
    mut info: ResMut<Info>,
    data: Res<Data>,
    mut resources: ResMut<Resources>,
    mut player_query: Query<(Entity, &mut Player)>,
    storm_cards: Query<&StormCard>,
    mut cameras: Query<(&mut Transform, &Camera)>,
    mut uniques: Query<(&mut Visible, &Unique)>,
    mut transforms: Query<&mut Transform, Without<Camera>>,
    time: Res<Time>,
    mut predictions: Query<&mut Prediction>,
    prediction_cards: QuerySet<(
        Query<(Entity, &FactionPredictionCard)>,
        Query<(Entity, &TurnPredictionCard)>,
    )>,
) {
    /*
    print!("Stack: ");
    for item in stack.0.iter().rev() {
        print!("{}, ", item);
    }
    println!();
    */
    if let Some(mut action) = stack.pop() {
        match handle_action_recursive(
            &mut action,
            &mut stack,
            &mut state,
            &mut info,
            &data,
            &mut resources,
            &mut player_query,
            &storm_cards,
            &mut cameras,
            &mut uniques,
            &time,
            &mut transforms,
            &mut predictions,
            &prediction_cards,
        ) {
            ActionResult::None => {
                stack.push(action);
            }
            ActionResult::Remove => (),
            ActionResult::Replace {
                action: replace_action,
            } => {
                stack.push(replace_action);
            }
            ActionResult::Add { action: add_action } => {
                stack.push(action);
                stack.push(add_action);
            }
        };
    }
}

fn handle_action_recursive(
    action: &mut Action,
    stack: &mut ResMut<ActionStack>,
    state: &mut ResMut<State>,
    info: &mut ResMut<Info>,
    data: &Res<Data>,
    resources: &mut ResMut<Resources>,
    player_query: &mut Query<(Entity, &mut Player)>,
    storm_cards: &Query<&StormCard>,
    cameras: &mut Query<(&mut Transform, &Camera)>,
    uniques: &mut Query<(&mut Visible, &Unique)>,
    time: &Res<Time>,
    transforms: &mut Query<&mut Transform, Without<Camera>>,
    predictions: &mut Query<&mut Prediction>,
    prediction_cards: &QuerySet<(
        Query<(Entity, &FactionPredictionCard)>,
        Query<(Entity, &TurnPredictionCard)>,
    )>,
) -> ActionResult {
    match action {
        Action::Simultaneous { actions } => {
            let mut new_actions = Vec::new();
            for mut action in actions.drain(..) {
                match handle_action_recursive(
                    &mut action,
                    stack,
                    state,
                    info,
                    data,
                    resources,
                    player_query,
                    storm_cards,
                    cameras,
                    uniques,
                    time,
                    transforms,
                    predictions,
                    prediction_cards,
                ) {
                    ActionResult::None => {
                        new_actions.push(action);
                    }
                    ActionResult::Remove => (),
                    ActionResult::Replace {
                        action: replace_action,
                    } => {
                        new_actions.push(replace_action);
                    }
                    ActionResult::Add { action: add_action } => {
                        new_actions.push(action);
                        new_actions.push(add_action);
                    }
                };
            }
            if !new_actions.is_empty() {
                ActionResult::Replace {
                    action: Action::Simultaneous {
                        actions: new_actions,
                    },
                }
            } else {
                ActionResult::Remove
            }
        }
        Action::Sequence { actions } => {
            actions.reverse();
            stack.extend(actions.drain(..));
            if let Some(mut action) = stack.pop() {
                handle_action_recursive(
                    &mut action,
                    stack,
                    state,
                    info,
                    data,
                    resources,
                    player_query,
                    storm_cards,
                    cameras,
                    uniques,
                    time,
                    transforms,
                    predictions,
                    prediction_cards,
                )
            } else {
                ActionResult::Remove
            }
        }
        Action::CameraMotion {
            src,
            dest,
            remaining_time,
            total_time,
        } => {
            if let Some((mut cam_transform, _)) = cameras.iter_mut().next() {
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
        Action::AnimateUIElement {
            element,
            src,
            dest,
            animation_time,
            current_time,
        } => {
            if let Ok(mut element_transform) = transforms.get_mut(*element) {
                if let Some((cam_transform, camera)) = cameras.iter_mut().next() {
                    if *current_time >= *animation_time {
                        element_transform.translation = screen_to_world(
                            dest.extend(0.1),
                            *cam_transform,
                            camera.projection_matrix,
                        );

                        return ActionResult::Remove;
                    } else {
                        let mut lerp_amount =
                            PI * (*current_time).min(*animation_time) / *animation_time;
                        lerp_amount = -0.5 * lerp_amount.cos() + 0.5;
                        element_transform.translation = screen_to_world(
                            src.extend(0.1),
                            *cam_transform,
                            camera.projection_matrix,
                        )
                        .lerp(
                            screen_to_world(
                                dest.extend(0.1),
                                *cam_transform,
                                camera.projection_matrix,
                            ),
                            lerp_amount,
                        );

                        *current_time += time.delta_seconds();
                    }
                }
            }
            ActionResult::None
        }
        Action::Animate3DElementToUI {
            element,
            src,
            dest,
            animation_time,
            current_time,
        } => {
            if let Ok(mut element_transform) = transforms.get_mut(*element) {
                if let Some((cam_transform, camera)) = cameras.iter_mut().next() {
                    if src.is_none() {
                        src.replace(element_transform.clone());
                    }
                    if *current_time >= *animation_time {
                        element_transform.translation = screen_to_world(
                            dest.extend(0.1),
                            *cam_transform,
                            camera.projection_matrix,
                        );
                        element_transform.rotation = cam_transform.rotation * src.unwrap().rotation;

                        return ActionResult::Remove;
                    } else {
                        let mut lerp_amount =
                            PI * (*current_time).min(*animation_time) / *animation_time;
                        lerp_amount = -0.5 * lerp_amount.cos() + 0.5;
                        element_transform.translation = src.unwrap().translation.lerp(
                            screen_to_world(
                                dest.extend(0.1),
                                *cam_transform,
                                camera.projection_matrix,
                            ),
                            lerp_amount,
                        );
                        element_transform.rotation = src
                            .unwrap()
                            .rotation
                            .lerp(cam_transform.rotation * src.unwrap().rotation, lerp_amount);

                        *current_time += time.delta_seconds();
                    }
                }
            }
            ActionResult::None
        }
        Action::MakePrediction { prediction } => {
            for mut player_prediction in predictions.iter_mut() {
                player_prediction.faction = prediction.faction.or(player_prediction.faction);
                player_prediction.turn = prediction.turn.or(player_prediction.turn);
            }
            if prediction.faction.is_some() {
                let num_factions = info.factions_in_play.len();
                let animation_time = 1.5;
                let mut delay = animation_time / (2.0 * num_factions as f32);
                let mut indiv_anim_time = animation_time - (delay * (num_factions - 1) as f32);
                // Animate selected card
                let chosen_action = prediction_cards
                    .q0()
                    .iter()
                    .enumerate()
                    .find(|(_, (_, card))| card.faction == prediction.faction.unwrap())
                    .map(|(i, (element, _))| {
                        Action::animate_ui(
                            element,
                            data.prediction_nodes.factions[i],
                            data.prediction_nodes.chosen_faction,
                            1.0,
                        )
                    })
                    .unwrap();
                // Animate out faction cards
                let mut out_actions: Vec<Action> = prediction_cards
                    .q0()
                    .iter()
                    .filter(|(_, card)| card.faction != prediction.faction.unwrap())
                    .enumerate()
                    .map(|(i, (element, _))| {
                        Action::delay(
                            Action::animate_ui(
                                element,
                                data.prediction_nodes.factions[i],
                                data.prediction_nodes.src,
                                indiv_anim_time,
                            ),
                            1.0 + (delay * i as f32),
                        )
                    })
                    .collect();
                out_actions.push(chosen_action);
                let out_action = Action::Simultaneous {
                    actions: out_actions,
                };
                // Animate in turn cards
                delay = animation_time / 30.0;
                indiv_anim_time = animation_time - (delay * 14.0);
                let in_actions: Vec<Action> = prediction_cards
                    .q1()
                    .iter()
                    .enumerate()
                    .map(|(i, (element, _))| Action::Simultaneous {
                        actions: vec![
                            Action::animate_3d_to_ui(element, None, data.prediction_nodes.src, 0.0),
                            Action::delay(
                                Action::animate_ui(
                                    element,
                                    data.prediction_nodes.src,
                                    data.prediction_nodes.turns[i],
                                    indiv_anim_time,
                                ),
                                delay * i as f32,
                            ),
                        ],
                    })
                    .collect();
                let in_action = Action::Simultaneous {
                    actions: in_actions,
                };
                return ActionResult::Replace {
                    action: Action::Sequence {
                        actions: vec![out_action, in_action, Action::await_indefinite()],
                    },
                };
            } else if prediction.turn.is_some() {
                let animation_time = 1.5;
                let delay = animation_time / 30.0;
                let indiv_anim_time = animation_time - (delay * 14.0);
                // Animate selected card
                let chosen_action = prediction_cards
                    .q1()
                    .iter()
                    .enumerate()
                    .find(|(_, (_, card))| card.turn == prediction.turn.unwrap())
                    .map(|(i, (element, _))| {
                        Action::animate_ui(
                            element,
                            data.prediction_nodes.turns[i],
                            data.prediction_nodes.chosen_turn,
                            1.0,
                        )
                    })
                    .unwrap();
                // Animate out turn cards
                let mut out_actions: Vec<Action> = prediction_cards
                    .q1()
                    .iter()
                    .filter(|(_, card)| card.turn != prediction.turn.unwrap())
                    .enumerate()
                    .map(|(i, (element, _))| {
                        Action::delay(
                            Action::animate_ui(
                                element,
                                data.prediction_nodes.turns[i],
                                data.prediction_nodes.src,
                                indiv_anim_time,
                            ),
                            1.0 + (delay * i as f32),
                        )
                    })
                    .collect();
                out_actions.push(chosen_action);
                let out_action = Action::Simultaneous {
                    actions: out_actions,
                };
                return ActionResult::Replace {
                    action: Action::Sequence {
                        actions: vec![out_action, Action::delay(Action::AdvancePhase, 1.5)],
                    },
                };
            }
            ActionResult::Remove
        }
        Action::Await { timer } => {
            if let Some(timer) = timer {
                *timer -= time.delta_seconds();
                if *timer <= 0.0 {
                    ActionResult::Remove
                } else {
                    ActionResult::None
                }
            } else {
                ActionResult::None
            }
        }
        Action::Delay {
            action: delayed,
            time: timer,
        } => {
            *timer -= time.delta_seconds();
            if *timer <= 0.0 {
                ActionResult::Replace {
                    action: delayed.deref_mut().clone(),
                }
            } else {
                ActionResult::None
            }
        }
        Action::AdvancePhase => {
            state.phase.advance();
            ActionResult::Remove
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
