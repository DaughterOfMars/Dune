use std::f32::consts::PI;

use bevy::prelude::*;
use derive_more::Display;
use iyes_loopless::prelude::IntoConditionalSystem;
use renet::RenetClient;
use serde::{Deserialize, Serialize};

use crate::{
    components::TreacheryCard,
    game::{
        state::{GameEvent, GameState, PlayerId},
        GameEventStage, ObjectEntityMap, ObjectId, PickedEvent,
    },
    lerper::{Lerp, Lerper, UITransform},
    network::{GameEvents, SendEvent},
    util::bid_positions,
    Screen,
};

pub struct BiddingPlugin;

impl Plugin for BiddingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(GameEventStage, bid)
            .add_system_to_stage(GameEventStage, win_bid)
            .add_system(make_bid.run_in_state(Screen::Game));
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
                    UITransform::from(pos).with_rotation(Quat::from_rotation_x(PI / 2.0) * Quat::from_rotation_z(PI)),
                    0.1,
                    0.0,
                ));
            }
        }
    }
}

fn make_bid(
    mut client: ResMut<RenetClient>,
    game_state: Res<GameState>,
    mut picked_events: EventReader<PickedEvent<TreacheryCard>>,
    cards: Query<&ObjectId, With<TreacheryCard>>,
    my_id: Res<PlayerId>,
) {
    for PickedEvent { picked, inner: _ } in picked_events.iter() {
        if let Ok(card_id) = cards.get(*picked) {
            let bid_state = game_state.bidding_cards.current().unwrap();
            if &bid_state.card.id == card_id {
                let current_bid = bid_state.current_bid.as_ref().map(|b| b.spice).unwrap_or_default();
                client.send_event(GameEvent::MakeBid {
                    player_id: *my_id,
                    spice: current_bid + 1,
                });
            }
        }
    }
}

fn win_bid(
    mut commands: Commands,
    game_events: Res<GameEvents>,
    object_entity: Res<ObjectEntityMap>,
    my_id: Res<PlayerId>,
) {
    if let Some(GameEvent::WinBid { player_id, card_id }) = game_events.peek() {
        if *my_id != *player_id {
            // TODO: animate into a UI component
            commands.entity(object_entity.world[card_id]).despawn_recursive();
        }
    }
}
