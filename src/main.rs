#[macro_use]
mod resources;
mod components;
mod data;
mod input;
mod lerper;
mod menu;
mod network;
mod phase;
mod stack;
mod util;

use components::*;
use data::*;
use input::GameInputPlugin;
use lerper::LerpPlugin;
use menu::MenuPlugin;
use network::*;
use phase::*;
use resources::*;
use util::divide_spice;

use bevy::{asset::LoadState, prelude::*, render::camera::PerspectiveProjection};

use bytecheck::CheckBytes;
use rkyv::{check_archive, Archive, ArchiveWriter, Seek, Unarchive, Write};

use ncollide3d::{
    na::{Point3, Vector3},
    shape::{ConvexHull, Cuboid, Cylinder, ShapeHandle, TriMesh},
    transformation::ToTriMesh,
};

use rand::seq::SliceRandom;

use std::{collections::HashMap, f32::consts::PI, io::Cursor};

#[derive(Copy, Clone, Debug)]
pub enum Screen {
    MainMenu,
    Server,
    Join,
    Loading,
    HostingGame,
    JoinedGame,
}

struct ScreenEntity;

#[derive(Archive, Unarchive, PartialEq, Clone, Debug)]
#[archive(derive(CheckBytes))]
pub enum MessageData {
    Load,
    Loaded,
    ServerInfo { players: Vec<String> },
}

impl MessageData {
    fn into_bytes(&self) -> Vec<u8> {
        let mut writer = ArchiveWriter::new(Cursor::new(Vec::new()));
        writer
            .archive_root(self)
            .expect("Failed to serialize message data!");
        writer.into_inner().into_inner()
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let archived = check_archive::<Self>(bytes, 0).expect("Failed to validate message data!");
        archived.unarchive()
    }
}

const STATE_CHANGE_STAGE: &str = "state_change";
const RESPONSE_STAGE: &str = "response";
const END_STAGE: &str = "end";

#[derive(Default)]
struct LoadingAssets {
    assets: Vec<HandleUntyped>,
}

fn main() {
    let mut app = App::build();
    app.add_resource(Msaa { samples: 4 })
        .add_resource(ClearColor(Color::BLACK))
        .init_resource::<Data>()
        .init_resource::<Info>()
        .init_resource::<LoadingAssets>();

    app.add_resource(State::new(Screen::MainMenu));

    app.add_stage_after(
        stage::UPDATE,
        STATE_CHANGE_STAGE,
        StateStage::<Screen>::default(),
    )
    .add_stage_after(
        STATE_CHANGE_STAGE,
        RESPONSE_STAGE,
        StateStage::<Screen>::default(),
    );

    app.add_plugins(DefaultPlugins)
        .add_plugin(GameInputPlugin)
        .add_plugin(PhasePlugin)
        .add_plugin(LerpPlugin)
        .add_plugin(MenuPlugin)
        .add_plugin(NetworkPlugin);

    app.add_stage(END_STAGE, SystemStage::parallel())
        .add_system_to_stage(END_STAGE, propagate_visibility.system())
        .add_startup_system(init_camera.system());

    app.on_state_enter(RESPONSE_STAGE, Screen::Loading, init_loading_game.system())
        .on_state_update(STATE_CHANGE_STAGE, Screen::Loading, load_game.system())
        .on_state_exit(RESPONSE_STAGE, Screen::Loading, tear_down.system());

    app.on_state_enter(RESPONSE_STAGE, Screen::HostingGame, init_game.system())
        .on_state_exit(RESPONSE_STAGE, Screen::HostingGame, tear_down.system())
        .on_state_exit(RESPONSE_STAGE, Screen::HostingGame, reset_game.system());

    app.on_state_update(
        STATE_CHANGE_STAGE,
        Screen::Server,
        process_network_messages.system(),
    );

    app.run();
}

fn init_camera(commands: &mut Commands) {
    commands
        .spawn(Camera3dBundle {
            perspective_projection: PerspectiveProjection {
                near: 0.01,
                far: 100.0,
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 2.5, 2.0))
                .looking_at(Vec3::zero(), Vec3::unit_y())
                * Transform::from_translation(Vec3::new(0.0, -0.4, 0.0)),
            ..Default::default()
        })
        .spawn(CameraUiBundle::default());
}

struct LoadingBar;

fn init_loading_game(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut loading_assets: ResMut<LoadingAssets>,
    mut colors: ResMut<Assets<ColorMaterial>>,
) {
    loading_assets.assets = asset_server.load_folder(".").unwrap();

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            ..Default::default()
        })
        .with(ScreenEntity)
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(50.0), Val::Percent(10.0)),
                        margin: Rect::all(Val::Auto),
                        border: Rect::all(Val::Px(5.0)),
                        ..Default::default()
                    },
                    material: colors.add(Color::BLACK.into()),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                ..Default::default()
                            },
                            material: colors.add(Color::RED.into()),
                            ..Default::default()
                        })
                        .with(LoadingBar);
                });
        });
}

