mod components;
mod data;
mod game;
mod input;
mod lerper;
mod resources;
mod stack;
mod util;

use std::{collections::HashMap, f32::consts::PI};

use bevy::{
    asset::LoadState,
    math::vec3,
    prelude::*,
    render::{
        camera::{PerspectiveProjection, Projection},
        mesh::Indices,
        render_resource::PrimitiveTopology,
        view::RenderLayers,
    },
    utils::default,
};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_mod_picking::{DefaultPickingPlugins, PickableBundle, PickingCameraBundle};
use iyes_loopless::{
    prelude::{AppLooplessStateExt, IntoConditionalSystem},
    state::NextState,
};
use rand::seq::SliceRandom;

use self::{components::*, game::*, input::GameInputPlugin, lerper::LerpPlugin, resources::*, util::card_jitter};

pub const MAX_PLAYERS: u8 = 6;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Screen {
    MainMenu,
    Loading,
    Game,
}

#[derive(Component)]
struct GameEntity;

#[derive(Default)]
struct LoadingAssets {
    assets: Vec<HandleUntyped>,
}

pub struct Active {
    pub entity: Entity,
}

pub struct NextActive {
    pub entity: Entity,
}

fn main() {
    if let Err(e) = dotenv::dotenv() {
        error!("{}", e);
    }
    let mut app = App::new();
    app.insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::BLACK))
        .init_resource::<Data>()
        .init_resource::<Info>()
        .init_resource::<LoadingAssets>();

    app.add_loopless_state(Screen::Loading);

    app.add_startup_system(init_cameras)
        .add_system(active_cam_picker.run_if_resource_exists::<NextActive>());

    app.add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(GameInputPlugin)
        .add_plugin(GamePlugin)
        .add_plugin(LerpPlugin)
        .add_plugins(DefaultPickingPlugins);

    app.add_enter_system(Screen::Loading, init_loading_game);
    app.add_system(load_game.run_in_state(Screen::Loading));
    app.add_exit_system(Screen::Loading, tear_down);
    app.add_enter_system(Screen::Game, init_game);

    app.run();
}

fn init_cameras(mut commands: Commands, mut info: ResMut<Info>) {
    let proj: Projection = PerspectiveProjection {
        near: 0.01,
        far: 100.0,
        ..default()
    }
    .into();
    let trans = Transform::from_translation(vec3(0.0, 2.5, 2.0)).looking_at(Vec3::ZERO, Vec3::Y)
        * Transform::from_translation(vec3(0.0, -0.4, 0.0));
    let primary_cam = commands
        .spawn_bundle(Camera3dBundle {
            projection: proj.clone(),
            transform: trans,
            camera: Camera {
                priority: 1,
                is_active: true,
                ..default()
            },
            ..default()
        })
        .insert(UiCameraConfig::default())
        .insert(RenderLayers::default().with(1))
        .insert(Player::new())
        .insert_bundle(PickingCameraBundle::default())
        .id();
    commands.insert_resource(Active { entity: primary_cam });
    info.turn_order.push(primary_cam);
    for index in 2..MAX_PLAYERS + 1 {
        info.turn_order.push(
            commands
                .spawn_bundle(Camera3dBundle {
                    projection: proj.clone(),
                    transform: trans,
                    camera: Camera {
                        priority: index as _,
                        is_active: false,
                        ..default()
                    },
                    ..default()
                })
                .insert(UiCameraConfig::default())
                .insert(RenderLayers::default().with(index))
                .insert(Player::new())
                .id(),
        );
    }
}

fn active_cam_picker(
    mut commands: Commands,
    mut active: ResMut<Active>,
    next_active: Res<NextActive>,
    mut cams: Query<(Entity, &mut Camera)>,
) {
    if next_active.entity != active.entity {
        active.entity = next_active.entity;
        for (entity, mut camera) in cams.iter_mut() {
            if active.entity == entity {
                camera.is_active = true;
                commands.entity(entity).insert_bundle(PickingCameraBundle::default());
            } else {
                camera.is_active = false;
                commands.entity(entity).remove_bundle::<PickingCameraBundle>();
            }
        }
    }
    commands.remove_resource::<NextActive>();
}

