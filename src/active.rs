use bevy::prelude::*;
use iyes_loopless::prelude::IntoConditionalSystem;

use crate::resources::Info;

pub struct Active {
    pub entity: Entity,
}

pub struct NextActive {
    pub entity: Entity,
}

pub struct AdvanceActive;

pub struct ActivePlugin;

impl Plugin for ActivePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PreUpdate, next_active.run_if_resource_exists::<NextActive>())
            .add_system_to_stage(
                CoreStage::PreUpdate,
                advance_active.run_if_resource_exists::<AdvanceActive>(),
            );
    }
}

fn next_active(mut commands: Commands, mut active: ResMut<Active>, next_active: Res<NextActive>) {
    if next_active.entity != active.entity {
        active.entity = next_active.entity;
    }
    commands.remove_resource::<NextActive>();
}

fn advance_active(mut commands: Commands, mut active: ResMut<Active>, info: Res<Info>) {
    let mut turn_cycle = info.turn_order.iter().cycle();
    while let Some(next) = turn_cycle.next() {
        if *next == active.entity {
            active.entity = *turn_cycle.next().unwrap();
            break;
        }
    }
    commands.remove_resource::<AdvanceActive>();
}