fn load_game(
    mut state: ResMut<State<Screen>>,
    asset_server: Res<AssetServer>,
    loading_assets: Res<LoadingAssets>,
    mut loading_bar: Query<&mut Style, With<LoadingBar>>,
    network: Res<Network>,
) {
    let mut counts = HashMap::new();
    for handle in loading_assets.assets.iter() {
        match asset_server.get_load_state(handle) {
            LoadState::NotLoaded => *counts.entry("loading").or_insert(0) += 1,
            LoadState::Loading => *counts.entry("loading").or_insert(0) += 1,
            LoadState::Loaded => *counts.entry("loaded").or_insert(0) += 1,
            LoadState::Failed => *counts.entry("failed").or_insert(0) += 1,
        }
    }
    loading_bar.iter_mut().next().map(|mut bar| {
        bar.size.width = Val::Percent(
            100.0
                * (*counts.entry("loaded").or_insert(0) as f32
                    / loading_assets.assets.len() as f32),
        );
    });
    if *counts.entry("loading").or_insert(0) == 0 {
        state
            .set_next(match network.network_type {
                NetworkType::Server => Screen::HostingGame,
                NetworkType::Client => Screen::JoinedGame,
                NetworkType::None => Screen::MainMenu,
            })
            .unwrap();
    }
}

fn init_game(
    commands: &mut Commands,
    data: Res<Data>,
    mut info: ResMut<Info>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut colors: ResMut<Assets<ColorMaterial>>,
) {
    // Light
    commands
        .spawn(LightBundle {
            transform: Transform::from_translation(Vec3::new(10.0, 10.0, 10.0)),
            ..Default::default()
        })
        .with(ScreenEntity);

    commands.spawn((Storm::default(),)).with(ScreenEntity);

    // Board
    info.default_clickables.push(
        commands
            .spawn(ColliderBundle::new(ShapeHandle::new(Cuboid::new(
                Vector3::new(1.0, 0.007, 1.1),
            ))))
            .with(ScreenEntity)
            .with(data.camera_nodes.board)
            .with_children(|parent| {
                parent.spawn_scene(asset_server.get_handle("board.gltf"));
            })
            .current_entity()
            .unwrap(),
    );

    commands
        .spawn(TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text {
                font: asset_server.get_handle("fonts/FiraSans-Bold.ttf"),
                value: "Test".to_string(),
                style: TextStyle {
                    font_size: 40.0,
                    color: Color::ANTIQUE_WHITE,
                    ..Default::default()
                },
            },
            ..Default::default()
        })
        .with(ScreenEntity)
        .with(PhaseText);

    for location in data.locations.iter() {
        commands
            .spawn((location.clone(),))
            .with(ScreenEntity)
            .with_children(|parent| {
                for (&sector, nodes) in location.sectors.iter() {
                    let vertices = nodes
                        .vertices
                        .iter()
                        .map(|p| Point3::new(p.x, 0.01, -p.y))
                        .collect();
                    let indices = nodes
                        .indices
                        .chunks_exact(3)
                        .map(|chunk| {
                            Point3::new(chunk[0] as usize, chunk[1] as usize, chunk[2] as usize)
                        })
                        .collect();
                    parent
                        .spawn(ColliderBundle::new(ShapeHandle::new(TriMesh::new(
                            vertices, indices, None,
                        ))))
                        .with(LocationSector {
                            location: location.clone(),
                            sector,
                        });
                }
            });

        if let Some(pos) = location.spice {
            commands.with(SpiceNode::new(pos));
        }
    }

    let card_face = asset_server.get_handle("card.gltf#Mesh0/Primitive0");
    let card_back = asset_server.get_handle("card.gltf#Mesh0/Primitive1");

    let treachery_back_texture = asset_server.get_handle("treachery/treachery_back.png");
    let treachery_back_material = materials.add(StandardMaterial {
        albedo_texture: Some(treachery_back_texture),
        ..Default::default()
    });

    for (i, card) in data.treachery_cards.iter().enumerate() {
        let treachery_front_texture = asset_server
            .get_handle(format!("treachery/treachery_{}.png", card.texture.as_str()).as_str());
        let treachery_front_material = materials.add(StandardMaterial {
            albedo_texture: Some(treachery_front_texture),
            ..Default::default()
        });

        commands
            .spawn((
                card.clone(),
                Transform::from_translation(Vec3::new(1.23, 0.0049 + (i as f32 * 0.001), -0.87))
                    * Transform::from_rotation(Quat::from_rotation_z(PI)),
                GlobalTransform::default(),
            ))
            .with(ScreenEntity)
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
            });
    }

    let traitor_back_texture = asset_server.get_handle("traitor/traitor_back.png");
    let traitor_back_material = materials.add(StandardMaterial {
        albedo_texture: Some(traitor_back_texture),
        ..Default::default()
    });

    for (i, card) in data.leaders.iter().enumerate() {
        let traitor_front_texture = asset_server
            .get_handle(format!("traitor/traitor_{}.png", card.texture.as_str()).as_str());
        let traitor_front_material = materials.add(StandardMaterial {
            albedo_texture: Some(traitor_front_texture),
            ..Default::default()
        });

        commands
            .spawn((
                TraitorCard {
                    leader: card.clone(),
                },
                Transform::from_translation(Vec3::new(1.23, 0.0049 + (i as f32 * 0.001), -0.3))
                    * Transform::from_rotation(Quat::from_rotation_z(PI)),
                GlobalTransform::default(),
            ))
            .with(ScreenEntity)
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
            });
    }

    let spice_back_texture = asset_server.get_handle("spice/spice_back.png");
    let spice_back_material = materials.add(StandardMaterial {
        albedo_texture: Some(spice_back_texture),
        ..Default::default()
    });

    for (i, card) in data.spice_cards.iter().enumerate() {
        let spice_front_texture =
            asset_server.get_handle(format!("spice/spice_{}.png", card.texture.as_str()).as_str());
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
            .with(ScreenEntity)
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
            });
    }

    let storm_back_texture = asset_server.get_handle("storm/storm_back.png");
    let storm_back_material = materials.add(StandardMaterial {
        albedo_texture: Some(storm_back_texture),
        ..Default::default()
    });

    for val in 1..7 {
        let storm_front_texture =
            asset_server.get_handle(format!("storm/storm_{}.png", val).as_str());
        let storm_front_material = materials.add(StandardMaterial {
            albedo_texture: Some(storm_front_texture),
            ..Default::default()
        });

        commands
            .spawn((
                StormCard { val },
                Transform::from_translation(Vec3::new(1.23, 0.0049 + (val as f32 * 0.001), 0.87))
                    * Transform::from_rotation(Quat::from_rotation_z(PI)),
                GlobalTransform::default(),
            ))
            .with(ScreenEntity)
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
            });
    }

    let deck_shape = ShapeHandle::new(Cuboid::new(Vector3::new(0.125, 0.03, 0.18)));

    info.default_clickables.push(
        commands
            .spawn(
                ColliderBundle::new(deck_shape.clone())
                    .with_transform(Transform::from_translation(data.camera_nodes.treachery.at)),
            )
            .with(ScreenEntity)
            .with(data.camera_nodes.treachery)
            .current_entity()
            .unwrap(),
    );

    info.default_clickables.push(
        commands
            .spawn(
                ColliderBundle::new(deck_shape.clone())
                    .with_transform(Transform::from_translation(data.camera_nodes.traitor.at)),
            )
            .with(ScreenEntity)
            .with(data.camera_nodes.traitor)
            .current_entity()
            .unwrap(),
    );

    info.default_clickables.push(
        commands
            .spawn(
                ColliderBundle::new(deck_shape.clone())
                    .with_transform(Transform::from_translation(data.camera_nodes.spice.at)),
            )
            .with(ScreenEntity)
            .with(data.camera_nodes.spice)
            .current_entity()
            .unwrap(),
    );

    info.default_clickables.push(
        commands
            .spawn(
                ColliderBundle::new(deck_shape)
                    .with_transform(Transform::from_translation(data.camera_nodes.storm.at)),
            )
            .with(ScreenEntity)
            .with(data.camera_nodes.storm)
            .current_entity()
            .unwrap(),
    );
}