#[derive(Component)]
struct LoadingBar;

fn init_loading_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_assets: ResMut<LoadingAssets>,
) {
    loading_assets.assets = asset_server.load_folder(".").unwrap();

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .insert(GameEntity)
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(50.0), Val::Percent(10.0)),
                        margin: UiRect::all(Val::Auto),
                        border: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    color: Color::BLACK.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                ..default()
                            },
                            color: Color::RED.into(),
                            ..default()
                        })
                        .insert(LoadingBar);
                });
        });
}

fn load_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    loading_assets: Res<LoadingAssets>,
    mut loading_bar: Query<&mut Style, With<LoadingBar>>,
) {
    let mut counts = HashMap::new();
    for handle in loading_assets.assets.iter() {
        match asset_server.get_load_state(handle) {
            LoadState::NotLoaded => *counts.entry("loading").or_insert(0) += 1,
            LoadState::Loading => *counts.entry("loading").or_insert(0) += 1,
            LoadState::Loaded => *counts.entry("loaded").or_insert(0) += 1,
            LoadState::Failed => *counts.entry("failed").or_insert(0) += 1,
            LoadState::Unloaded => *counts.entry("unloaded").or_insert(0) += 1,
        }
    }
    loading_bar.iter_mut().next().map(|mut bar| {
        bar.size.width =
            Val::Percent(100.0 * (*counts.entry("loaded").or_insert(0) as f32 / loading_assets.assets.len() as f32));
    });
    if *counts.entry("loading").or_insert(0) == 0 {
        commands.insert_resource(NextState(Screen::Game));
    }
}

