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
    math::{uvec2, uvec3},
    prelude::*,
    render::camera::{PerspectiveProjection, Projection, Viewport},
    utils::default,
    window::{WindowId, WindowResized},
};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_mod_picking::{DefaultPickingPlugins, PickableBundle};

use self::{components::*, game::*, input::GameInputPlugin, lerper::LerpPlugin, resources::*, util::card_jitter};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Screen {
    MainMenu,
    Loading,
    Game,
}

#[derive(Component)]
struct ScreenEntity;

#[derive(Default)]
struct LoadingAssets {
    assets: Vec<HandleUntyped>,
}

fn main() {
    let mut app = App::new();
    app.insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::BLACK))
        .init_resource::<Data>()
        .init_resource::<Info>()
        .init_resource::<LoadingAssets>();

    app.add_startup_system(init_camera).add_system(set_camera_viewports);

    app.add_state(Screen::MainMenu);

    app.add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(GameInputPlugin)
        .add_plugin(GamePlugin)
        .add_plugin(LerpPlugin)
        .add_plugins(DefaultPickingPlugins);

    app.add_system_set(SystemSet::on_enter(Screen::Loading).with_system(init_loading_game))
        .add_system_set(SystemSet::on_update(Screen::Loading).with_system(load_game))
        .add_system_set(SystemSet::on_exit(Screen::Loading).with_system(tear_down));

    app.run();
}

#[derive(Component)]
struct CameraPosition {
    index: u8,
}

fn init_camera(mut commands: Commands) {
    let proj: Projection = PerspectiveProjection {
        near: 0.01,
        far: 100.0,
        ..default()
    }
    .into();
    let trans = Transform::from_translation(Vec3::new(0.0, 2.5, 2.0)).looking_at(Vec3::ZERO, Vec3::Y)
        * Transform::from_translation(Vec3::new(0.0, -0.4, 0.0));
    for index in 0..6 {
        commands
            .spawn_bundle(Camera3dBundle {
                projection: proj.clone(),
                transform: trans,
                ..default()
            })
            .insert(UiCameraConfig::default())
            .insert(CameraPosition { index });
    }
}

fn set_camera_viewports(
    windows: Res<Windows>,
    mut resize_events: EventReader<WindowResized>,
    mut cams: Query<(&mut Camera, &CameraPosition), With<Active>>,
) {
    for resize_event in resize_events.iter() {
        if resize_event.id == WindowId::primary() {
            let window = windows.primary();
            let total = cams.iter().count();
            for (mut camera, CameraPosition { index }) in cams.iter_mut() {
                let (cols, rows) = match total {
                    1 => (1, 1),
                    2 => (2, 1),
                    3 => (2, 2),
                    4 => (2, 2),
                    5 => (3, 2),
                    6 => (3, 2),
                    _ => unreachable!(),
                };
                let (col, row, center) = match index {
                    0 => (0, 0, false),
                    1 => (1, 0, false),
                    2 => match cols {
                        2 => (0, 1, true),
                        3 => (2, 0, false),
                        _ => unreachable!(),
                    },
                    3 => match cols {
                        2 => (1, 1, false),
                        3 => (0, 1, true),
                        _ => unreachable!(),
                    },
                    4 => (1, 1, true),
                    5 => (2, 1, false),
                    _ => unreachable!(),
                };
                let physical_size = uvec2(window.physical_height() / rows, window.physical_width() / cols);
                let physical_position = uvec2(col * physical_size.x, row * physical_size.y);
                camera.viewport.replace(Viewport {
                    physical_position,
                    physical_size,
                    ..default()
                });
            }
        }
    }
}

#[derive(Component)]
struct LoadingBar;

fn init_loading_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_assets: ResMut<LoadingAssets>,
    mut colors: ResMut<Assets<ColorMaterial>>,
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
        .insert(ScreenEntity)
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
    mut state: ResMut<State<Screen>>,
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
        state.set(Screen::MainMenu).unwrap();
    }
}

