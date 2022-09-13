use std::{f32::consts::PI, time::Duration};

use bevy::{
    math::{vec2, vec3},
    prelude::*,
};
use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::{
    game::{
        state::{GameEvent, GameState},
        GameEventPauser, GameEventStage, ObjectEntityMap,
    },
    lerper::{Lerp, Lerper, UITransform},
    network::GameEvents,
};

pub struct SpiceBlowPlugin;

impl Plugin for SpiceBlowPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(GameEventStage, reveal)
            .add_system_to_stage(GameEventStage, place_spice);
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Display, Serialize, Deserialize)]
pub enum SpiceBlowPhase {
    Reveal,
    ShaiHalud,
    PlaceSpice,
}

#[derive(Component)]
pub struct RevealedSpiceCard;

fn reveal(
    mut commands: Commands,
    game_events: Res<GameEvents>,
    game_state: Res<GameState>,
    object_entity: Res<ObjectEntityMap>,
    mut spice_cards: Query<&mut Lerper>,
    mut pause: ResMut<GameEventPauser>,
) {
    if let Some(GameEvent::RevealSpiceBlow) = game_events.peek() {
        let entity = object_entity.world[&game_state.spice_card.as_ref().unwrap().id];
        if let Ok(mut lerper) = spice_cards.get_mut(entity) {
            lerper.push(Lerp::ui_to(
                UITransform::from(vec2(0.0, 0.0)).with_rotation(Quat::from_rotation_x(PI / 2.0)),
                0.1,
                0.0,
            ));
            pause.pause_for(Duration::from_secs(3));
            commands.entity(entity).insert(RevealedSpiceCard);
        }
    }
}

fn place_spice(
    mut commands: Commands,
    game_events: Res<GameEvents>,
    mut spice_card: Query<(Entity, &mut Lerper), With<RevealedSpiceCard>>,
) {
    if let Some(GameEvent::PlaceSpice { location, spice }) = game_events.peek() {
        // TODO: Add spice tokens to board location
        // TODO: stack
        for (entity, mut lerper) in spice_card.iter_mut() {
            lerper.push(Lerp::world_to(
                Transform::from_translation(vec3(1.5, 0.0049, 0.87)),
                0.1,
                0.0,
            ));
            commands.entity(entity).remove::<RevealedSpiceCard>();
        }
    }
}