fn init_game(
    mut commands: Commands,
    mut info: ResMut<Info>,
    data: Res<Data>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::thread_rng();
    info.turn_order.shuffle(&mut rng);
    // Light
    commands
        .spawn_bundle(PointLightBundle {
            transform: Transform::from_translation(vec3(10.0, 10.0, 10.0)),
            ..default()
        })
        .insert(GameEntity);

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });

    commands.spawn_bundle((Storm::default(),)).insert(GameEntity);

    // Board
    commands
        .spawn_bundle(SceneBundle {
            scene: asset_server.get_handle("board.gltf#Scene0"),
            ..default()
        })
        .insert_bundle(PickableBundle::default())
        .insert(GameEntity)
        .insert(data.camera_nodes.board);

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..default()
                },
                ..default()
            },
            text: Text::from_section(
                "Test",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 40.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            ..default()
        })
        .insert(GameEntity)
        .insert(PhaseText);

    for (location, location_data) in data.locations.iter() {
        commands
            .spawn_bundle(SpatialBundle::default())
            .insert(location.clone())
            .insert(GameEntity)
            .with_children(|parent| {
                for (&sector, nodes) in location_data.sectors.iter() {
                    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
                    mesh.insert_attribute(
                        Mesh::ATTRIBUTE_POSITION,
                        nodes.vertices.iter().map(|p| [p.x, 0.01, -p.y]).collect::<Vec<_>>(),
                    );
                    mesh.set_indices(Some(Indices::U32(nodes.indices.clone())));
                    mesh.duplicate_vertices();
                    mesh.compute_flat_normals();
                    mesh.compute_aabb();
                    parent
                        .spawn_bundle(PbrBundle {
                            mesh: meshes.add(mesh),
                            material: materials.add(StandardMaterial::from(Color::rgba(1.0, 1.0, 1.0, 0.0))),
                            visibility: Visibility { is_visible: true },
                            ..default()
                        })
                        .insert_bundle(PickableBundle::default())
                        .insert(LocationSector {
                            location: location.clone(),
                            sector,
                        });
                }
            });

        if let Some(pos) = location_data.spice {
            commands.spawn().insert(SpiceNode::new(pos));
        }
    }

    let card_face = asset_server.get_handle("card.gltf#Mesh0/Primitive0");
    let card_back = asset_server.get_handle("card.gltf#Mesh0/Primitive1");

    let treachery_back_texture = asset_server.get_handle("treachery/treachery_back.png");
    let treachery_back_material = materials.add(StandardMaterial::from(treachery_back_texture));

    commands
        .spawn_bundle((Deck(vec![]),))
        .insert_bundle(SpatialBundle::from_transform(
            Transform::from_translation(vec3(1.23, 0.0049, -0.87))
                * Transform::from_rotation(Quat::from_rotation_z(PI)),
        ))
        .with_children(|parent| {
            for (i, card_data) in data.treachery_deck.iter().enumerate() {
                let treachery_front_texture =
                    asset_server.get_handle(format!("treachery/treachery_{}.png", card_data.texture.as_str()).as_str());
                let treachery_front_material = materials.add(StandardMaterial::from(treachery_front_texture));

                parent
                    .spawn_bundle((Card, card_data.card.clone(), GameEntity))
                    .insert_bundle(SpatialBundle::from_transform(
                        Transform::from_translation(Vec3::Y * 0.001 * (i as f32)) * card_jitter(),
                    ))
                    .with_children(|parent| {
                        parent.spawn_bundle(PbrBundle {
                            mesh: card_face.clone(),
                            material: treachery_front_material,
                            ..default()
                        });
                        parent.spawn_bundle(PbrBundle {
                            mesh: card_back.clone(),
                            material: treachery_back_material.clone(),
                            ..default()
                        });
                    });
            }
        });

    let traitor_back_texture = asset_server.get_handle("traitor/traitor_back.png");
    let traitor_back_material = materials.add(StandardMaterial::from(traitor_back_texture));

    commands
        .spawn_bundle((Deck(vec![]),))
        .insert_bundle(SpatialBundle::from_transform(
            Transform::from_translation(vec3(1.23, 0.0049, -0.3)) * Transform::from_rotation(Quat::from_rotation_z(PI)),
        ))
        .with_children(|parent| {
            for (i, (leader, leader_data)) in data.leaders.iter().enumerate() {
                let traitor_front_texture =
                    asset_server.get_handle(format!("traitor/traitor_{}.png", leader_data.texture.as_str()).as_str());
                let traitor_front_material = materials.add(StandardMaterial::from(traitor_front_texture));

                parent
                    .spawn_bundle((Card, TraitorCard { leader: leader.clone() }))
                    .insert_bundle(SpatialBundle::from_transform(
                        Transform::from_translation(Vec3::Y * 0.001 * (i as f32)) * card_jitter(),
                    ))
                    .insert(GameEntity)
                    .with_children(|parent| {
                        parent.spawn_bundle(PbrBundle {
                            mesh: card_face.clone(),
                            material: traitor_front_material,
                            ..default()
                        });
                        parent.spawn_bundle(PbrBundle {
                            mesh: card_back.clone(),
                            material: traitor_back_material.clone(),
                            ..default()
                        });
                    });
            }
        });

    let spice_back_texture = asset_server.get_handle("spice/spice_back.png");
    let spice_back_material = materials.add(StandardMaterial::from(spice_back_texture));

    commands
        .spawn_bundle((Deck(vec![]),))
        .insert_bundle(SpatialBundle {
            transform: Transform::from_translation(vec3(1.23, 0.0049, 0.3))
                * Transform::from_rotation(Quat::from_rotation_z(PI)),
            ..default()
        })
        .with_children(|parent| {
            for (i, (card, card_data)) in data.spice_cards.iter().enumerate() {
                let spice_front_texture =
                    asset_server.get_handle(format!("spice/spice_{}.png", card_data.texture.as_str()).as_str());
                let spice_front_material = materials.add(StandardMaterial::from(spice_front_texture));

                parent
                    .spawn_bundle((Card, card.clone()))
                    .insert_bundle(SpatialBundle::from_transform(
                        Transform::from_translation(Vec3::Y * 0.001 * (i as f32)) * card_jitter(),
                    ))
                    .insert(GameEntity)
                    .with_children(|parent| {
                        parent.spawn_bundle(PbrBundle {
                            mesh: card_face.clone(),
                            material: spice_front_material,
                            ..default()
                        });
                        parent.spawn_bundle(PbrBundle {
                            mesh: card_back.clone(),
                            material: spice_back_material.clone(),
                            ..default()
                        });
                    });
            }
        });

    let storm_back_texture = asset_server.get_handle("storm/storm_back.png");
    let storm_back_material = materials.add(StandardMaterial::from(storm_back_texture));

    commands
        .spawn_bundle((Deck(vec![]),))
        .insert_bundle(SpatialBundle::from_transform(
            Transform::from_translation(vec3(1.23, 0.0049, 0.87)) * Transform::from_rotation(Quat::from_rotation_z(PI)),
        ))
        .with_children(|parent| {
            for val in 1..=6 {
                let storm_front_texture = asset_server.get_handle(format!("storm/storm_{}.png", val).as_str());
                let storm_front_material = materials.add(StandardMaterial::from(storm_front_texture));

                parent
                    .spawn_bundle((Card, StormCard { val }))
                    .insert_bundle(SpatialBundle::from_transform(
                        Transform::from_translation(Vec3::Y * 0.001 * (val as f32)) * card_jitter(),
                    ))
                    .insert(GameEntity)
                    .with_children(|parent| {
                        parent.spawn_bundle(PbrBundle {
                            mesh: card_face.clone(),
                            material: storm_front_material,
                            ..default()
                        });
                        parent.spawn_bundle(PbrBundle {
                            mesh: card_back.clone(),
                            material: storm_back_material.clone(),
                            ..default()
                        });
                    });
            }
        });

    let deck_mesh = meshes.add(Mesh::from(shape::Box::new(0.25, 0.06, 0.36)));

    commands
        .spawn_bundle(PbrBundle {
            mesh: deck_mesh.clone(),
            transform: Transform::from_translation(data.camera_nodes.treachery.at),
            material: materials.add(StandardMaterial::from(Color::rgba(1.0, 1.0, 1.0, 0.0))),
            ..default()
        })
        .insert_bundle(PickableBundle::default())
        .insert(GameEntity)
        .insert(data.camera_nodes.treachery);

    commands
        .spawn_bundle(PbrBundle {
            mesh: deck_mesh.clone(),
            transform: Transform::from_translation(data.camera_nodes.traitor.at),
            material: materials.add(StandardMaterial::from(Color::rgba(1.0, 1.0, 1.0, 0.0))),
            ..default()
        })
        .insert_bundle(PickableBundle::default())
        .insert(GameEntity)
        .insert(data.camera_nodes.traitor);

    commands
        .spawn_bundle(PbrBundle {
            mesh: deck_mesh.clone(),
            transform: Transform::from_translation(data.camera_nodes.spice.at),
            material: materials.add(StandardMaterial::from(Color::rgba(1.0, 1.0, 1.0, 0.0))),
            ..default()
        })
        .insert_bundle(PickableBundle::default())
        .insert(GameEntity)
        .insert(data.camera_nodes.spice);

    commands
        .spawn_bundle(PbrBundle {
            mesh: deck_mesh,
            transform: Transform::from_translation(data.camera_nodes.storm.at),
            material: materials.add(StandardMaterial::from(Color::rgba(1.0, 1.0, 1.0, 0.0))),
            ..default()
        })
        .insert_bundle(PickableBundle::default())
        .insert(GameEntity)
        .insert(data.camera_nodes.storm);
}

fn tear_down(mut commands: Commands, screen_entities: Query<Entity, With<GameEntity>>) {
    for entity in screen_entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
