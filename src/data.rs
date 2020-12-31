use bevy::math::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Faction {
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
            Self::SpacingGuild => (5, Some(vec!["Tueks Sietch".to_string()]), 5),
            Self::Harkonnen => (10, Some(vec!["Carthag".to_string()]), 10),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Leader {
    pub name: String,
    pub power: i32,
    pub faction: Faction,
    pub texture: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Location {
    pub name: String,
    pub terrain: Terrain,
    pub verts: Vec<(f32, f32)>
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum Terrain {
    Sand,
    Rock,
    Stronghold,
    PolarSink,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum Bonus {
    Carryalls,
    Ornothopters,
    Smugglers,
    Harvesters,
}
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum Effect {
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

impl Effect {
    fn description(&self) -> String {
        match self {
            Effect::Worthless => 
                "Play as part of your Battle Plan, in place of weapon, defense, or both.
                This card has no value in play, and you can discard it only by playing it in your Battle Plan.".to_string(),
            Effect::PoisonWeapon => 
                "Play as part of your Battle Plan.
                Kills enemy leader before the battle is resolved.
                May be countered by an appropriate defense (Snooper).
                You may keep this card if you win in this battle.".to_string(),
            Effect::ProjectileWeapon => 
                "Play as part of your Battle Plan.
                Kills enemy leader before the battle is resolved.
                May be countered by an appropriate defense (Shield).
                You may keep this card if you win in this battle.".to_string(),
            Effect::CheapHero => 
                "Play as a leader with zero strength on your Battle Plan.
                (leader allows you to play 1 weapon & 1 defense card on Battle Plan)
                Can be played together with another leader, allowing you to return & save that leader immediately after both Battle Plans were revealed.".to_string(),
            Effect::PoisonDefense => 
                "Play as part of your Battle Plan.
                Protects your leader from enemy poison weapon in this battle.
                You may keep this card if you win in this battle.".to_string(),
            Effect::ProjectileDefense => 
                "Play as part of your Battle Plan.
                Protects your leader from enemy projectile weapon in this battle.
                You may keep this card if you win in this battle.".to_string(),
            Effect::Atomics =>
                "Play after the storm movement is calulated by before storm is moved, and only if you have token(s) on the Shield Wall or an adjacent territory.
                All tokens in the Shield Wall are destroyed. Arrakeen, Imperial Basin & Carthag are no longer protected from the storm for the rest of the game.".to_string(),
            Effect::Movement => 
                "Play during Movement round.
                Take an additional on-planet token movement subject to normal movement rules.
                This may be the same or another group of your tokens.".to_string(),
            Effect::Karama => 
                "You may play this cartd to activate a single Karama Power of your choice.".to_string(),
            Effect::Lasgun =>
                "Play as part of your Battle Plan.
                Automatically kills enemy leader regardless of defense card used.
                You may keep this card if you win in this battle.
                If anyone plays a Shield in this battle, and neither leader is a traitor, then all tokens and leaders in this battle's territory are killed. Both players lose this battle.".to_string(),
            Effect::Revive =>
                "Play at any time.
                You may immediately revive 1 of your leaders of up to 5 of your tokens from the tanks to your reserves at no cost in spice.
                Does not count against per-turn limits on revivals.".to_string(),
            Effect::Truthtrance =>
                "Ask one other player a single yes/no question about the game which must be answered publicly.
                No game or rule effect may interrupt the answer being given.
                The player must answer 'yes' or 'no' truthfully.".to_string(),
            Effect::WeatherControl =>
                "Play at the start of the Storm round, before the storm movement is calulated.
                You control the storm this round and may move it from 0 to 10 sectors in a counterclockwise direction.".to_string()
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TreacheryCard {
    pub id: i32,
    pub effect: Effect,
    pub name: String,
    pub texture: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SpiceCard {
    pub name: String,
    pub texture: String,
}

pub struct StormCard {
    pub val: i32
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct CameraNodes {
    pub main: CameraNode,
    pub shield: CameraNode,
    pub board: CameraNode,
    pub spice: CameraNode,
    pub storm: CameraNode,
}
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct CameraNode {
    pub pos: Vec3,
    pub at: Vec3,
    pub up: Vec3,
}