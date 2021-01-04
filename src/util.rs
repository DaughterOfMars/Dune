use bevy::{
    math::{Mat4, Vec3, Vec4Swizzles},
    prelude::*,
};

use crate::{
    components::{Player, Unique},
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
