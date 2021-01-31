use rand::Rng;

use crate::network::Network;

use super::*;

pub(crate) fn stack_troops(
    commands: &mut Commands,
    mut queue: ResMut<ActionQueue>,
    troops: Query<(Entity, &Unique, &Troop)>,
    locations: Query<(Entity, &LocationSector), With<Disorganized>>,
) {
    for (loc_entity, loc_sec) in locations.iter() {
        let mut map = HashMap::new();
        for (entity, faction) in troops.iter().filter_map(|(entity, unique, troop)| {
            troop.location.and_then(|location| {
                if location == loc_entity {
                    Some((entity, unique.faction))
                } else {
                    None
                }
            })
        }) {
            map.entry(faction).or_insert(Vec::new()).push(entity);
        }
        for (node_ind, troops) in map.values().enumerate() {
            let node = loc_sec.location.sectors[&loc_sec.sector].fighters[node_ind];
            queue.push_multiple_front(
                troops
                    .iter()
                    .enumerate()
                    .map(|(i, entity)| {
                        Action::add_lerp(
                            *entity,
                            Lerp::new(
                                LerpType::world_to(
                                    Transform::from_translation(Vec3::new(node.x, node.z, -node.y))
                                        * Transform::from_translation(
                                            i as f32 * 0.0018 * Vec3::unit_y(),
                                        ),
                                ),
                                0.1,
                                0.0,
                            ),
                        )
                        .into()
                    })
                    .collect::<Vec<_>>(),
            );
        }
        commands.remove_one::<Disorganized>(loc_entity);
    }
}

pub(crate) fn public_troop(mut troops: Query<(&Troop, &mut Unique)>) {
    for (troop, mut unique) in troops.iter_mut() {
        unique.public = troop.location.is_some();
    }
}

/*
pub(crate) fn active_player(
    info: Res<Info>,
    players: Query<&Player>,
    mut uniques: Query<(&mut Visible, &Unique)>,
) {
    let entity = info.get_active_player();
    let active_player_faction = players.get(entity).unwrap().faction;
    for (mut visible, unique) in uniques.iter_mut() {
        if visible.is_visible != (unique.public || unique.faction == active_player_faction) {
            visible.is_visible = unique.public || unique.faction == active_player_faction;
        }
    }
}
*/

pub(crate) fn phase_text(
    phase: Res<GamePhase>,
    info: Res<Info>,
    players: Query<&Player>,
    mut text: Query<&mut Text, With<PhaseText>>,
) {
    let s = match phase.curr {
        Phase::Setup(subphase) => match subphase {
            SetupSubPhase::ChooseFactions => "Choosing Factions...".to_string(),
            SetupSubPhase::Prediction => "Bene Gesserit are making a prediction...".to_string(),
            SetupSubPhase::AtStart => format!(
                "{:?} Initial Placement...",
                players.get(info.get_active_player()).unwrap().faction
            ),
            SetupSubPhase::DealTraitors => "Dealing Traitor Cards...".to_string(),
            SetupSubPhase::PickTraitors => "Picking Traitors...".to_string(),
            SetupSubPhase::DealTreachery => "Dealing Treachery Cards...".to_string(),
        },
        Phase::Storm(_) => "Storm Phase".to_string(),
        Phase::SpiceBlow => "Spice Blow Phase".to_string(),
        Phase::Nexus => "Nexus Phase".to_string(),
        Phase::Bidding => "Bidding Phase".to_string(),
        Phase::Revival => "Revival Phase".to_string(),
        Phase::Movement => "Movement Phase".to_string(),
        Phase::Battle => "Battle Phase".to_string(),
        Phase::Collection => "Collection Phase".to_string(),
        Phase::Control => "Control Phase".to_string(),
        Phase::EndGame => "".to_string(),
    };

    if let Some(mut text) = text.iter_mut().next() {
        text.value = s;
    }
}

