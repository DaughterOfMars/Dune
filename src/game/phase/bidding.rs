use std::f32::consts::PI;

use bevy::prelude::*;
use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::{
    game::{
        state::{GameEvent, GameState},
        GameEventStage, ObjectEntityMap,
    },
    lerper::{Lerp, Lerper, UITransform},
    network::GameEvents,
    util::bid_positions,
};

pub struct BiddingPlugin;

impl Plugin for BiddingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(GameEventStage, bid);
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Display, Serialize, Deserialize)]
pub enum BiddingPhase {
    DealCards,
    Bidding,
}

fn bid(
    game_events: Res<GameEvents>,
    game_state: Res<GameState>,
    object_entity: Res<ObjectEntityMap>,
    mut bid_cards: Query<&mut Lerper>,
) {
    if let Some(GameEvent::StartBidding | GameEvent::WinBid { .. }) = game_events.peek() {
        let positions = bid_positions(game_state.bidding_cards.len());
        for (bid_state, pos) in game_state.bidding_cards.iter().zip(positions.into_iter()) {
            if let Ok(mut lerper) = bid_cards.get_mut(object_entity.world[&bid_state.card.id]) {
                lerper.push(Lerp::ui_to(
                    UITransform::from(pos).with_rotation(Quat::from_rotation_x(PI / 2.0)),
                    0.1,
                    0.0,
                ));
            }
        }
    }
}
