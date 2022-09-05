use std::f32::consts::PI;

use bevy::{
    math::vec2,
    prelude::*,
    render::camera::{Camera, OrthographicProjection},
};
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
        Self::Linear
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

    pub fn ui_to_world(dest: Transform, time: f32, delay: f32) -> Self {
        Lerp::new(LerpType::ui_to_world(dest), time, delay)
    }

    pub fn with_interpolation(mut self, interp_fn: impl InterpolationFn + Send + Sync + 'static) -> Self {
        self.interp_fn = Box::new(interp_fn);
        self
    }
}

#[derive(Copy, Clone, Inspectable)]
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

#[derive(Component, Inspectable)]
pub struct UIElement {
    #[inspectable(ignore)]
    pub cam: Entity,
    pub ui_transform: UITransform,
}

pub struct LerpCompleted {
    pub entity: Entity,
}

pub struct LerpPlugin;

impl Plugin for LerpPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LerpCompleted>()
            .add_system(camera_system)
            .add_system(lerp_system)
            .add_system(ui_element_system)
            .register_inspectable::<UIElement>();
    }
}

fn lerp_system(
    mut commands: Commands,
    mut event_writer: EventWriter<LerpCompleted>,
    time: Res<Time>,
    cameras: Query<(&GlobalTransform, &Camera), Without<OrthographicProjection>>,
    mut lerps: Query<(Entity, &mut Lerp, &mut Transform, Option<&mut UIElement>), Without<Camera>>,
) {
    for (entity, mut lerp, mut transform, opt_ui_element) in lerps.iter_mut() {
        let (mut dest_ui, mut dest_3d, mut ui_element) = (None, None, None);
        match lerp.lerp_type {
            LerpType::UI { src, dest, cam } => {
                if let Some(src) = src {
                    ui_element = Some(UIElement { cam, ui_transform: src });
                } else {
                    ui_element = Some(UIElement {
                        cam,
                        ui_transform: opt_ui_element.map(|e| e.ui_transform).unwrap_or_default(),
                    });
                }
                dest_ui = Some(dest);
            }
            LerpType::World { src, dest } => {
                if let Some(src) = src {
                    *transform = src;
                }
                dest_3d = Some(dest);
            }
            LerpType::UIToWorld { src, dest } => {
                if let Some(src) = src {
                    if let Some(ui_element) = opt_ui_element {
                        if let Ok((cam_transform, camera)) = cameras.get(ui_element.cam) {
                            let cam_transform = cam_transform.compute_transform();
                            *transform = Transform::from_translation(screen_to_world(
                                src.translation,
                                cam_transform,
                                camera.projection_matrix(),
                            )) * Transform::from_rotation(cam_transform.rotation * src.rotation)
                                * Transform::from_scale(Vec3::splat(UI_SCALE * src.scale));
                        }
                        commands.entity(entity).remove::<UIElement>();
                    }
                }
                dest_3d = Some(dest);
            }
            LerpType::WorldToUI { src, dest, cam } => {
                if let Some(src) = src {
                    if let Ok((cam_transform, camera)) = cameras.get(cam) {
                        ui_element = Some(UIElement {
                            cam,
                            ui_transform: camera
                                .world_to_viewport(cam_transform, src.translation)
                                .map(Into::into)
                                .unwrap_or_default(),
                        });
                    }
                } else {
                    ui_element = Some(UIElement {
                        cam,
                        ui_transform: opt_ui_element.map(|e| e.ui_transform).unwrap_or_default(),
                    });
                }
                dest_ui = Some(dest);
            }
            _ => (),
        }

        if let Some(dest) = dest_3d {
            if lerp.delay > 0.0 {
                lerp.delay -= time.delta_seconds() * SPEED_MOD;
            } else {
                if lerp.remaining_time <= 0.05 {
                    *transform = dest;
                    commands.entity(entity).remove::<Lerp>();
                    event_writer.send(LerpCompleted { entity });
                } else {
                    let prev_lerp_amount = (lerp.animation_time - lerp.remaining_time) / lerp.animation_time;
                    lerp.remaining_time -= time.delta_seconds() * SPEED_MOD;
                    let new_lerp_amount = (lerp.animation_time - lerp.remaining_time) / lerp.animation_time;
                    let lerp_amount = lerp.interp_fn.interpolate(new_lerp_amount - prev_lerp_amount);

                    transform.translation = transform.translation.lerp(dest.translation, lerp_amount);
                    transform.rotation = transform.rotation.lerp(dest.rotation, lerp_amount);
                    transform.scale = transform.scale.lerp(dest.scale, lerp_amount);
                }
            }
        } else if let (Some(dest), Some(mut ui_element)) = (dest_ui, ui_element) {
            if lerp.delay > 0.0 {
                lerp.delay -= time.delta_seconds() * SPEED_MOD;
            } else {
                let ui_transform = &mut ui_element.ui_transform;
                if lerp.remaining_time <= 0.05 {
                    *ui_transform = dest;
                    commands.entity(entity).remove::<Lerp>();
                    event_writer.send(LerpCompleted { entity });
                } else {
                    let prev_lerp_amount = (lerp.animation_time - lerp.remaining_time) / lerp.animation_time;
                    lerp.remaining_time -= time.delta_seconds() * SPEED_MOD;
                    let new_lerp_amount = (lerp.animation_time - lerp.remaining_time) / lerp.animation_time;
                    let lerp_amount = lerp.interp_fn.interpolate(new_lerp_amount - prev_lerp_amount);

                    ui_transform.translation = ui_transform.translation.lerp(dest.translation, lerp_amount);
                    ui_transform.rotation = ui_transform.rotation.lerp(dest.rotation, lerp_amount);
                    ui_transform.scale = ui_transform.scale + (dest.scale - ui_transform.scale) * lerp_amount;
                }
                commands.entity(entity).insert(ui_element);
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
        if let LerpType::Camera { dest } = lerp.lerp_type {
            if lerp.remaining_time <= 0.0 {
                *transform = Transform::from_translation(dest.pos).looking_at(dest.at, dest.up);

                commands.entity(entity).remove::<Lerp>();
            } else {
                let prev_lerp_amount = (lerp.animation_time - lerp.remaining_time) / lerp.animation_time;
                lerp.remaining_time -= time.delta_seconds() * SPEED_MOD;
                let new_lerp_amount = (lerp.animation_time - lerp.remaining_time) / lerp.animation_time;
                let lerp_amount = lerp.interp_fn.interpolate(new_lerp_amount - prev_lerp_amount);

                let dest_transform = Transform::from_translation(dest.pos).looking_at(dest.at, dest.up);
                transform.translation = transform.translation.lerp(dest_transform.translation, lerp_amount);
                transform.rotation = transform.rotation.lerp(dest_transform.rotation, lerp_amount);
            }
        } else {
            commands.entity(entity).remove::<Lerp>();
        }
    }
}

fn ui_element_system(
    mut ui_elements: Query<(&mut Transform, &UIElement), Without<Camera>>,
    cameras: Query<(&GlobalTransform, &Camera)>,
) {
    for (mut transform, ui_element) in ui_elements.iter_mut() {
        if let Ok((cam_transform, camera)) = cameras.get(ui_element.cam) {
            let cam_transform = cam_transform.compute_transform();
            *transform = Transform::from_translation(screen_to_world(
                ui_element.ui_transform.translation,
                cam_transform,
                camera.projection_matrix(),
            )) * Transform::from_rotation(cam_transform.rotation * ui_element.ui_transform.rotation)
                * Transform::from_scale(Vec3::splat(UI_SCALE * ui_element.ui_transform.scale))
        }
    }
}
