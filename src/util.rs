use std::{collections::HashMap, f32::consts::PI};

use bevy::{
    math::{vec2, Mat4, Vec3, Vec4Swizzles},
    prelude::*,
};
use rand::{prelude::SliceRandom, Rng};

const UI_Z: f32 = 0.008;

pub fn screen_to_world(ss_pos: Vec2, cam_transform: Transform, projection_matrix: Mat4) -> Vec3 {
    let p = cam_transform.compute_matrix() * projection_matrix.inverse() * ss_pos.extend(UI_Z).extend(1.0);
    p.xyz() / p.w
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

pub fn shuffle_deck<R>(rng: &mut R, offset: f32, entities: &mut HashMap<Entity, Mut<Transform>>)
where
    R: Rng + ?Sized,
{
    let start = entities
        .values()
        .min_by(|transform1, transform2| transform1.translation.y.partial_cmp(&transform2.translation.y).unwrap())
        .unwrap()
        .translation;
    let mut order = entities.keys().cloned().collect::<Vec<_>>();
    order.shuffle(rng);
    for entity in order {
        entities.get_mut(&entity).unwrap().translation = start + (offset * Vec3::Y);
    }
}

pub fn hand_positions(n: usize) -> Vec<Vec2> {
    // TODO: Make this radial
    (0..n)
        .map(|i| vec2(2.0 * ((1.0 + i as f32) / (1.0 + n as f32)) - 1.0, -1.1))
        .collect()
}

pub fn card_jitter() -> Transform {
    Transform::from_translation(Vec3::X * rand::random::<f32>() * 0.001)
        * Transform::from_translation(Vec3::Z * rand::random::<f32>() * 0.001)
        * Transform::from_rotation(Quat::from_rotation_y(rand::random::<f32>() * PI * 0.01))
}
