use bevy::{
    math::{Mat4, Vec3, Vec4Swizzles},
    prelude::*,
    render::camera::Camera,
};
use ncollide3d::{
    na::{Isometry3, Point3, Translation3, UnitQuaternion, Vector3},
    query::{Ray, RayCast},
};

use crate::{
    components::{Collider, Player, Unique},
    resources::Info,
};

pub fn screen_to_world(ss_pos: Vec3, transform: Transform, v: Mat4) -> Vec3 {
    let p = transform.compute_matrix() * v.inverse() * ss_pos.extend(1.0);
    p.xyz() / p.w
}

pub fn set_view_to_active_player(
    info: &ResMut<Info>,
    players: &mut Query<(Entity, &mut Player)>,
    uniques: &mut Query<(&mut Visible, &Unique)>,
) {
    let entity = info.play_order[info.active_player];
    let active_player_faction = players.get_mut(entity).unwrap().1.faction;
    for (mut visible, unique) in uniques.iter_mut() {
        visible.is_visible = unique.faction == active_player_faction;
    }
}

pub fn divide_spice(mut total: i32) -> (i32, i32, i32, i32) {
    let (mut tens, mut fives, mut twos, mut ones) = (0, 0, 0, 0);
    while total > 0 {
        match total {
            1 => {
                total -= 1;
                ones += 1;
            }
            2..=4 => {
                total -= 2;
                twos += 1;
            }
            5..=9 => {
                total -= 5;
                fives += 1;
            }
            _ => {
                total -= 10;
                tens += 1;
            }
        }
    }
    (tens, fives, twos, ones)
}

pub fn compute_click_ray(
    window: &Window,
    click_pos: Vec2,
    camera: &Camera,
    cam_transform: &Transform,
) -> Ray<f32> {
    let ss_pos = Vec2::new(
        2.0 * (click_pos.x / window.physical_width() as f32) - 1.0,
        2.0 * (click_pos.y / window.physical_height() as f32) - 1.0,
    );
    let p0 = screen_to_world(ss_pos.extend(0.0), *cam_transform, camera.projection_matrix);
    let p1 = screen_to_world(ss_pos.extend(1.0), *cam_transform, camera.projection_matrix);
    let dir = (p1 - p0).normalize();
    Ray::new(
        Point3::new(p0.x, p0.y, p0.z),
        Vector3::new(dir.x, dir.y, dir.z),
    )
}

pub fn closest<'a, T: Component>(
    windows: &Res<Windows>,
    cameras: &Query<(&Camera, &Transform)>,
    colliders: &'a Query<(Entity, &Collider, &Transform, &'a T)>,
) -> Option<(Entity, &'a T)> {
    if let Some((camera, cam_transform)) = cameras.iter().next() {
        if let Some(window) = windows.get_primary() {
            if let Some(pos) = window.cursor_position() {
                let ray = compute_click_ray(window, pos, camera, cam_transform);
                let (mut closest_toi, mut closest_t) = (None, None);
                for (entity, collider, transform, t) in
                    colliders
                        .iter()
                        .filter_map(|(entity, collider, transform, t)| {
                            if collider.enabled {
                                return Some((entity, collider, transform, t));
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
                        if closest_toi.is_none() {
                            closest_toi = Some(toi);
                            closest_t = Some((entity, t));
                        } else {
                            if toi < closest_toi.unwrap() {
                                closest_toi = Some(toi);
                                closest_t = Some((entity, t));
                            }
                        }
                    }
                }
                return closest_t;
            }
        }
    }
    None
}
