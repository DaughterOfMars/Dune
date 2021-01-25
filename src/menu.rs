use bevy::{prelude::*, render::camera::PerspectiveProjection};

use crate::{Screen, RESPONSE_STAGE, STATE_CHANGE_STAGE};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut bevy::prelude::AppBuilder) {
        app.add_startup_system(init_camera.system())
            .add_startup_system(init_main_menu.system())
            .add_stage_after(
                stage::UPDATE,
                STATE_CHANGE_STAGE,
                StateStage::<Screen>::default(),
            )
            .add_stage_after(
                STATE_CHANGE_STAGE,
                RESPONSE_STAGE,
                StateStage::<Screen>::default(),
            )
            .init_resource::<ButtonMaterials>()
            .on_state_enter(RESPONSE_STAGE, Screen::MainMenu, init_main_menu.system())
            .on_state_exit(RESPONSE_STAGE, Screen::MainMenu, tear_down.system())
            .on_state_update(STATE_CHANGE_STAGE, Screen::MainMenu, button_system.system())
            .on_state_update(STATE_CHANGE_STAGE, Screen::Join, button_system.system())
            .on_state_update(STATE_CHANGE_STAGE, Screen::Server, button_system.system());
    }
}

struct MenuEntity;

enum ButtonActionType {
    HostGame,
    JoinGame,
}

struct ButtonAction {
    action_type: ButtonActionType,
}

struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    pressed: Handle<ColorMaterial>,
}

impl FromResources for ButtonMaterials {
    fn from_resources(resources: &Resources) -> Self {
        let mut materials = resources.get_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
        }
    }
}

fn button_system(
    mut state: ResMut<State<crate::Screen>>,
    button_materials: Res<ButtonMaterials>,
    mut interactions: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &ButtonAction),
        (Mutated<Interaction>, With<Button>),
    >,
) {
    for (&interaction, mut material, action) in interactions.iter_mut() {
        match interaction {
            Interaction::Clicked => {
                *material = button_materials.pressed.clone();
                match action.action_type {
                    ButtonActionType::HostGame => {
                        state.set_next(crate::Screen::Server).unwrap();
                    }
                    ButtonActionType::JoinGame => {
                        state.set_next(crate::Screen::Join).unwrap();
                    }
                }
            }
            Interaction::Hovered => *material = button_materials.hovered.clone(),
            Interaction::None => *material = button_materials.normal.clone(),
        }
    }
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

fn init_main_menu(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    button_materials: Res<ButtonMaterials>,
) {
    commands
        .spawn(CameraUiBundle::default())
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
        .with(MenuEntity)
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Percent(10.0), Val::Percent(6.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    material: button_materials.normal.clone(),
                    ..Default::default()
                })
                .with(ButtonAction {
                    action_type: ButtonActionType::HostGame,
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            value: "Host Game".to_string(),
                            style: TextStyle {
                                font_size: 20.0,
                                color: Color::ANTIQUE_WHITE,
                                ..Default::default()
                            },
                        },
                        ..Default::default()
                    });
                })
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Percent(10.0), Val::Percent(6.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    material: button_materials.normal.clone(),
                    ..Default::default()
                })
                .with(ButtonAction {
                    action_type: ButtonActionType::JoinGame,
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            value: "Join Game".to_string(),
                            style: TextStyle {
                                font_size: 20.0,
                                color: Color::ANTIQUE_WHITE,
                                ..Default::default()
                            },
                        },
                        ..Default::default()
                    });
                });
        });
}

fn tear_down(commands: &mut Commands, menu_entities: Query<Entity, With<MenuEntity>>) {
    for entity in menu_entities.iter() {
        commands.despawn_recursive(entity);
    }
}
