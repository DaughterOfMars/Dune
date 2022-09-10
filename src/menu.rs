use std::collections::HashSet;

use bevy::prelude::*;
use iyes_loopless::prelude::*;
use renet::RenetClient;

use crate::{
    game::state::{GameEvent, PlayerId},
    network::{connect_to_server, spawn_server, SendEvent, ServerEvent},
    tear_down, Screen,
};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonColors>()
            .add_enter_system(Screen::MainMenu, tear_down.chain(init_main_menu))
            .add_enter_system(Screen::Host, tear_down.chain(init_host_menu))
            .add_enter_system(Screen::Join, tear_down.chain(init_client_menu))
            .add_system(button.run_not_in_state(Screen::Game))
            .add_system_set(
                ConditionSet::new()
                    .run_not_in_state(Screen::Game)
                    .run_not_in_state(Screen::MainMenu)
                    .with_system(update_server_list)
                    .with_system(server_client_list)
                    .into(),
            )
            .add_system(start_game.run_if_resource_added::<StartGameMarker>());
    }
}

#[derive(Component)]
enum ButtonAction {
    HostGame,
    JoinGame,
    StartGame,
    GoBack,
}

struct ButtonColors {
    normal: UiColor,
    hovered: UiColor,
    pressed: UiColor,
}

impl Default for ButtonColors {
    fn default() -> Self {
        Self {
            normal: Color::rgb(0.15, 0.15, 0.15).into(),
            hovered: Color::rgb(0.25, 0.25, 0.25).into(),
            pressed: Color::rgb(0.35, 0.75, 0.35).into(),
        }
    }
}

fn button(
    mut commands: Commands,
    button_colors: Res<ButtonColors>,
    mut interactions: Query<(&Interaction, &mut UiColor, &ButtonAction), (Changed<Interaction>, With<Button>)>,
) {
    for (&interaction, mut color, action) in interactions.iter_mut() {
        match interaction {
            Interaction::Clicked => {
                *color = button_colors.pressed;
                match action {
                    ButtonAction::HostGame => {
                        spawn_server(&mut commands);
                        connect_to_server(&mut commands).unwrap();
                        commands.insert_resource(NextState(Screen::Host));
                    }
                    ButtonAction::JoinGame => {
                        connect_to_server(&mut commands).unwrap();
                        commands.insert_resource(NextState(Screen::Join));
                    }
                    ButtonAction::StartGame => {
                        commands.insert_resource(StartGameMarker);
                    }
                    ButtonAction::GoBack => {
                        commands.insert_resource(NextState(Screen::MainMenu));
                    }
                }
            }
            Interaction::Hovered => *color = button_colors.hovered,
            Interaction::None => *color = button_colors.normal,
        }
    }
}

struct StartGameMarker;

fn start_game(mut commands: Commands, mut client: ResMut<RenetClient>) {
    client.send_event(ServerEvent::LoadAssets);
    commands.remove_resource::<StartGameMarker>();
}

fn init_main_menu(mut commands: Commands, asset_server: Res<AssetServer>, button_colors: Res<ButtonColors>) {
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
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Percent(10.0), Val::Percent(6.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    color: button_colors.normal,
                    ..default()
                })
                .insert(ButtonAction::HostGame)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle::from_section(
                        "Host Game",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 20.0,
                            color: Color::ANTIQUE_WHITE,
                        },
                    ));
                });
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Percent(10.0), Val::Percent(6.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    color: button_colors.normal,
                    ..default()
                })
                .insert(ButtonAction::JoinGame)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle::from_section(
                        "Join Game",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 20.0,
                            color: Color::ANTIQUE_WHITE,
                        },
                    ));
                });
        });
}

#[derive(Default, Component)]
struct ServerList(HashSet<PlayerId>);

fn init_host_menu(mut commands: Commands, asset_server: Res<AssetServer>, button_colors: Res<ButtonColors>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(50.0), Val::Percent(100.0)),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle::from_section(
                    "Joined Users:",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 20.0,
                        color: Color::BLACK,
                    },
                ))
                .insert(ServerList::default());
        });
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(50.0), Val::Percent(100.0)),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Percent(10.0), Val::Percent(6.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    color: button_colors.normal,
                    ..default()
                })
                .insert(ButtonAction::StartGame)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle::from_section(
                        "Start Game",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 20.0,
                            color: Color::ANTIQUE_WHITE,
                        },
                    ));
                });
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Percent(10.0), Val::Percent(6.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    color: button_colors.normal,
                    ..default()
                })
                .insert(ButtonAction::GoBack)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle::from_section(
                        "Back",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 20.0,
                            color: Color::ANTIQUE_WHITE,
                        },
                    ));
                });
        });
}

fn init_client_menu(mut commands: Commands, asset_server: Res<AssetServer>, button_colors: Res<ButtonColors>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(50.0), Val::Percent(100.0)),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle::from_section(
                    "Joined Users:",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 20.0,
                        color: Color::BLACK,
                    },
                ))
                .insert(ServerList::default());
        });
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(50.0), Val::Percent(100.0)),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Waiting for Server...",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 20.0,
                    color: Color::BLACK,
                },
            ));
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Percent(10.0), Val::Percent(6.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    color: button_colors.normal,
                    ..default()
                })
                .insert(ButtonAction::GoBack)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle::from_section(
                        "Back",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 20.0,
                            color: Color::BLACK,
                        },
                    ));
                });
        });
}

fn update_server_list(mut game_events: EventReader<GameEvent>, mut list: Query<&mut ServerList>) {
    for event in game_events.iter() {
        match event {
            GameEvent::PlayerJoined { player_id } => {
                if let Ok(mut list) = list.get_single_mut() {
                    list.0.insert(*player_id);
                }
            }
            GameEvent::PlayerDisconnected { player_id } => {
                if let Ok(mut list) = list.get_single_mut() {
                    list.0.remove(player_id);
                }
            }
            _ => (),
        }
    }
}

fn server_client_list(mut list: Query<(&mut Text, &ServerList), Changed<ServerList>>) {
    if let Ok((mut list, ServerList(players))) = list.get_single_mut() {
        let mut s = "Joined Users:".to_string();
        // TODO: Fix this
        for player_id in players.iter() {
            s += "\n";
            s += player_id.0.to_string().as_str();
        }
        list.sections[0].value = s;
    }
}
