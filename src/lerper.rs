use std::f32::consts::PI;

use bevy::{prelude::*, render::camera::Camera};

use crate::{data::CameraNode, util::screen_to_world};

#[derive(Copy, Clone)]
pub enum LerpType {
    UI {
        src: Option<Vec2>,
        dest: Vec2,
    },
    World {
        src: Option<Transform>,
        dest: Transform,
    },
    UIToWorld {
        src: Option<Vec2>,
        dest: Transform,
    },
    WorldToUI {
        src: Option<Transform>,
        dest: Vec2,
    },
    Camera {
        src: Option<Transform>,
        dest: CameraNode,
    },
}

#[derive(Copy, Clone)]
pub struct Lerp {
    lerp_type: LerpType,
    pub time: f32,
    animation_time: f32,
    delay: f32,
}

impl Lerp {
    pub fn new(lerp_type: LerpType, time: f32, delay: f32) -> Self {
        Lerp {
            lerp_type,
            time,
            animation_time: time,
            delay,
        }
    }

    pub fn move_camera(dest: CameraNode, time: f32) -> Self {
        Lerp {
            lerp_type: LerpType::Camera { src: None, dest },
            time,
            animation_time: time,
            delay: 0.0,
        }
    }
}

struct UITransform {
    pos: Vec2,
}

pub struct LerpPlugin;

impl Plugin for LerpPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(camera_system.system())
            .add_system(lerp_system.system());
    }
}

