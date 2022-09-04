use std::f32::consts::PI;

use bevy::{
    prelude::*,
    render::camera::{Camera, OrthographicProjection},
};

use crate::{data::CameraNode, util::screen_to_world};

const UI_SCALE: f32 = 0.01;
const UI_Z: f32 = 0.1;
const SPEED_MOD: f32 = 1.0;

#[derive(Copy, Clone)]
pub enum LerpType {
    UI {
        src: Option<UITransform>,
        dest: UITransform,
    },
    World {
        src: Option<Transform>,
        dest: Transform,
    },
    UIToWorld {
        src: Option<UITransform>,
        dest: Transform,
    },
    WorldToUI {
        src: Option<Transform>,
        dest: UITransform,
    },
    Camera {
        src: Option<Transform>,
        dest: CameraNode,
    },
}

impl LerpType {
    pub fn ui_to(dest: UITransform) -> Self {
        LerpType::UI { src: None, dest }
    }

    pub fn ui_from_to(src: UITransform, dest: UITransform) -> Self {
        LerpType::UI { src: Some(src), dest }
    }

    pub fn world_to(dest: Transform) -> Self {
        LerpType::World { src: None, dest }
    }

    pub fn world_from_to(src: Transform, dest: Transform) -> Self {
        LerpType::World { src: Some(src), dest }
    }

    pub fn world_to_ui(dest: UITransform) -> Self {
        LerpType::WorldToUI { src: None, dest }
    }

    pub fn card_to_ui(dest: Vec2, scale: f32) -> Self {
        LerpType::WorldToUI {
            src: None,
            dest: (dest, Quat::from_rotation_x(0.5 * PI), scale).into(),
        }
    }

    pub fn ui_to_world(dest: Transform) -> Self {
        LerpType::UIToWorld { src: None, dest }
    }
}

#[derive(Copy, Clone, Component)]
pub struct Lerp {
    lerp_type: LerpType,
    src: Option<Transform>,
    dest: Option<Transform>,
    pub time: f32,
    animation_time: f32,
    delay: f32,
}

impl Lerp {
    pub fn new(lerp_type: LerpType, time: f32, delay: f32) -> Self {
        Lerp {
            lerp_type,
            src: None,
            dest: None,
            time,
            animation_time: time,
            delay,
        }
    }

    pub fn move_camera(dest: CameraNode, time: f32) -> Self {
        Lerp {
            lerp_type: LerpType::Camera { src: None, dest },
            src: None,
            dest: None,
            time,
            animation_time: time,
            delay: 0.0,
        }
    }
}

#[derive(Default, Copy, Clone)]
pub struct UITransform {
    translation: Vec2,
    rotation: Quat,
    scale: f32,
}

impl UITransform {
    pub fn from_translation(translation: Vec2) -> Self {
        UITransform {
            translation,
            scale: 1.0,
            ..Default::default()
        }
    }

    pub fn from_rotation(rotation: Quat) -> Self {
        UITransform {
            rotation,
            scale: 1.0,
            ..Default::default()
        }
    }

    pub fn from_scale(scale: f32) -> Self {
        UITransform {
            scale,
            ..Default::default()
        }
    }

    pub fn with_translation(mut self, translation: Vec2) -> Self {
        self.translation = translation;
        self
    }

    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
}

impl From<Vec2> for UITransform {
    fn from(v: Vec2) -> Self {
        UITransform::from_translation(v)
    }
}

impl From<Quat> for UITransform {
    fn from(q: Quat) -> Self {
        UITransform::from_rotation(q)
    }
}

impl From<(Vec2, Quat)> for UITransform {
    fn from((v, q): (Vec2, Quat)) -> Self {
        UITransform::from_translation(v).with_rotation(q)
    }
}

impl From<(Vec2, Quat, f32)> for UITransform {
    fn from((v, q, s): (Vec2, Quat, f32)) -> Self {
        UITransform::from_translation(v).with_rotation(q).with_scale(s)
    }
}

pub struct LerpPlugin;

impl Plugin for LerpPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(camera_system).add_system(lerp_system);
    }
}