fn process_network_messages(
    mut info: ResMut<Info>,
    mut state: ResMut<State<Screen>>,
    network: Res<Network>,
    mut server: Query<&mut Server>,
    mut client: Query<&mut Client>,
) {
    match network.network_type {
        NetworkType::Client => {
            if let Some(mut client) = client.iter_mut().next() {
                for data in client.messages.drain(..) {
                    let message = MessageData::from_bytes(&data[..]);
                    match message {
                        MessageData::Load => {
                            state.overwrite_next(Screen::Loading).unwrap();
                        }
                        MessageData::ServerInfo { players } => {
                            info.players = players;
                        }
                        _ => (),
                    }
                }
            }
        }
        NetworkType::Server => if let Some(mut server) = server.iter_mut().next() {},
        NetworkType::None => (),
    }
}

fn propagate_visibility(
    root: Query<(&Visible, &Children), (Without<Parent>, Changed<Visible>)>,
    mut children: Query<&mut Visible, With<Parent>>,
) {
    for (visible, root_children) in root.iter() {
        for &child in root_children.iter() {
            if let Ok(mut child_visible) = children.get_mut(child) {
                if child_visible.is_visible != visible.is_visible {
                    child_visible.is_visible = visible.is_visible;
                }
            }
        }
    }
}

fn tear_down(commands: &mut Commands, screen_entities: Query<Entity, With<ScreenEntity>>) {
    for entity in screen_entities.iter() {
        commands.despawn_recursive(entity);
    }
}

fn reset_game(mut info: ResMut<Info>) {
    info.reset();
}