fn lerp_system(
    commands: &mut Commands,
    time: Res<Time>,
    cameras: Query<(&Transform, &Camera)>,
    mut world_lerps: Query<
        (Entity, &mut Lerp, &mut Transform),
        (Without<UITransform>, Without<Camera>),
    >,
    mut ui_lerps: Query<(Entity, &mut Lerp, &mut Transform, &mut UITransform), Without<Camera>>,
) {
    for (entity, mut lerp, mut transform) in world_lerps.iter_mut() {
        if lerp.delay > 0.0 {
            lerp.delay -= time.delta_seconds();
        } else {
            match lerp.lerp_type {
                LerpType::UI { src, dest } => {
                    if let Some((cam_transform, _)) = cameras.iter().next() {
                        transform.rotation *= cam_transform.rotation;
                    }
                    if let Some(src) = src {
                        commands.insert_one(entity, UITransform { pos: src });
                    } else {
                        lerp.lerp_type = LerpType::WorldToUI { src: None, dest };
                    }
                }
                LerpType::WorldToUI { mut src, dest } => {
                    if let Some((cam_transform, camera)) = cameras.iter().next() {
                        if src.is_none() {
                            src.replace(transform.clone());
                        }
                        if lerp.time <= 0.0 {
                            transform.translation = screen_to_world(
                                dest.extend(0.1),
                                *cam_transform,
                                camera.projection_matrix,
                            );
                            transform.rotation = cam_transform.rotation * src.unwrap().rotation;

                            commands.insert_one(entity, UITransform { pos: dest });
                            commands.remove_one::<Lerp>(entity);
                        } else {
                            let mut lerp_amount =
                                PI * (lerp.animation_time - lerp.time) / lerp.animation_time;
                            lerp_amount = -0.5 * lerp_amount.cos() + 0.5;
                            transform.translation = src.unwrap().translation.lerp(
                                screen_to_world(
                                    dest.extend(0.1),
                                    *cam_transform,
                                    camera.projection_matrix,
                                ),
                                lerp_amount,
                            );
                            transform.rotation = src
                                .unwrap()
                                .rotation
                                .lerp(cam_transform.rotation * src.unwrap().rotation, lerp_amount);

                            lerp.time -= time.delta_seconds();
                        }
                    }
                }
                LerpType::UIToWorld { src, dest } => {
                    if let Some(src) = src {
                        commands.insert_one(entity, UITransform { pos: src });
                    } else {
                        lerp.lerp_type = LerpType::World { src: None, dest };
                    }
                }
                LerpType::World { mut src, dest } => {
                    if src.is_none() {
                        src.replace(transform.clone());
                    }
                    if lerp.time <= 0.0 {
                        *transform = dest;

                        commands.remove_one::<Lerp>(entity);
                    } else {
                        let mut lerp_amount =
                            PI * (lerp.animation_time - lerp.time) / lerp.animation_time;
                        lerp_amount = -0.5 * lerp_amount.cos() + 0.5;
                        transform.translation =
                            src.unwrap().translation.lerp(dest.translation, lerp_amount);
                        transform.rotation = src.unwrap().rotation.lerp(dest.rotation, lerp_amount);

                        lerp.time -= time.delta_seconds();
                    }
                }
                _ => (),
            }
        }
    }
    for (entity, mut lerp, mut transform, mut ui_transform) in ui_lerps.iter_mut() {
        if lerp.delay > 0.0 {
            lerp.delay -= time.delta_seconds();
        } else {
            match lerp.lerp_type {
                LerpType::UI { mut src, dest } => {
                    if let Some((cam_transform, camera)) = cameras.iter().next() {
                        if src.is_none() {
                            src.replace(ui_transform.pos);
                        }
                        if lerp.time <= 0.0 {
                            transform.translation = screen_to_world(
                                dest.extend(0.1),
                                *cam_transform,
                                camera.projection_matrix,
                            );

                            ui_transform.pos = dest;
                            commands.remove_one::<Lerp>(entity);
                        } else {
                            let mut lerp_amount =
                                PI * (lerp.animation_time - lerp.time) / lerp.animation_time;
                            lerp_amount = -0.5 * lerp_amount.cos() + 0.5;
                            transform.translation = screen_to_world(
                                src.unwrap().extend(0.1),
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

                            lerp.time -= time.delta_seconds();
                        }
                    }
                }
                LerpType::UIToWorld { mut src, dest } => {
                    if let Some((cam_transform, camera)) = cameras.iter().next() {
                        if src.is_none() {
                            src.replace(ui_transform.pos);
                        }
                        if lerp.time <= 0.0 {
                            *transform = dest;

                            commands.remove_one::<UITransform>(entity);
                            commands.remove_one::<Lerp>(entity);
                        } else {
                            let src_transform = screen_to_world(
                                src.unwrap().extend(0.1),
                                *cam_transform,
                                camera.projection_matrix,
                            );
                            let mut lerp_amount =
                                PI * (lerp.animation_time - lerp.time) / lerp.animation_time;
                            lerp_amount = -0.5 * lerp_amount.cos() + 0.5;
                            transform.translation =
                                src_transform.lerp(dest.translation, lerp_amount);
                            transform.rotation =
                                cam_transform.rotation.lerp(dest.rotation, lerp_amount);

                            lerp.time -= time.delta_seconds();
                        }
                    }
                }
                LerpType::World { src, dest } => {
                    if let Some((cam_transform, _)) = cameras.iter().next() {
                        transform.rotation *= -cam_transform.rotation;
                    }
                    if let Some(_) = src {
                        commands.remove_one::<UITransform>(entity);
                    } else {
                        lerp.lerp_type = LerpType::UIToWorld {
                            src: Some(ui_transform.pos),
                            dest,
                        };
                    }
                }
                LerpType::WorldToUI { src, dest } => {
                    if let Some(_) = src {
                        commands.remove_one::<UITransform>(entity);
                    } else {
                        lerp.lerp_type = LerpType::UI {
                            src: Some(ui_transform.pos),
                            dest,
                        };
                    }
                }
                LerpType::Camera { .. } => {
                    commands.remove_one::<UITransform>(entity);
                }
            }
        }
    }
}

fn camera_system(
    commands: &mut Commands,
    time: Res<Time>,
    mut cameras: Query<(Entity, &mut Lerp, &mut Transform), With<Camera>>,
) {
    for (entity, mut lerp, mut transform) in cameras.iter_mut() {
        if let LerpType::Camera { mut src, dest } = lerp.lerp_type {
            if src.is_none() {
                src.replace(transform.clone());
            }
            if lerp.time <= 0.0 {
                *transform = Transform::from_translation(dest.pos).looking_at(dest.at, dest.up);

                commands.remove_one::<Lerp>(entity);
            } else {
                let dest_transform =
                    Transform::from_translation(dest.pos).looking_at(dest.at, dest.up);
                let mut lerp_amount = PI * (lerp.animation_time - lerp.time) / lerp.animation_time;
                lerp_amount = -0.5 * lerp_amount.cos() + 0.5;
                transform.translation = src
                    .unwrap()
                    .translation
                    .lerp(dest_transform.translation, lerp_amount);
                transform.rotation = src
                    .unwrap()
                    .rotation
                    .lerp(dest_transform.rotation, lerp_amount);

                lerp.time -= time.delta_seconds();
            }
        } else {
            commands.remove_one::<Lerp>(entity);
        }
    }
}
