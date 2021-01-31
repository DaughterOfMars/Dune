use std::collections::HashMap;

use bevy::{
    math::{Rect, Size, Vec2, Vec3},
    ui::Val,
};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Hash)]
pub(crate) enum Faction {
    Atreides,
    Harkonnen,
    Emperor,
    SpacingGuild,
    Fremen,
    BeneGesserit,
}

impl Faction {
    pub fn initial_values(&self) -> (i32, Option<Vec<String>>, i32) {
        match self {
            Self::Atreides => (10, Some(vec!["Arrakeen".to_string()]), 10),
            Self::BeneGesserit => (1, None, 5),
            Self::Fremen => (
                10,
                Some(vec![
                    "Sietch Tabr".to_string(),
                    "False Wall South".to_string(),
                    "False Wall West".to_string(),
                ]),
                10,
            ),
            Self::Emperor => (0, None, 10),
            Self::SpacingGuild => (5, Some(vec!["Tuek's Sietch".to_string()]), 5),
            Self::Harkonnen => (10, Some(vec!["Carthag".to_string()]), 10),
        }
    }
}

impl std::fmt::Display for Faction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Faction::Atreides => write!(f, "Atreides"),
            Faction::Harkonnen => write!(f, "Harkonnen"),
            Faction::Emperor => write!(f, "Emperor"),
            Faction::SpacingGuild => write!(f, "Spacing Guild"),
            Faction::Fremen => write!(f, "Fremen"),
            Faction::BeneGesserit => write!(f, "Bene Gesserit"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct Leader {
    pub name: String,
    pub power: i32,
    pub faction: Faction,
    pub texture: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct Location {
    pub name: String,
    pub terrain: Terrain,
    pub spice: Option<Vec3>,
    pub sectors: HashMap<i32, LocationNodes>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct LocationNodes {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<i32>,
    pub fighters: Vec<Vec3>,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub(crate) enum Terrain {
    Sand,
    Rock,
    Stronghold,
    PolarSink,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) enum Bonus {
    Carryalls,
    Ornothopters,
    Smugglers,
    Harvesters,
}
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub(crate) enum CardEffect {
    Worthless,
    PoisonWeapon,
    ProjectileWeapon,
    CheapHero,
    PoisonDefense,
    ProjectileDefense,
    Atomics,
    Movement,
    Karama,
    Lasgun,
    Revive,
    Truthtrance,
    WeatherControl,
}

impl CardEffect {
    fn description(&self) -> String {
        match self {
            CardEffect::Worthless =>
                "Play as part of your Battle Plan, in place of weapon, defense, or both.
                This card has no value in play, and you can discard it only by playing it in your Battle Plan.".to_string(),
            CardEffect::PoisonWeapon =>
                "Play as part of your Battle Plan.
                Kills enemy leader before the battle is resolved.
                May be countered by an appropriate defense (Snooper).
                You may keep this card if you win in this battle.".to_string(),
            CardEffect::ProjectileWeapon =>
                "Play as part of your Battle Plan.
                Kills enemy leader before the battle is resolved.
                May be countered by an appropriate defense (Shield).
                You may keep this card if you win in this battle.".to_string(),
            CardEffect::CheapHero =>
                "Play as a leader with zero strength on your Battle Plan.
                (leader allows you to play 1 weapon & 1 defense card on Battle Plan)
                Can be played together with another leader, allowing you to return & save that leader immediately after both Battle Plans were revealed.".to_string(),
            CardEffect::PoisonDefense =>
                "Play as part of your Battle Plan.
                Protects your leader from enemy poison weapon in this battle.
                You may keep this card if you win in this battle.".to_string(),
            CardEffect::ProjectileDefense =>
                "Play as part of your Battle Plan.
                Protects your leader from enemy projectile weapon in this battle.
                You may keep this card if you win in this battle.".to_string(),
            CardEffect::Atomics =>
                "Play after the storm movement is calulated by before storm is moved, and only if you have token(s) on the Shield Wall or an adjacent territory.
                All tokens in the Shield Wall are destroyed. Arrakeen, Imperial Basin & Carthag are no longer protected from the storm for the rest of the game.".to_string(),
            CardEffect::Movement =>
                "Play during Movement round.
                Take an additional on-planet token movement subject to normal movement rules.
                This may be the same or another group of your tokens.".to_string(),
            CardEffect::Karama =>
                "You may play this cartd to activate a single Karama Power of your choice.".to_string(),
            CardEffect::Lasgun =>
                "Play as part of your Battle Plan.
                Automatically kills enemy leader regardless of defense card used.
                You may keep this card if you win in this battle.
                If anyone plays a Shield in this battle, and neither leader is a traitor, then all tokens and leaders in this battle's territory are killed. Both players lose this battle.".to_string(),
            CardEffect::Revive =>
                "Play at any time.
                You may immediately revive 1 of your leaders of up to 5 of your tokens from the tanks to your reserves at no cost in spice.
                Does not count against per-turn limits on revivals.".to_string(),
            CardEffect::Truthtrance =>
                "Ask one other player a single yes/no question about the game which must be answered publicly.
                No game or rule CardEffect may interrupt the answer being given.
                The player must answer 'yes' or 'no' truthfully.".to_string(),
            CardEffect::WeatherControl =>
                "Play at the start of the Storm round, before the storm movement is calulated.
                You control the storm this round and may move it from 0 to 10 sectors in a counterclockwise direction.".to_string()
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct TreacheryCard {
    pub id: i32,
    pub effect: CardEffect,
    pub name: String,
    pub texture: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct TraitorCard {
    pub leader: Leader,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct SpiceCard {
    pub name: String,
    pub texture: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct StormCard {
    pub val: i32,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct FactionPredictionCard {
    pub faction: Faction,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct TurnPredictionCard {
    pub turn: i32,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub(crate) struct CameraNodes {
    pub main: CameraNode,
    pub shield: CameraNode,
    pub board: CameraNode,
    pub treachery: CameraNode,
    pub traitor: CameraNode,
    pub spice: CameraNode,
    pub storm: CameraNode,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct PredictionNodes {
    pub src: Vec2,
    pub factions: Vec<Vec2>,
    pub turns: Vec<Vec2>,
    pub chosen_faction: Vec2,
    pub chosen_turn: Vec2,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub(crate) struct CameraNode {
    pub pos: Vec3,
    pub at: Vec3,
    pub up: Vec3,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct TokenNodes {
    pub leaders: Vec<Vec3>,
    pub spice: Vec<Vec3>,
    pub fighters: Vec<Vec3>,
    pub factions: Vec<Vec3>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct UiRect {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl UiRect {
    pub fn center(&self) -> Vec2 {
        Vec2::new(
            0.5 * (self.right - self.left) + self.left,
            0.5 * (self.top - self.bottom) + self.bottom,
        )
    }

    pub fn top_left(&self) -> Rect<Val> {
        Rect {
            top: Val::Percent(50.0 * (-self.top + 1.0)),
            left: Val::Percent(50.0 * (self.left + 1.0)),
            ..Default::default()
        }
    }

    pub fn size(&self) -> Size<Val> {
        Size::new(
            Val::Percent(50.0 * (self.right - self.left)),
            Val::Percent(50.0 * (self.top - self.bottom)),
        )
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct UiStructure {
    pub turn_order: UiRect,
    pub phases: UiRect,
    pub turn_counter: UiRect,
}

impl UiStructure {
    pub fn get_turn_tiles(&self) -> Vec<UiRect> {
        let tile_height = (self.turn_order.bottom - self.turn_order.top) / 6.0;
        (0..6)
            .map(|i| UiRect {
                top: self.turn_order.top + i as f32 * tile_height,
                left: self.turn_order.left,
                right: self.turn_order.right,
                bottom: self.turn_order.top + (i as f32 + 1.0) * tile_height,
            })
            .collect()
    }

    pub fn get_phase_bounds(&self) -> Vec<UiRect> {
        let tile_width = (self.turn_order.right - self.turn_order.left) / 6.0;
        (0..6)
            .map(|i| UiRect {
                top: self.turn_order.top,
                left: self.turn_order.left + i as f32 * tile_width,
                right: self.turn_order.right + (i as f32 + 1.0) * tile_width,
                bottom: self.turn_order.bottom,
            })
            .collect()
    }
}
