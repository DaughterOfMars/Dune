use std::{collections::VecDeque, f32::consts::PI};

use bevy::{math::vec2, prelude::*, render::camera::Camera};

use crate::{data::CameraNode, util::screen_to_world};

const UI_SCALE: f32 = 1.0;
const SPEED_MOD: f32 = 1.0;

#[derive(Default, Component)]
pub struct Lerper {
    queue: VecDeque<Lerp>,
    current: Option<Lerp>,
}

impl Lerper {
    pub fn push(&mut self, lerp: Lerp) {
        if let Some(last) = self.queue.back().or(self.current.as_ref()) {
            if last.lerp_type != lerp.lerp_type {
                self.queue.push_back(lerp);
            }
        } else {
            self.queue.push_back(lerp);
        }
    }

    pub fn set_if_empty(&mut self, lerp: Lerp) {
        if self.current.is_none() && self.queue.is_empty() {
            self.queue.push_back(lerp);
        }
    }

    pub fn replace(&mut self, lerp: Lerp) {
        self.queue.clear();
        if let Some(current) = &self.current {
            if current.lerp_type != lerp.lerp_type {
                self.current.take();
                self.queue.push_back(lerp);
            }
        } else {
            self.queue.push_back(lerp);
        }
    }

    pub fn with(mut self, lerp: Lerp) -> Self {
        self.push(lerp);
        self
    }
}

impl From<Lerp> for Lerper {
    fn from(lerp: Lerp) -> Self {
        Self::default().with(lerp)
    }
}

#[derive(PartialEq, Copy, Clone)]
enum LerpType {
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
        dest: CameraNode,
    },
}

impl LerpType {
    pub fn ui_to(dest: impl Into<UITransform>) -> Self {
        LerpType::UI {
            src: None,
            dest: dest.into(),
        }
    }

    pub fn ui_from_to(src: impl Into<UITransform>, dest: impl Into<UITransform>) -> Self {
        LerpType::UI {
            src: Some(src.into()),
            dest: dest.into(),
        }
    }

    pub fn world_to(dest: impl Into<Transform>) -> Self {
        LerpType::World {
            src: None,
            dest: dest.into(),
        }
    }

    pub fn world_from_to(src: impl Into<Transform>, dest: impl Into<Transform>) -> Self {
        LerpType::World {
            src: Some(src.into()),
            dest: dest.into(),
        }
    }

    pub fn world_to_ui(dest: impl Into<UITransform>) -> Self {
        LerpType::WorldToUI {
            src: None,
            dest: dest.into(),
        }
    }

    pub fn card_to_ui(dest: impl Into<UITransform>) -> Self {
        LerpType::WorldToUI {
            src: None,
            dest: dest.into().with_rotation(Quat::from_rotation_x(0.5 * PI)),
        }
    }

    pub fn ui_to_world(dest: Transform) -> Self {
        LerpType::UIToWorld { src: None, dest }
    }
}

pub trait InterpolationFn {
    fn interpolate(&self, lerp_amount: f32) -> f32;
}

pub enum InterpolationFunction {
    Linear,
    Exponential,
    Cubic,
    Easing,
}

impl Default for InterpolationFunction {
    fn default() -> Self {
        Self::Easing
    }
}

impl InterpolationFn for InterpolationFunction {
    fn interpolate(&self, lerp_amount: f32) -> f32 {
        match self {
            Self::Linear => lerp_amount,
            Self::Exponential => lerp_amount.powi(2),
            Self::Cubic => lerp_amount.powi(3),
            Self::Easing => -0.5 * (PI * lerp_amount).cos() + 0.5,
        }
    }
}

impl<T: Fn(f32) -> f32> InterpolationFn for T {
    fn interpolate(&self, lerp_amount: f32) -> f32 {
        self(lerp_amount)
    }
}

pub struct Lerp {
    lerp_type: LerpType,
    pub remaining_time: f32,
    animation_time: f32,
    delay: f32,
    interp_fn: Box<dyn InterpolationFn + Send + Sync>,
}

impl Lerp {
    fn new(lerp_type: LerpType, time: f32, delay: f32) -> Self {
        Lerp {
            lerp_type,
            remaining_time: time,
            animation_time: time,
            delay,
            interp_fn: Box::new(InterpolationFunction::default()),
        }
    }

    pub fn move_camera(dest: CameraNode, time: f32) -> Self {
        Lerp {
            lerp_type: LerpType::Camera { dest },
            remaining_time: time,
            animation_time: time,
            delay: 0.0,
            interp_fn: Box::new(InterpolationFunction::default()),
        }
    }

    pub fn ui_to(dest: impl Into<UITransform>, time: f32, delay: f32) -> Self {
        Lerp::new(LerpType::ui_to(dest), time, delay)
    }

    pub fn ui_from_to(src: impl Into<UITransform>, dest: impl Into<UITransform>, time: f32, delay: f32) -> Self {
        Lerp::new(LerpType::ui_from_to(src, dest), time, delay)
    }

    pub fn world_to(dest: impl Into<Transform>, time: f32, delay: f32) -> Self {
        Lerp::new(LerpType::world_to(dest), time, delay)
    }

