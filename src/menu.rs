use bevy::prelude::*;

use crate::{tear_down, Screen, ScreenEntity, RESPONSE_STAGE, STATE_CHANGE_STAGE};
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut bevy::prelude::AppBuilder) {
        app.add_startup_system(init_main_menu.system())
            .init_resource::<ButtonMaterials>()
            .on_state_enter(RESPONSE_STAGE, Screen::MainMenu, init_main_menu.system())
            .on_state_exit(RESPONSE_STAGE, Screen::MainMenu, tear_down.system())
            .on_state_enter(RESPONSE_STAGE, Screen::Server, init_server_menu.system())
            .on_state_exit(RESPONSE_STAGE, Screen::Server, tear_down.system())
            .on_state_enter(RESPONSE_STAGE, Screen::Join, init_join_menu.system())
            .on_state_exit(RESPONSE_STAGE, Screen::Join, tear_down.system())
            .on_state_update(STATE_CHANGE_STAGE, Screen::MainMenu, button_system.system())
            .on_state_update(STATE_CHANGE_STAGE, Screen::Join, button_system.system())
            .on_state_update(STATE_CHANGE_STAGE, Screen::Server, button_system.system());
    }
}

enum ButtonActionType {
    HostGame,
    JoinGame,
    StartGame,
    GoBack,
    ConnectToServer,
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
    mut state: ResMut<State<Screen>>,
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
                        state.set_next(Screen::Server).unwrap();
                    }
                    ButtonActionType::JoinGame => {
                        state.set_next(Screen::Join).unwrap();
                    }
                    ButtonActionType::StartGame => {
                        state.set_next(Screen::Loading).unwrap();
                    }
                    ButtonActionType::GoBack => {
                        state.set_next(Screen::MainMenu).unwrap();
                    }
                    ButtonActionType::ConnectToServer => {
                        // Connect to server
                    }
                }
            }
            Interaction::Hovered => *material = button_materials.hovered.clone(),
            Interaction::None => *material = button_materials.normal.clone(),
        }
    }
}

fn init_main_menu(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    button_materials: Res<ButtonMaterials>,
) {
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

fn init_server_menu(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    button_materials: Res<ButtonMaterials>,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(50.0), Val::Percent(100.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            ..Default::default()
        })
        .with(ScreenEntity)
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    value: "Joined Users:".to_string(),
                    style: TextStyle {
                        font_size: 20.0,
                        color: Color::ANTIQUE_WHITE,
                        ..Default::default()
                    },
                },
                ..Default::default()
            });
        })
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(50.0), Val::Percent(100.0)),
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
                    action_type: ButtonActionType::StartGame,
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            value: "Start Game".to_string(),
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
                    action_type: ButtonActionType::GoBack,
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            value: "Back".to_string(),
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

fn init_join_menu(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    button_materials: Res<ButtonMaterials>,
) {
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
                    action_type: ButtonActionType::ConnectToServer,
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
                    action_type: ButtonActionType::GoBack,
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            value: "Back".to_string(),
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
