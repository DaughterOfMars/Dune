use std::{f32::consts::PI, time::Duration};

use bevy::{
    math::{vec2, vec3},
    prelude::*,
};
use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::{
    components::StormCard,
    game::{
        state::{GameEvent, GameState},
        GameEventPauser, GameEventStage, ObjectEntityMap,
    },
    lerper::{Lerp, Lerper, UITransform},
    network::GameEvents,
};

pub struct StormPlugin;

impl Plugin for StormPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(GameEventStage, reveal)
            .add_system_to_stage(GameEventStage, move_storm);
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Display, Serialize, Deserialize)]
pub enum StormPhase {
    Reveal,
    WeatherControl,
    FamilyAtomics,
    MoveStorm,
}

fn reveal(
    game_events: Res<GameEvents>,
    game_state: Res<GameState>,
    object_entity: Res<ObjectEntityMap>,
    mut storm_cards: Query<&mut Lerper>,
    mut pause: ResMut<GameEventPauser>,
) {
    if let Some(GameEvent::RevealStorm) = game_events.peek() {
        if let Ok(mut lerper) = storm_cards.get_mut(object_entity.world[&game_state.storm_card.as_ref().unwrap().id]) {
            lerper.push(Lerp::ui_to(
                UITransform::from(vec2(0.0, 0.0)).with_rotation(Quat::from_rotation_x(PI / 2.0)),
                0.1,
                0.0,
            ));
        }
        pause.pause_for(Duration::from_secs(3));
    }
}

fn move_storm(game_events: Res<GameEvents>, mut storm_cards: Query<&mut Lerper, With<StormCard>>) {
    if let Some(GameEvent::MoveStorm { sectors }) = game_events.peek() {
        // TODO move storm
        for mut lerper in storm_cards.iter_mut() {
            // TODO: shuffle
            lerper.push(Lerp::world_to(
                Transform::from_translation(vec3(1.5, 0.0049, 0.87)),
                0.1,
                1.0,
            ));
        }
    }
}
