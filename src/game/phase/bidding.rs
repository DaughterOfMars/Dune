use bevy::prelude::*;
use derive_more::Display;
use serde::{Deserialize, Serialize};

pub struct BiddingPlugin;

impl Plugin for BiddingPlugin {
    fn build(&self, app: &mut App) {
        todo!()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Display, Serialize, Deserialize)]
pub enum BiddingPhase {
    DealCards,
    Bidding,
}
