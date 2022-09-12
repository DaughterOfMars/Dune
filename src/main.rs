#![feature(hash_drain_filter)]

mod components;
mod data;
mod game;
mod input;
mod lerper;
mod menu;
mod network;
mod stack;
mod util;

use std::collections::HashMap;

use bevy::{
    asset::LoadState,
    math::vec3,
    prelude::*,
    render::{camera::PerspectiveProjection, mesh::Indices, render_resource::PrimitiveTopology},
    utils::default,
};
#[cfg(feature = "debug")]
use bevy_editor_pls::EditorPlugin;
use bevy_mod_picking::{DefaultPickingPlugins, PickableBundle, PickingCameraBundle};
use bevy_renet::RenetClientPlugin;
use data::Data;
use iyes_loopless::{
    prelude::{AppLooplessStateExt, IntoConditionalSystem},
    state::NextState,
};
use lerper::{LerpUICamera, Lerper};
use network::{SendEvent, ServerEvent};
use renet::RenetClient;

use self::{
    components::*, game::*, input::GameInputPlugin, lerper::LerpPlugin, menu::MenuPlugin,
    network::RenetNetworkingPlugin,
};

pub const MAX_PLAYERS: u8 = 6;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Screen {
    MainMenu,
    Host,
    Join,
    Loading,
    Game,
}

#[derive(Default)]
struct LoadingAssets {
    assets: Vec<HandleUntyped>,
}

fn main() {
    if let Err(e) = dotenv::dotenv() {
        error!("{}", e);
    }
    let mut app = App::new();
    app.insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::BLACK))
        .init_resource::<LoadingAssets>()
        .init_resource::<Data>();

    app.add_loopless_state(Screen::MainMenu);

    app.add_plugins(DefaultPlugins);

    #[cfg(feature = "debug")]
    app.add_plugin(EditorPlugin);

    app.add_plugin(RenetClientPlugin)
        .add_plugin(RenetNetworkingPlugin)
        .add_plugins(DefaultPickingPlugins);

    app.add_startup_system(init_camera);

    app.add_system(start_game);
    app.add_enter_system(Screen::Loading, tear_down.chain(init_loading_game));
    app.add_system(load_game.run_in_state(Screen::Loading));
    app.add_enter_system(Screen::Game, tear_down.chain(init_scene));

    app.add_plugin(GamePlugin)
        .add_plugin(MenuPlugin)
        .add_plugin(GameInputPlugin)
        .add_plugin(LerpPlugin);

    app.run();
}

fn init_camera(mut commands: Commands) {
    commands
        .spawn_bundle(Camera3dBundle {
            projection: PerspectiveProjection {
                near: 0.01,
                far: 100.0,
                ..default()
            }
            .into(),
            transform: Transform::from_translation(vec3(0.0, 2.5, 2.0)).looking_at(Vec3::ZERO, Vec3::Y)
                * Transform::from_translation(vec3(0.0, -0.4, 0.0)),
            ..default()
        })
        .insert(UiCameraConfig::default())
        .insert_bundle(PickingCameraBundle::default())
        .insert_bundle((Lerper::default(), LerpUICamera));
}

fn start_game(mut commands: Commands, mut server_events: EventReader<ServerEvent>) {
    for event in server_events.iter() {
        if let ServerEvent::LoadAssets = event {
            commands.insert_resource(NextState(Screen::Loading));
        }
    }
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
    mut client: ResMut<RenetClient>,
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
        client.send_event(ServerEvent::StartGame);
    }
}

fn init_scene(
    mut commands: Commands,
    data: Res<Data>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_translation(vec3(10.0, 10.0, 10.0)),
        ..default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });

    commands.spawn_bundle((Storm::default(),));

    // Board
    commands
        .spawn_bundle(SceneBundle {
            scene: asset_server.get_handle("board.gltf#Scene0"),
            ..default()
        })
        .insert_bundle(PickableBundle::default())
        .insert(data.camera_nodes.board);

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(5.0),
                    right: Val::Px(5.0),
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
        .insert(PlayerFactionText);

    for (location, location_data) in data.locations.iter() {
        commands
            .spawn_bundle(SpatialBundle::default())
            .insert(*location)
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
                        .insert(LocationSector {
                            location: *location,
                            sector,
                        })
                        .insert_bundle(PickableBundle::default());
                }
            });

        if let Some(pos) = location_data.spice {
            commands.spawn().insert(SpiceNode::new(pos));
        }
    }
}

fn tear_down(mut commands: Commands, screen_entities: Query<Entity, Without<Camera>>) {
    for entity in screen_entities.iter() {
        commands.entity(entity).despawn();
    }
}
