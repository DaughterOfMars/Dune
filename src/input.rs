use bevy::{prelude::*, render::camera::Camera};
use ncollide3d::{
    na::{Isometry3, Point3, Translation3, UnitQuaternion, Vector3},
    query::{Ray, RayCast},
};

use crate::{
    components::{ClickAction, Collider},
    resources::{Data, Info},
    stack::{Action, ActionStack},
    util::screen_to_world,
};

pub fn input_system(
    mut stack: ResMut<ActionStack>,
    info: Res<Info>,
    data: Res<Data>,
    windows: Res<Windows>,
    mouse_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    cameras: Query<(&Transform, &Camera)>,
    colliders: Query<(Entity, &Collider, &Transform, &ClickAction)>,
) {
    let (input_enabled, context) = match stack.peek() {
        Some(Action::Await { context, .. }) => (true, context),
        None => (true, &None),
        _ => (false, &None),
    };
    if input_enabled {
        if mouse_input.just_pressed(MouseButton::Left) {
            if let Some((&cam_transform, camera)) = cameras.iter().next() {
                if let Some(window) = windows.get_primary() {
                    if let Some(pos) = window.cursor_position() {
                        let ss_pos = Vec2::new(
                            2.0 * (pos.x / window.physical_width() as f32) - 1.0,
                            2.0 * (pos.y / window.physical_height() as f32) - 1.0,
                        );
                        let p0 = screen_to_world(
                            ss_pos.extend(0.0),
                            cam_transform,
                            camera.projection_matrix,
                        );
                        let p1 = screen_to_world(
                            ss_pos.extend(1.0),
                            cam_transform,
                            camera.projection_matrix,
                        );
                        let dir = (p1 - p0).normalize();
                        let ray = Ray::new(
                            Point3::new(p0.x, p0.y, p0.z),
                            Vector3::new(dir.x, dir.y, dir.z),
                        );
                        let (mut closest_toi, mut closest_action) = (None, None);
                        for (collider, transform, action) in
                            colliders
                                .iter()
                                .filter_map(|(entity, collider, transform, action)| {
                                    if action.enabled {
                                        if let Some(context) = context {
                                            if let Some(generator) =
                                                action.contextual_actions.get(context)
                                            {
                                                return Some((
                                                    collider,
                                                    transform,
                                                    generator(entity),
                                                ));
                                            }
                                        }
                                    }
                                    if let Some(action) = action.base_action.as_ref() {
                                        return Some((collider, transform, action.clone()));
                                    }
                                    None
                                })
                        {
                            let (axis, angle) = transform.rotation.to_axis_angle();
                            let angleaxis = axis * angle;
                            if let Some(toi) = collider.shape.toi_with_ray(
                                &Isometry3::from_parts(
                                    Translation3::new(
                                        transform.translation.x,
                                        transform.translation.y,
                                        transform.translation.z,
                                    ),
                                    UnitQuaternion::new(Vector3::new(
                                        angleaxis.x,
                                        angleaxis.y,
                                        angleaxis.z,
                                    )),
                                ),
                                &ray,
                                100.0,
                                true,
                            ) {
                                println!("Toi: {}", toi);
                                if closest_toi.is_none() {
                                    closest_toi = Some(toi);
                                    closest_action = Some(action);
                                } else {
                                    if toi < closest_toi.unwrap() {
                                        closest_toi = Some(toi);
                                        closest_action = Some(action);
                                    }
                                }
                            }
                        }
                        if let Some(action) = closest_action {
                            stack.pop();
                            stack.push(action);
                        }
                    }
                }
            }
        } else if keyboard_input.just_pressed(KeyCode::Escape) {
            stack.push(Action::move_camera(data.camera_nodes.main, 1.5));
        }
    }
}