fn lerp_system(
    mut commands: Commands,
    time: Res<Time>,
    cameras: Query<(&Transform, &Camera), Without<OrthographicProjection>>,
    mut lerps: Query<(Entity, &mut Lerp, &mut Transform), Without<Camera>>,
) {
    for (entity, mut lerp, mut transform) in lerps.iter_mut() {
        if lerp.dest.is_none() {
            match lerp.lerp_type {
                LerpType::UI { src, dest } => {
                    if let Some((cam_transform, camera)) = cameras.iter().next() {
                        if let Some(src) = src {
                            lerp.src = Some(
                                Transform::from_translation(screen_to_world(
                                    src.translation.extend(UI_Z),
                                    *cam_transform,
                                    camera.projection_matrix(),
                                )) * Transform::from_rotation(cam_transform.rotation * src.rotation)
                                    * Transform::from_scale(Vec3::splat(UI_SCALE * src.scale)),
                            );
                        }
                        lerp.dest = Some(
                            Transform::from_translation(screen_to_world(
                                dest.translation.extend(UI_Z),
                                *cam_transform,
                                camera.projection_matrix(),
                            )) * Transform::from_rotation(cam_transform.rotation * dest.rotation)
                                * Transform::from_scale(Vec3::splat(UI_SCALE * dest.scale)),
                        );
                    }
                }
                LerpType::World { src, dest } => {
                    lerp.src = src;
                    lerp.dest = Some(dest);
                }
                LerpType::UIToWorld { src, dest } => {
                    if let Some((cam_transform, camera)) = cameras.iter().next() {
                        if let Some(src) = src {
                            lerp.src = Some(
                                Transform::from_translation(screen_to_world(
                                    src.translation.extend(UI_Z),
                                    *cam_transform,
                                    camera.projection_matrix(),
                                )) * Transform::from_rotation(cam_transform.rotation * src.rotation)
                                    * Transform::from_scale(Vec3::splat(UI_SCALE * src.scale)),
                            );
                        }
                    }
                    lerp.dest = Some(dest);
                }
                LerpType::WorldToUI { src, dest } => {
                    lerp.src = src;
                    if let Some((cam_transform, camera)) = cameras.iter().next() {
                        lerp.dest = Some(
                            Transform::from_translation(screen_to_world(
                                dest.translation.extend(UI_Z),
                                *cam_transform,
                                camera.projection_matrix(),
                            )) * Transform::from_rotation(cam_transform.rotation * dest.rotation)
                                * Transform::from_scale(Vec3::splat(UI_SCALE * dest.scale)),
                        );
                    }
                }
                _ => (),
            }
        }
        if let Some(dest) = lerp.dest {
            if lerp.delay > 0.0 {
                lerp.delay -= time.delta_seconds() * SPEED_MOD;
            } else {
                if lerp.src.is_none() {
                    lerp.src.replace(transform.clone());
                }
                if lerp.time <= 0.0 {
                    *transform = dest;

                    commands.entity(entity).remove::<Lerp>();
                } else {
                    let mut lerp_amount = (lerp.animation_time - lerp.time) / lerp.animation_time;
                    match lerp.lerp_type {
                        LerpType::World { .. } | LerpType::UI { .. } => {
                            lerp_amount = -0.5 * (PI * lerp_amount).cos() + 0.5;
                        }
                        LerpType::UIToWorld { .. } => {
                            lerp_amount = lerp_amount.powi(2);
                        }
                        LerpType::WorldToUI { .. } => {
                            lerp_amount = (lerp_amount - 1.0).powi(3) + 1.0;
                        }
                        _ => (),
                    }

                    transform.translation = lerp.src.unwrap().translation.lerp(dest.translation, lerp_amount);
                    transform.rotation = lerp.src.unwrap().rotation.lerp(dest.rotation, lerp_amount);
                    transform.scale = lerp.src.unwrap().scale.lerp(dest.scale, lerp_amount);

                    lerp.time -= time.delta_seconds() * SPEED_MOD;
                }
            }
        }
    }
}

fn camera_system(
    mut commands: Commands,
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

                commands.entity(entity).remove::<Lerp>();
            } else {
                let dest_transform = Transform::from_translation(dest.pos).looking_at(dest.at, dest.up);
                let mut lerp_amount = PI * (lerp.animation_time - lerp.time) / lerp.animation_time;
                lerp_amount = -0.5 * lerp_amount.cos() + 0.5;
                transform.translation = src.unwrap().translation.lerp(dest_transform.translation, lerp_amount);
                transform.rotation = src.unwrap().rotation.lerp(dest_transform.rotation, lerp_amount);

                lerp.time -= time.delta_seconds() * SPEED_MOD;
            }
        } else {
            commands.entity(entity).remove::<Lerp>();
        }
    }
}
