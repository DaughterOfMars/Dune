use std::f32::consts::PI;

use bevy::{prelude::*, render::camera::Camera};

use crate::{data::CameraNode, util::screen_to_world};

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

pub struct Lerp {
    pub lerp_type: LerpType,
    pub time: f32,
    pub delay: f32,
}

struct UITransform {
    pos: Vec2,
}

pub struct LerpPlugin;

impl Plugin for LerpPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(lerp_system.system());
    }
}

fn lerp_system(
    commands: &mut Commands,
    time: Res<Time>,
    cameras: Query<(&Transform, &Camera)>,
    mut world_lerps: Query<(Entity, &mut Lerp, &mut Transform), Without<UITransform>>,
    mut ui_lerps: Query<(Entity, &mut Lerp, &mut Transform, &UITransform)>,
) {
    for (entity, mut lerp, mut transform) in world_lerps.iter_mut() {
        if lerp.delay >= 0.0 {
            lerp.delay -= time.delta_seconds();
        } else {
            match lerp.lerp_type {
                LerpType::UI { src, dest } => {
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

                            commands.remove_one::<Lerp>(entity);
                        } else {
                            let src_transform = src.unwrap();
                            let dest_transform = screen_to_world(
                                dest.extend(0.1),
                                *cam_transform,
                                camera.projection_matrix,
                            );
                            let total_dist = src_transform.translation.distance(dest_transform);
                            let curr_dist = transform.translation.distance(dest_transform);
                            let mut lerp_amount = PI * (total_dist - curr_dist) / total_dist;
                            lerp_amount = -0.5 * lerp_amount.cos() + 0.5;
                            transform.translation =
                                src_transform.translation.lerp(dest_transform, lerp_amount);
                            transform.rotation = src_transform
                                .rotation
                                .lerp(cam_transform.rotation * src_transform.rotation, lerp_amount);

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
                        let src_transform = src.unwrap();
                        let dest_transform = dest;
                        let total_dist = src_transform
                            .translation
                            .distance(dest_transform.translation);
                        let curr_dist = transform.translation.distance(dest_transform.translation);
                        let mut lerp_amount = PI * (total_dist - curr_dist) / total_dist;
                        lerp_amount = -0.5 * lerp_amount.cos() + 0.5;
                        transform.translation = src_transform
                            .translation
                            .lerp(dest_transform.translation, lerp_amount);
                        transform.rotation = src_transform
                            .rotation
                            .lerp(dest_transform.rotation, lerp_amount);

                        lerp.time -= time.delta_seconds();
                    }
                }
                LerpType::Camera { mut src, dest } => {
                    if src.is_none() {
                        src.replace(transform.clone());
                    }
                    if lerp.time <= 0.0 {
                        *transform =
                            Transform::from_translation(dest.pos).looking_at(dest.at, dest.up);

                        commands.remove_one::<Lerp>(entity);
                    } else {
                        let src_transform = src.unwrap();
                        let dest_transform =
                            Transform::from_translation(dest.pos).looking_at(dest.at, dest.up);
                        let total_dist = src_transform
                            .translation
                            .distance(dest_transform.translation);
                        let curr_dist = transform.translation.distance(dest_transform.translation);
                        let mut lerp_amount = PI * (total_dist - curr_dist) / total_dist;
                        lerp_amount = -0.5 * lerp_amount.cos() + 0.5;
                        transform.translation = src_transform
                            .translation
                            .lerp(dest_transform.translation, lerp_amount);
                        transform.rotation = src_transform
                            .rotation
                            .lerp(dest_transform.rotation, lerp_amount);

                        lerp.time -= time.delta_seconds();
                    }
                }
            }
        }
    }
    for (entity, mut lerp, mut transform, ui_transform) in ui_lerps.iter_mut() {
        if lerp.delay >= 0.0 {
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

                            commands.remove_one::<UITransform>(entity);
                            commands.remove_one::<Lerp>(entity);
                        } else {
                            let src_transform = screen_to_world(
                                src.unwrap().extend(0.1),
                                *cam_transform,
                                camera.projection_matrix,
                            );
                            let dest_transform = screen_to_world(
                                dest.extend(0.1),
                                *cam_transform,
                                camera.projection_matrix,
                            );
                            let total_dist = src_transform.distance(dest_transform);
                            let curr_dist = transform.translation.distance(dest_transform);
                            let mut lerp_amount = PI * (total_dist - curr_dist) / total_dist;
                            lerp_amount = -0.5 * lerp_amount.cos() + 0.5;
                            transform.translation = src_transform.lerp(dest_transform, lerp_amount);

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
                            let dest_transform = dest;
                            let total_dist = src_transform.distance(dest_transform.translation);
                            let curr_dist =
                                transform.translation.distance(dest_transform.translation);
                            let mut lerp_amount = PI * (total_dist - curr_dist) / total_dist;
                            lerp_amount = -0.5 * lerp_amount.cos() + 0.5;
                            transform.translation =
                                src_transform.lerp(dest_transform.translation, lerp_amount);
                            transform.rotation = cam_transform
                                .rotation
                                .lerp(dest_transform.rotation, lerp_amount);

                            lerp.time -= time.delta_seconds();
                        }
                    }
                }
                LerpType::World { src, dest } => {
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