    pub fn world_from_to(src: impl Into<Transform>, dest: impl Into<Transform>, time: f32, delay: f32) -> Self {
        Lerp::new(LerpType::world_from_to(src, dest), time, delay)
    }

    pub fn world_to_ui(dest: impl Into<UITransform>, time: f32, delay: f32) -> Self {
        Lerp::new(LerpType::world_to_ui(dest), time, delay)
    }

    pub fn card_to_ui(dest: impl Into<UITransform>, time: f32, delay: f32) -> Self {
        Lerp::new(LerpType::card_to_ui(dest), time, delay)
    }

    pub fn ui_to_world(dest: Transform, time: f32, delay: f32) -> Self {
        Lerp::new(LerpType::ui_to_world(dest), time, delay)
    }

    pub fn with_interpolation(mut self, interp_fn: impl InterpolationFn + Send + Sync + 'static) -> Self {
        self.interp_fn = Box::new(interp_fn);
        self
    }

    pub fn is_complete(&self) -> bool {
        self.remaining_time <= 0.0
    }
}

#[derive(PartialEq, Copy, Clone, Debug, Component)]
pub struct UITransform {
    pub translation: Vec2,
    pub rotation: Quat,
    pub scale: f32,
}

impl Default for UITransform {
    fn default() -> Self {
        Self {
            translation: vec2(1.5, -1.5),
            rotation: Default::default(),
            scale: 1.0,
        }
    }
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

    fn to_world(self, cam_transform: Transform, camera: &Camera) -> Transform {
        Transform::from_translation(screen_to_world(
            self.translation,
            cam_transform,
            camera.projection_matrix(),
        )) * Transform::from_rotation(cam_transform.rotation * self.rotation)
            * Transform::from_scale(Vec3::splat(UI_SCALE * self.scale))
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

#[derive(Component)]
pub struct LerpUICamera;

pub struct LerpPlugin;

impl Plugin for LerpPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(lerper).add_system(lerp_world);
    }
}

fn lerper(
    mut commands: Commands,
    mut lerpers: Query<(Entity, &mut Lerper, &Transform)>,
    camera: Query<(&Transform, &Camera), With<LerpUICamera>>,
) {
    for (entity, mut lerper, transform) in lerpers.iter_mut() {
        let mut current = None;
        if lerper.current.as_ref().map(|lerp| lerp.is_complete()).unwrap_or(true) {
            current = lerper.queue.pop_front();
        }
        if let Some(lerp) = current {
            match lerp.lerp_type {
                LerpType::UI { src, dest } => {
                    let (cam_transform, camera) = camera.single();
                    let src = src
                        .map(|src| src.to_world(*cam_transform, camera))
                        .unwrap_or(*transform);
                    commands.entity(entity).insert(LerpPoints {
                        src,
                        dest: dest.to_world(*cam_transform, camera),
                    });
                }
                LerpType::World { src, dest } => {
                    commands.entity(entity).insert(LerpPoints {
                        src: src.unwrap_or(*transform),
                        dest,
                    });
                }
                LerpType::UIToWorld { src, dest } => {
                    commands.entity(entity).insert(LerpPoints {
                        src: src
                            .map(|src| {
                                let (cam_transform, camera) = camera.single();
                                src.to_world(*cam_transform, camera)
                            })
                            .unwrap_or(*transform),
                        dest,
                    });
                }
                LerpType::WorldToUI { src, dest } => {
                    let (cam_transform, camera) = camera.single();
                    commands.entity(entity).insert(LerpPoints {
                        src: src.unwrap_or(*transform),
                        dest: dest.to_world(*cam_transform, camera),
                    });
                }
                LerpType::Camera { dest } => {
                    commands.entity(entity).insert(LerpPoints {
                        src: *transform,
                        dest: Transform::from_translation(dest.pos).looking_at(dest.at, dest.up),
                    });
                }
            }
            lerper.current.replace(lerp);
        }
    }
}

#[derive(Component)]
pub struct LerpPoints {
    src: Transform,
    dest: Transform,
}

fn lerp_world(
    mut commands: Commands,
    time: Res<Time>,
    mut lerps: Query<(Entity, &mut Lerper, &LerpPoints, &mut Transform)>,
) {
    for (entity, mut lerper, lerp_points, mut transform) in lerps.iter_mut() {
        if let Some(lerp) = &mut lerper.current {
            if lerp.delay > 0.0 {
                lerp.delay -= time.delta_seconds() * SPEED_MOD;
            } else {
                if lerp.is_complete() {
                    *transform = lerp_points.dest;
                    commands.entity(entity).remove::<LerpPoints>();
                } else {
                    lerp.remaining_time -= time.delta_seconds() * SPEED_MOD;
                    let lerp_amount = lerp
                        .interp_fn
                        .interpolate((lerp.animation_time - lerp.remaining_time) / lerp.animation_time);

                    transform.translation = lerp_points
                        .src
                        .translation
                        .lerp(lerp_points.dest.translation, lerp_amount);
                    transform.rotation = lerp_points.src.rotation.lerp(lerp_points.dest.rotation, lerp_amount);
                    transform.scale = lerp_points.src.scale.lerp(lerp_points.dest.scale, lerp_amount);
                }
            }
        }
    }
}
