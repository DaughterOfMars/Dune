use std::f32::consts::PI;

use bevy::{math::vec2, prelude::*, render::camera::Camera};
use bevy_inspector_egui::{Inspectable, RegisterInspectable};

use crate::{data::CameraNode, util::screen_to_world};

const UI_SCALE: f32 = 1.0;
const SPEED_MOD: f32 = 1.0;

#[derive(Copy, Clone)]
enum LerpType {
    UI {
        src: Option<UITransform>,
        dest: UITransform,
        cam: Entity,
    },
    World {
        src: Option<Transform>,
        dest: Transform,
    },
    UIToWorld {
        src: Option<UITransform>,
        dest: Transform,
        cam: Entity,
    },
    WorldToUI {
        src: Option<Transform>,
        dest: UITransform,
        cam: Entity,
    },
    Camera {
        dest: CameraNode,
    },
}

impl LerpType {
    pub fn ui_to(dest: impl Into<UITransform>, cam: Entity) -> Self {
        LerpType::UI {
            src: None,
            dest: dest.into(),
            cam,
        }
    }

    pub fn ui_from_to(src: impl Into<UITransform>, dest: impl Into<UITransform>, cam: Entity) -> Self {
        LerpType::UI {
            src: Some(src.into()),
            dest: dest.into(),
            cam,
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

    pub fn world_to_ui(dest: impl Into<UITransform>, cam: Entity) -> Self {
        LerpType::WorldToUI {
            src: None,
            dest: dest.into(),
            cam,
        }
    }

    pub fn card_to_ui(dest: impl Into<UITransform>, cam: Entity) -> Self {
        LerpType::WorldToUI {
            src: None,
            dest: dest.into().with_rotation(Quat::from_rotation_x(0.5 * PI)),
            cam,
        }
    }

    pub fn ui_to_world(dest: Transform, cam: Entity) -> Self {
        LerpType::UIToWorld { src: None, dest, cam }
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

#[derive(Component)]
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

    pub fn ui_to(dest: impl Into<UITransform>, cam: Entity, time: f32, delay: f32) -> Self {
        Lerp::new(LerpType::ui_to(dest, cam), time, delay)
    }

    pub fn ui_from_to(
        src: impl Into<UITransform>,
        dest: impl Into<UITransform>,
        cam: Entity,
        time: f32,
        delay: f32,
    ) -> Self {
        Lerp::new(LerpType::ui_from_to(src, dest, cam), time, delay)
    }

    pub fn world_to(dest: impl Into<Transform>, time: f32, delay: f32) -> Self {
        Lerp::new(LerpType::world_to(dest), time, delay)
    }

    pub fn world_from_to(src: impl Into<Transform>, dest: impl Into<Transform>, time: f32, delay: f32) -> Self {
        Lerp::new(LerpType::world_from_to(src, dest), time, delay)
    }

    pub fn world_to_ui(dest: impl Into<UITransform>, cam: Entity, time: f32, delay: f32) -> Self {
        Lerp::new(LerpType::world_to_ui(dest, cam), time, delay)
    }

    pub fn card_to_ui(dest: impl Into<UITransform>, cam: Entity, time: f32, delay: f32) -> Self {
        Lerp::new(LerpType::card_to_ui(dest, cam), time, delay)
    }

    pub fn ui_to_world(dest: Transform, cam: Entity, time: f32, delay: f32) -> Self {
        Lerp::new(LerpType::ui_to_world(dest, cam), time, delay)
    }

    pub fn with_interpolation(mut self, interp_fn: impl InterpolationFn + Send + Sync + 'static) -> Self {
        self.interp_fn = Box::new(interp_fn);
        self
    }
}

#[derive(Copy, Clone, Component, Inspectable)]
pub struct UITransform {
    translation: Vec2,
    rotation: Quat,
    scale: f32,
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
pub struct UIElement {
    pub cam: Entity,
}

pub struct LerpCompletedEvent {
    pub entity: Entity,
}

pub struct LerpPlugin;

impl Plugin for LerpPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LerpCompletedEvent>()
            .add_system(init_lerp_system)
            .add_system(lerp_ui_system)
            .add_system(lerp_world_system)
            .add_system(ui_element_system)
            .register_inspectable::<UITransform>();
    }
}

#[derive(Component)]
pub struct LerpUI {
    src: UITransform,
    dest: UITransform,
}

#[derive(Component)]
pub struct LerpWorld {
    src: Transform,
    dest: Transform,
}

fn init_lerp_system(
    mut commands: Commands,
    cameras: Query<(&GlobalTransform, &Camera)>,
    mut lerps: Query<(Entity, &Lerp, &Transform), Added<Lerp>>,
) {
    for (entity, lerp, transform) in lerps.iter_mut() {
        match lerp.lerp_type {
            LerpType::UI { src, dest, cam } => {
                let src = src.unwrap_or_else(|| {
                    if let Ok((cam_transform, camera)) = cameras.get(cam) {
                        camera
                            .world_to_viewport(cam_transform, transform.translation)
                            .map(Into::into)
                            .unwrap_or_default()
                    } else {
                        Default::default()
                    }
                });
                commands
                    .entity(entity)
                    .insert(LerpUI { src, dest })
                    .insert(src)
                    .insert(UIElement { cam });
            }
            LerpType::World { src, dest } => {
                commands
                    .entity(entity)
                    .insert(LerpWorld {
                        src: src.unwrap_or(*transform),
                        dest,
                    })
                    .remove::<UITransform>()
                    .remove::<UIElement>();
            }
            LerpType::UIToWorld { src, dest, cam } => {
                commands
                    .entity(entity)
                    .insert(LerpWorld {
                        src: src
                            .and_then(|src| {
                                if let Ok((cam_transform, camera)) = cameras.get(cam) {
                                    let cam_transform = cam_transform.compute_transform();
                                    Some(
                                        Transform::from_translation(screen_to_world(
                                            src.translation,
                                            cam_transform,
                                            camera.projection_matrix(),
                                        )) * Transform::from_rotation(cam_transform.rotation * src.rotation)
                                            * Transform::from_scale(Vec3::splat(UI_SCALE * src.scale)),
                                    )
                                } else {
                                    None
                                }
                            })
                            .unwrap_or(*transform),
                        dest,
                    })
                    .remove::<UITransform>()
                    .remove::<UIElement>();
            }
            LerpType::WorldToUI { src, dest, cam } => {
                let src = src
                    .and_then(|src| {
                        if let Ok((cam_transform, camera)) = cameras.get(cam) {
                            camera.world_to_viewport(cam_transform, src.translation).map(Into::into)
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();
                commands
                    .entity(entity)
                    .insert(LerpUI { src, dest })
                    .insert(src)
                    .insert(UIElement { cam });
            }
            LerpType::Camera { dest } => {
                commands
                    .entity(entity)
                    .insert(LerpWorld {
                        src: *transform,
                        dest: Transform::from_translation(dest.pos).looking_at(dest.at, dest.up),
                    })
                    .remove::<UITransform>()
                    .remove::<UIElement>();
            }
        }
    }
}

fn lerp_ui_system(
    mut commands: Commands,
    mut event_writer: EventWriter<LerpCompletedEvent>,
    time: Res<Time>,
    mut lerps: Query<(Entity, &mut Lerp, &LerpUI, &mut UITransform), Without<Camera>>,
) {
    for (entity, mut lerp, lerp_ui, mut transform) in lerps.iter_mut() {
        if lerp.delay > 0.0 {
            lerp.delay -= time.delta_seconds() * SPEED_MOD;
        } else {
            if lerp.remaining_time <= 0.0 {
                commands.entity(entity).remove::<Lerp>().remove::<LerpUI>();
                event_writer.send(LerpCompletedEvent { entity });
            } else {
                lerp.remaining_time -= time.delta_seconds() * SPEED_MOD;
                let lerp_amount = lerp
                    .interp_fn
                    .interpolate((lerp.animation_time - lerp.remaining_time) / lerp.animation_time);

                transform.translation = lerp_ui.src.translation.lerp(lerp_ui.dest.translation, lerp_amount);
                transform.rotation = lerp_ui.src.rotation.lerp(lerp_ui.dest.rotation, lerp_amount);
                transform.scale = lerp_ui.src.scale + (lerp_ui.dest.scale - lerp_ui.src.scale) * lerp_amount;
            }
        }
    }
}

fn lerp_world_system(
    mut commands: Commands,
    mut event_writer: EventWriter<LerpCompletedEvent>,
    time: Res<Time>,
    mut lerps: Query<(Entity, &mut Lerp, &LerpWorld, &mut Transform)>,
) {
    for (entity, mut lerp, lerp_world, mut transform) in lerps.iter_mut() {
        if lerp.delay > 0.0 {
            lerp.delay -= time.delta_seconds() * SPEED_MOD;
        } else {
            if lerp.remaining_time <= 0.0 {
                *transform = lerp_world.dest;
                commands.entity(entity).remove::<Lerp>().remove::<LerpWorld>();
                event_writer.send(LerpCompletedEvent { entity });
            } else {
                lerp.remaining_time -= time.delta_seconds() * SPEED_MOD;
                let lerp_amount = lerp
                    .interp_fn
                    .interpolate((lerp.animation_time - lerp.remaining_time) / lerp.animation_time);

                transform.translation = lerp_world
                    .src
                    .translation
                    .lerp(lerp_world.dest.translation, lerp_amount);
                transform.rotation = lerp_world.src.rotation.lerp(lerp_world.dest.rotation, lerp_amount);
                transform.scale = lerp_world.src.scale.lerp(lerp_world.dest.scale, lerp_amount);
            }
        }
    }
}

fn ui_element_system(
    mut ui_elements: Query<(&mut Transform, &UITransform, &UIElement), Without<Camera>>,
    cameras: Query<(&GlobalTransform, &Camera)>,
) {
    for (mut transform, ui_transform, ui_element) in ui_elements.iter_mut() {
        if let Ok((cam_transform, camera)) = cameras.get(ui_element.cam) {
            let cam_transform = cam_transform.compute_transform();
            *transform = Transform::from_translation(screen_to_world(
                ui_transform.translation,
                cam_transform,
                camera.projection_matrix(),
            )) * Transform::from_rotation(cam_transform.rotation * ui_transform.rotation)
                * Transform::from_scale(Vec3::splat(UI_SCALE * ui_transform.scale))
        }
    }
}