pub(crate) fn init_factions(
    commands: &mut Commands,
    data: Res<Data>,
    mut info: ResMut<Info>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut colors: ResMut<Assets<ColorMaterial>>,
    players: Query<&Player>,
) {
    println!("Enter: init_factions");
    let card_face = asset_server.get_handle("card.gltf#Mesh0/Primitive0");
    let card_back = asset_server.get_handle("card.gltf#Mesh0/Primitive1");

    let shield_face = asset_server.get_handle("shield.gltf#Mesh0/Primitive1");
    let shield_back = asset_server.get_handle("shield.gltf#Mesh0/Primitive2");
    let shield_shape = ShapeHandle::new(Cuboid::new(Vector3::new(0.525, 0.285, 0.06)));

    let prediction_back_texture = asset_server.get_handle("treachery/treachery_back.png");
    let prediction_back_material = materials.add(StandardMaterial {
        albedo_texture: Some(prediction_back_texture),
        ..Default::default()
    });

    let faction_prediction_shape =
        ShapeHandle::new(Cuboid::new(Vector3::new(0.125, 0.0005, 0.18) * 0.01));
    let turn_prediction_shape =
        ShapeHandle::new(Cuboid::new(Vector3::new(0.125, 0.0005, 0.18) * 0.006));

    let little_token = asset_server.get_handle("little_token.gltf#Mesh0/Primitive0");
    let big_token = asset_server.get_handle("big_token.gltf#Mesh0/Primitive0");
    let spice_token = asset_server.get_handle("spice_token.gltf#Mesh0/Primitive0");

    let little_token_shape = ShapeHandle::new(
        ConvexHull::try_from_points(&Cylinder::<f32>::new(0.0018, 0.03).to_trimesh(32).coords)
            .unwrap(),
    );
    let big_token_shape = ShapeHandle::new(
        ConvexHull::try_from_points(&Cylinder::<f32>::new(0.0035, 0.06).to_trimesh(32).coords)
            .unwrap(),
    );
    let spice_token_shape = ShapeHandle::new(
        ConvexHull::try_from_points(&Cylinder::<f32>::new(0.0018, 0.017).to_trimesh(32).coords)
            .unwrap(),
    );

    let turn_tiles = data.ui_structure.get_turn_tiles();

    for (i, player) in players.iter().enumerate() {
        let faction = player.faction;
        let faction_code = match faction {
            Faction::Atreides => "at",
            Faction::Harkonnen => "hk",
            Faction::Emperor => "em",
            Faction::SpacingGuild => "sg",
            Faction::Fremen => "fr",
            Faction::BeneGesserit => "bg",
        };

        let logo_texture =
            asset_server.get_handle(format!("tokens/{}_logo.png", faction_code).as_str());

        commands
            .spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    position: turn_tiles[i].top_left(),
                    size: turn_tiles[i].size(),
                    align_items: AlignItems::FlexStart,
                    padding: Rect {
                        top: Val::Percent(1.0),
                        bottom: Val::Percent(1.0),
                        left: Val::Percent(1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                material: colors.add(if i % 2 == 0 {
                    (Color::RED + Color::rgba_linear(0.0, 0.0, 0.0, -0.5)).into()
                } else {
                    (Color::GREEN + Color::rgba_linear(0.0, 0.0, 0.0, -0.5)).into()
                }),
                ..Default::default()
            })
            .with(ScreenEntity)
            .with_children(|parent| {
                parent
                    .spawn(ImageBundle {
                        style: Style {
                            size: Size::new(Val::Px(20.0), Val::Px(20.0)),
                            ..Default::default()
                        },
                        material: colors.add(logo_texture.into()),
                        ..Default::default()
                    })
                    .spawn(TextBundle {
                        text: Text {
                            font: asset_server.get_handle("fonts/FiraSans-Bold.ttf"),
                            value: faction.to_string(),
                            style: TextStyle {
                                font_size: 20.0,
                                color: Color::ANTIQUE_WHITE,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    });
            });

        let shield_front_texture =
            asset_server.get_handle(format!("shields/{}_shield_front.png", faction_code).as_str());
        let shield_back_texture =
            asset_server.get_handle(format!("shields/{}_shield_back.png", faction_code).as_str());
        let shield_front_material = materials.add(StandardMaterial {
            albedo_texture: Some(shield_front_texture),
            ..Default::default()
        });
        let shield_back_material = materials.add(StandardMaterial {
            albedo_texture: Some(shield_back_texture),
            ..Default::default()
        });
        commands
            .spawn(
                ColliderBundle::new(shield_shape.clone())
                    .with_transform(Transform::from_translation(Vec3::new(0.0, 0.27, 1.34))),
            )
            .with(ScreenEntity)
            .with(data.camera_nodes.shield)
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
            .spawn(ColliderBundle::new(faction_prediction_shape.clone()))
            .with(ScreenEntity)
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

        for (i, leader) in data
            .leaders
            .iter()
            .filter(|l| l.faction == faction)
            .enumerate()
        {
            let texture =
                asset_server.get_handle(format!("leaders/{}.png", leader.texture).as_str());
            let material = materials.add(StandardMaterial {
                albedo_texture: Some(texture),
                ..Default::default()
            });

            commands
                .spawn(
                    ColliderBundle::new(big_token_shape.clone())
                        .with_transform(Transform::from_translation(data.token_nodes.leaders[i])),
                )
                .with(ScreenEntity)
                .with_bundle(UniqueBundle::new(faction))
                .with_children(|parent| {
                    parent.spawn(PbrBundle {
                        mesh: big_token.clone(),
                        material,
                        ..Default::default()
                    });
                });
        }

        let troop_texture =
            asset_server.get_handle(format!("tokens/{}_troop.png", faction_code).as_str());
        let troop_material = materials.add(StandardMaterial {
            albedo_texture: Some(troop_texture),
            ..Default::default()
        });

        for i in 0..20 {
            commands
                .spawn(
                    ColliderBundle::new(little_token_shape.clone()).with_transform(
                        Transform::from_translation(
                            data.token_nodes.fighters[0] + (i as f32 * 0.0036 * Vec3::unit_y()),
                        ),
                    ),
                )
                .with(ScreenEntity)
                .with_bundle(UniqueBundle::new(faction))
                .with(Troop {
                    value: 1,
                    location: None,
                })
                .with_children(|parent| {
                    parent.spawn(PbrBundle {
                        mesh: little_token.clone(),
                        material: troop_material.clone(),
                        ..Default::default()
                    });
                });
        }

        let spice_1_texture = asset_server.get_handle("tokens/spice_1.png");
        let spice_1_material = materials.add(StandardMaterial {
            albedo_texture: Some(spice_1_texture),
            ..Default::default()
        });
        let spice_2_texture = asset_server.get_handle("tokens/spice_2.png");
        let spice_2_material = materials.add(StandardMaterial {
            albedo_texture: Some(spice_2_texture),
            ..Default::default()
        });
        let spice_5_texture = asset_server.get_handle("tokens/spice_5.png");
        let spice_5_material = materials.add(StandardMaterial {
            albedo_texture: Some(spice_5_texture),
            ..Default::default()
        });
        let spice_10_texture = asset_server.get_handle("tokens/spice_10.png");
        let spice_10_material = materials.add(StandardMaterial {
            albedo_texture: Some(spice_10_texture),
            ..Default::default()
        });

        let (_, _, spice) = faction.initial_values();

        let (tens, fives, twos, ones) = divide_spice(spice);
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
                .spawn(
                    ColliderBundle::new(spice_token_shape.clone()).with_transform(
                        Transform::from_translation(
                            data.token_nodes.spice[s] + (i as f32 * 0.0036 * Vec3::unit_y()),
                        ),
                    ),
                )
                .with(ScreenEntity)
                .with_bundle(UniqueBundle::new(faction))
                .with(Spice { value })
                .with_children(|parent| {
                    parent.spawn(PbrBundle {
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
        let prediction_front_material = materials.add(StandardMaterial {
            albedo_texture: Some(prediction_front_texture),
            ..Default::default()
        });
        commands
            .spawn(ColliderBundle::new(turn_prediction_shape.clone()))
            .with(ScreenEntity)
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
}

pub(crate) fn shuffle_decks(
    mut treachery_cards: Query<(Entity, &mut Transform, &TreacheryCard)>,
    mut traitor_cards: Query<(Entity, &mut Transform, &TraitorCard)>,
) {
    println!("Enter: shuffle_decks");
    let mut rng = rand::thread_rng();
    shuffle_deck(
        &mut rng,
        0.001,
        &mut treachery_cards
            .iter_mut()
            .map(|(entity, transform, _)| (entity, transform))
            .collect(),
    );
    shuffle_deck(
        &mut rng,
        0.001,
        &mut traitor_cards
            .iter_mut()
            .map(|(entity, transform, _)| (entity, transform))
            .collect(),
    );
}

pub(crate) fn pick_factions(
    commands: &mut Commands,
    mut info: ResMut<Info>,
    data: Res<Data>,
    network: Res<Network>,
) {
    println!("Enter: pick_factions");
    // TODO: pick manually
    let factions = vec![
        Faction::Atreides,
        Faction::BeneGesserit,
        Faction::Emperor,
        Faction::Fremen,
        Faction::Harkonnen,
        Faction::SpacingGuild,
    ];

    let mut rng = rand::thread_rng();

    for i in 0..info.players.len() {
        let faction = factions[rng.gen_range(0..6)];
        commands
            .spawn((Player::new(faction, &data.leaders),))
            .with(ScreenEntity);

        if faction == Faction::BeneGesserit {
            commands.with(Prediction {
                faction: None,
                turn: None,
            });
        }

        let entity = commands.current_entity().unwrap();
        info.play_order.push(entity);
        info.factions_in_play.push(faction);
        if network.address == info.players[i].parse().ok() {
            info.me = Some(entity);
        }
    }
}

pub(crate) fn animate_prediction_cards(
    mut info: ResMut<Info>,
    data: Res<Data>,
    mut queue: ResMut<ActionQueue>,
    players: Query<&Player>,
    prediction_cards: QuerySet<(
        Query<(Entity, &FactionPredictionCard)>,
        Query<(Entity, &TurnPredictionCard)>,
    )>,
) {
    println!("Enter: animate_prediction_cards");
    if let Some(me) = info.me {
        if players.get(me).unwrap().faction == Faction::BeneGesserit {
            info.active_player = info.me;
            // Lerp in faction cards
            let num_factions = info.factions_in_play.len();
            let animation_time = 1.5;
            let delay = animation_time / (2.0 * num_factions as f32);
            let indiv_anim_time = animation_time - (delay * (num_factions - 1) as f32);

            let actions = prediction_cards
                .q0()
                .iter()
                .enumerate()
                .map(|(i, (element, _))| {
                    Action::add_lerp(
                        element,
                        Lerp::new(
                            LerpType::ui_from_to(
                                (data.prediction_nodes.src, Quat::from_rotation_x(0.5 * PI)).into(),
                                (
                                    data.prediction_nodes.factions[i],
                                    Quat::from_rotation_x(0.5 * PI),
                                )
                                    .into(),
                            ),
                            indiv_anim_time,
                            delay * i as f32,
                        ),
                    )
                    .into()
                })
                .collect::<Vec<_>>();
            queue.push_multiple(actions);
            let clickables = prediction_cards
                .q0()
                .iter()
                .map(|(element, _)| element)
                .collect();
            queue.push_single(Action::Enable { clickables }.into());
            queue.push_single(Action::ContextChange(Context::Predicting).into());

            // Lerp in Turn Cards
            let animation_time = 1.5;
            let delay = animation_time / 30.0;
            let indiv_anim_time = animation_time - (delay * 14.0);

            let actions = prediction_cards
                .q1()
                .iter()
                .enumerate()
                .map(|(i, (element, _))| {
                    Action::add_lerp(
                        element,
                        Lerp::new(
                            LerpType::ui_from_to(
                                (
                                    data.prediction_nodes.src,
                                    Quat::from_rotation_x(0.5 * PI),
                                    0.6,
                                )
                                    .into(),
                                (
                                    data.prediction_nodes.turns[i],
                                    Quat::from_rotation_x(0.5 * PI),
                                    0.6,
                                )
                                    .into(),
                            ),
                            indiv_anim_time,
                            delay * i as f32,
                        ),
                    )
                    .into()
                })
                .collect::<Vec<_>>();
            queue.push_multiple(actions);
            let clickables = prediction_cards
                .q1()
                .iter()
                .map(|(element, _)| element)
                .collect();
            queue.push_single(Action::Enable { clickables }.into());
            queue.push_single(Action::ContextChange(Context::Predicting).into());
            queue.push_single(Action::PassTurn.into());
            queue.push_single(Action::AdvancePhase.into());
        }
    }
}

pub(crate) fn get_initial_spice() {
    todo!();
}

pub(crate) fn place_troops(
    info: Res<Info>,
    players: Query<(Entity, &Player)>,
    mut queue: ResMut<ActionQueue>,
    data: Res<Data>,
    clickable_locations: Query<(Entity, &LocationSector)>,
    mut troops: Query<(Entity, &mut Troop, &Unique, &Transform)>,
    cameras: Query<Entity, (With<Camera>, Without<OrthographicProjection>)>,
) {
    println!("Enter: place_troops");
    let clickables = clickable_locations
        .iter()
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();

    let mut actions_map = players
        .iter()
        .map(|(entity, player)| {
            let (num_troops, locations, _) = player.faction.initial_values();
            (
                entity,
                // Check if we even have free troops to place
                if num_troops > 0 {
                    if let Some(locations) = locations {
                        let mut res = vec![Action::SetActivePlayer { player: entity }];
                        if locations.len() == 0 {
                            // Do nothing
                        } else if locations.len() == 1 {
                            let (location, loc_sec) = clickable_locations
                                .iter()
                                .find(|(_, loc_sec)| loc_sec.location.name == locations[0])
                                .unwrap();
                            let mut troop_stack = troops
                                .iter_mut()
                                .filter(|(_, troop, unique, _)| {
                                    unique.faction == player.faction && troop.location.is_none()
                                })
                                .collect::<Vec<_>>();
                            troop_stack.sort_by(|(_, _, _, transform1), (_, _, _, transform2)| {
                                transform1
                                    .translation
                                    .y
                                    .partial_cmp(&transform2.translation.y)
                                    .unwrap()
                            });
                            res.extend((0..num_troops).map(|i| {
                                if let Some((entity, troop, _, _)) = troop_stack.get_mut(i as usize)
                                {
                                    troop.location = Some(location);
                                    let node =
                                        loc_sec.location.sectors[&loc_sec.sector].fighters[0];
                                    Action::add_lerp(
                                        *entity,
                                        Lerp::new(
                                            LerpType::world_to(
                                                Transform::from_translation(Vec3::new(
                                                    node.x, node.z, -node.y,
                                                )) * Transform::from_translation(
                                                    i as f32 * 0.0018 * Vec3::unit_y(),
                                                ),
                                            ),
                                            0.1,
                                            0.0,
                                        ),
                                    )
                                } else {
                                    panic!();
                                }
                            }));
                        } else {
                            res.push(Action::ContextChange(Context::PlacingTroops));
                        };
                        res
                    } else {
                        vec![
                            Action::SetActivePlayer { player: entity },
                            Action::ContextChange(Context::PlacingTroops),
                        ]
                    }
                } else {
                    vec![]
                },
            )
        })
        .collect::<HashMap<_, _>>();

    let faction_order = info
        .play_order
        .iter()
        .map(|&entity| (entity, players.get(entity).unwrap().1.faction))
        .enumerate()
        .collect::<Vec<_>>();

    let (bg_pos, fr_pos) = (
        faction_order
            .iter()
            .find(|(_, (_, faction))| *faction == Faction::BeneGesserit)
            .unwrap()
            .0,
        faction_order
            .iter()
            .find(|(_, (_, faction))| *faction == Faction::Fremen)
            .unwrap()
            .0,
    );

    // Move the camera so we can see the board good
    queue.push_single(
        Action::add_lerp(
            cameras.iter().next().unwrap(),
            Lerp::move_camera(data.camera_nodes.board, 1.0),
        )
        .into(),
    );
    queue.push_single(Action::Enable { clickables }.into());
    queue.extend(if bg_pos < fr_pos {
        faction_order[..bg_pos]
            .iter()
            .chain(std::iter::once(&faction_order[fr_pos]))
            .chain(std::iter::once(&faction_order[bg_pos]))
            .chain(faction_order[bg_pos + 1..fr_pos].iter())
            .chain(faction_order[fr_pos + 1..].iter())
            .map(|(_, (entity, _))| actions_map.remove(entity).unwrap())
            .flatten()
            .map(|action| action.into())
            .collect::<Vec<_>>()
    } else {
        faction_order
            .iter()
            .map(|(_, (entity, _))| actions_map.remove(&entity).unwrap())
            .flatten()
            .map(|action| action.into())
            .collect::<Vec<_>>()
    });
    queue.push_single(
        Action::add_lerp(
            cameras.iter().next().unwrap(),
            Lerp::move_camera(data.camera_nodes.main, 1.0),
        )
        .into(),
    );
    queue.push_single(Action::AdvancePhase.into());
}

pub(crate) fn deal_traitor_cards(
    info: Res<Info>,
    data: Res<Data>,
    mut queue: ResMut<ActionQueue>,
    mut players: Query<&mut Player>,
    mut traitor_cards: Query<(Entity, &mut Transform, &TraitorCard)>,
) {
    println!("Enter: deal_traitor_cards");
    let mut cards = traitor_cards
        .iter_mut()
        .map(|(entity, transform, _)| (entity, transform))
        .collect::<Vec<_>>();
    cards.sort_by(|(_, transform1), (_, transform2)| {
        transform1
            .translation
            .y
            .partial_cmp(&transform2.translation.y)
            .unwrap()
    });
    let mut actions = Vec::new();
    let positions = hand_positions(4);
    let turn_tile_pts = data
        .ui_structure
        .get_turn_tiles()
        .iter()
        .map(|tile| tile.center())
        .collect::<Vec<_>>();

    let mut delay = 0.0;
    for i in 0..4 {
        for (j, &entity) in info.play_order.iter().enumerate() {
            if let Ok(mut player) = players.get_mut(entity) {
                let card = cards.pop().unwrap().0;
                player.traitor_cards.push(card);
                if entity == info.get_active_player() {
                    actions.push(
                        Action::add_lerp(
                            card,
                            Lerp::new(LerpType::card_to_ui(positions[i], 1.0), 0.6, delay),
                        )
                        .then(
                            Action::Assign {
                                element: card,
                                faction: player.faction,
                            }
                            .into(),
                        ),
                    );
                } else {
                    actions.push(
                        Action::add_lerp(
                            card,
                            Lerp::new(
                                LerpType::world_to_ui(
                                    (
                                        turn_tile_pts[j],
                                        Quat::from_rotation_x(0.5 * PI) * Quat::from_rotation_z(PI),
                                        0.4,
                                    )
                                        .into(),
                                ),
                                0.6,
                                delay,
                            ),
                        )
                        .then(
                            Action::Assign {
                                element: card,
                                faction: player.faction,
                            }
                            .into(),
                        ),
                    );
                }
            }
            delay += 0.2;
        }
    }
    queue.push_multiple(actions);
}

pub(crate) fn storm_reveal(info: Res<Info>, mut phase: ResMut<GamePhase>) {
    println!("Enter: storm_reveal");
    // Make card visible to everyone
    if info.turn == 0 {
        phase.push_next(Phase::Storm(StormSubPhase::MoveStorm))
    } else {
        phase.push_next(Phase::Storm(StormSubPhase::WeatherControl))
    }
}

pub(crate) fn storm_weather_control(
    mut queue: ResMut<ActionQueue>,
    mut treachery_cards: Query<(Entity, &TreacheryCard)>,
) {
    println!("Enter: storm_weather_control");
    if let Some((entity, _)) = treachery_cards
        .iter_mut()
        .find(|(_, card)| card.name == "Weather Control")
    {
        // TODO: Add weather control card as clickable
        todo!();
        queue.push_single(Action::Enable { clickables: vec![] }.into());
        queue.push_single(Action::ContextChange(Context::Prompting).into());
        queue.push_single(Action::PassTurn.into());
    }
}

pub(crate) fn storm_family_atomics(
    mut queue: ResMut<ActionQueue>,
    mut treachery_cards: Query<(Entity, &TreacheryCard)>,
) {
    println!("Enter: storm_family_atomics");
    if let Some((entity, _)) = treachery_cards
        .iter_mut()
        .find(|(_, card)| card.name == "Family Atomics")
    {
        // TODO: Add family atomics as clickable
        queue.push_single(Action::Enable { clickables: vec![] }.into());
        queue.push_single(Action::ContextChange(Context::Prompting).into());
        queue.push_single(Action::PassTurn.into());
    }
}

pub(crate) fn storm_move() {
    /*
    let mut rng = rand::thread_rng();
    if info.turn == 0 {
        for mut storm in storm_query.iter_mut() {
            storm.sector = rng.gen_range(0..18);
        }
    } else {
        let &storm_card = collections.storm_deck.last().unwrap();
        let delta = storm_cards.get(storm_card).unwrap().val;
        for mut storm in storm_query.iter_mut() {
            storm.sector += delta;
            storm.sector %= 18;
        }
        // TODO: Kill everything it passed over and wipe spice
        collections.storm_deck.shuffle(&mut rng)
        // TODO: Choose a first player
        // TODO: Assign bonuses
    }
    */
}