fn init_game(
    mut commands: Commands,
    data: Res<Data>,
    mut info: ResMut<Info>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut colors: ResMut<Assets<ColorMaterial>>,
) {
    // Light
    // commands
    //     .spawn_bundle(PointLightBundle {
    //         transform: Transform::from_translation(Vec3::new(10.0, 10.0, 10.0)),
    //         ..default()
    //     })
    //     .insert(ScreenEntity);

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });

    commands.spawn_bundle((Storm::default(),)).insert(ScreenEntity);

    // Board
    info.default_clickables.push(
        commands
            .spawn_bundle(PickableBundle::default())
            .insert(ScreenEntity)
            .insert(data.camera_nodes.board)
            .with_children(|parent| {
                parent.spawn_bundle(DynamicSceneBundle {
                    scene: asset_server.get_handle("board.gltf"),
                    ..default()
                });
            })
            .id(),
    );

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
                    color: Color::BLACK,
                    ..default()
                },
            ),
            ..default()
        })
        .insert(ScreenEntity)
        .insert(PhaseText);

    for (location, location_data) in data.locations.iter() {
        commands
            .spawn()
            .insert(location.clone())
            .insert(ScreenEntity)
            .with_children(|parent| {
                for (&sector, nodes) in location_data.sectors.iter() {
                    // TODO spawn these shapes
                    // let vertices = nodes.vertices.iter().map(|p| Vec3::new(p.x, 0.01, -p.y)).collect();
                    // let indices = nodes
                    //     .indices
                    //     .chunks_exact(3)
                    //     .map(|chunk| uvec3(chunk[0] as u32, chunk[1] as u32, chunk[2] as u32))
                    //     .collect();
                    parent.spawn_bundle(PickableBundle::default()).insert(LocationSector {
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
        .spawn()
        .insert(Deck(vec![]))
        .insert(
            Transform::from_translation(Vec3::new(1.23, 0.0049, -0.87))
                * Transform::from_rotation(Quat::from_rotation_z(PI)),
        )
        .insert(GlobalTransform::default())
        .with_children(|parent| {
            for (i, card_data) in data.treachery_deck.iter().enumerate() {
                let treachery_front_texture =
                    asset_server.get_handle(format!("treachery/treachery_{}.png", card_data.texture.as_str()).as_str());
                let treachery_front_material = materials.add(StandardMaterial::from(treachery_front_texture));

                parent
                    .spawn()
                    .insert(Card)
                    .insert(card_data.card.clone())
                    .insert(Transform::from_translation(Vec3::Y * 0.001 * (i as f32)) * card_jitter())
                    .insert(GlobalTransform::default())
                    .insert(ScreenEntity)
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
        .spawn_bundle((
            Deck(vec![]),
            Transform::from_translation(Vec3::new(1.23, 0.0049, -0.3))
                * Transform::from_rotation(Quat::from_rotation_z(PI)),
            GlobalTransform::default(),
        ))
        .with_children(|parent| {
            for (i, (leader, leader_data)) in data.leaders.iter().enumerate() {
                let traitor_front_texture =
                    asset_server.get_handle(format!("traitor/traitor_{}.png", leader_data.texture.as_str()).as_str());
                let traitor_front_material = materials.add(StandardMaterial::from(traitor_front_texture));

                parent
                    .spawn_bundle((
                        Card,
                        TraitorCard { leader: leader.clone() },
                        Transform::from_translation(Vec3::Y * 0.001 * (i as f32)) * card_jitter(),
                        GlobalTransform::default(),
                    ))
                    .insert(ScreenEntity)
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
        .spawn_bundle((
            Deck(vec![]),
            Transform::from_translation(Vec3::new(1.23, 0.0049, 0.3))
                * Transform::from_rotation(Quat::from_rotation_z(PI)),
            GlobalTransform::default(),
        ))
        .with_children(|parent| {
            for (i, (card, card_data)) in data.spice_cards.iter().enumerate() {
                let spice_front_texture =
                    asset_server.get_handle(format!("spice/spice_{}.png", card_data.texture.as_str()).as_str());
                let spice_front_material = materials.add(StandardMaterial::from(spice_front_texture));

                parent
                    .spawn_bundle((
                        Card,
                        card.clone(),
                        Transform::from_translation(Vec3::Y * 0.001 * (i as f32)) * card_jitter(),
                        GlobalTransform::default(),
                    ))
                    .insert(ScreenEntity)
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
        .spawn_bundle((
            Deck(vec![]),
            Transform::from_translation(Vec3::new(1.23, 0.0049, 0.87))
                * Transform::from_rotation(Quat::from_rotation_z(PI)),
            GlobalTransform::default(),
        ))
        .with_children(|parent| {
            for val in 1..=6 {
                let storm_front_texture = asset_server.get_handle(format!("storm/storm_{}.png", val).as_str());
                let storm_front_material = materials.add(StandardMaterial::from(storm_front_texture));

                parent
                    .spawn_bundle((
                        Card,
                        StormCard { val },
                        Transform::from_translation(Vec3::Y * 0.001 * (val as f32)) * card_jitter(),
                        GlobalTransform::default(),
                    ))
                    .insert(ScreenEntity)
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

    // TODO: do something like this? Maybe just add pickers to each card I dunno
    // let deck_shape = ShapeHandle::new(Cuboid::new(Vector3::new(0.125, 0.03, 0.18)));
    //
    // info.default_clickables.push(
    //     commands
    //         .spawn_bundle(
    //             ColliderBundle::new(deck_shape.clone())
    //                 .with_transform(Transform::from_translation(data.camera_nodes.treachery.at)),
    //         )
    //         .insert(ScreenEntity)
    //         .insert(data.camera_nodes.treachery)
    //         .current_entity()
    //         .unwrap(),
    // );
    //
    // info.default_clickables.push(
    //     commands
    //         .spawn_bundle(
    //             ColliderBundle::new(deck_shape.clone())
    //                 .with_transform(Transform::from_translation(data.camera_nodes.traitor.at)),
    //         )
    //         .insert(ScreenEntity)
    //         .insert(data.camera_nodes.traitor)
    //         .current_entity()
    //         .unwrap(),
    // );
    //
    // info.default_clickables.push(
    //     commands
    //         .spawn_bundle(
    //             ColliderBundle::new(deck_shape.clone())
    //                 .with_transform(Transform::from_translation(data.camera_nodes.spice.at)),
    //         )
    //         .insert(ScreenEntity)
    //         .insert(data.camera_nodes.spice)
    //         .current_entity()
    //         .unwrap(),
    // );
    //
    // info.default_clickables.push(
    //     commands
    //         .spawn_bundle(
    //
    // ColliderBundle::new(deck_shape).with_transform(Transform::from_translation(data.camera_nodes.storm.at)),
    //         )
    //         .insert(ScreenEntity)
    //         .insert(data.camera_nodes.storm)
    //         .current_entity()
    //         .unwrap(),
    // );
}

fn tear_down(mut commands: Commands, screen_entities: Query<Entity, With<ScreenEntity>>) {
    for entity in screen_entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn reset_game(mut info: ResMut<Info>) {
    info.reset();
}
