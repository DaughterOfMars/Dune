use bevy::{math::vec3, prelude::*};
use bevy_mod_picking::PickableBundle;
use rand::prelude::IteratorRandom;

use crate::{
    components::{
        Active, Faction, FactionPredictionCard, Player, Prediction, Spice, Troop, TurnPredictionCard, Unique,
    },
    game::Phase,
    resources::{Data, Info},
    util::divide_spice,
    GameEntity, Screen,
};

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        todo!()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SetupPhase {
    ChooseFactions,
    Prediction,
    AtStart,
    DealTraitors,
    PickTraitors,
    DealTreachery,
}

// TODO:
// - Trigger animate-in for faction cards
// - At end of animation, add Pickable component to faction cards
// - Create event when Pickable faction card is clicked
// - Add system to detect these events and add Faction component to player, pass to next player
// (maybe use Active component? Or store it in a resource since there will only ever be one active player?
// what about players that aren't "Active" but can still take actions? Active player system?)

pub fn pick_factions(
    mut commands: Commands,
    mut info: ResMut<Info>,
    data: Res<Data>,
    mut phase: ResMut<State<SetupPhase>>,
    to_pick: Query<(Entity, &Player), (With<Active>, Without<Faction>)>,
    picked: Query<&Faction, With<Player>>,
) {
    // TODO: pick using events
    let factions = vec![
        Faction::Atreides,
        Faction::BeneGesserit,
        Faction::Emperor,
        Faction::Fremen,
        Faction::Harkonnen,
        Faction::SpacingGuild,
    ]
    .into_iter()
    .filter(|f| !picked.iter().any(|p| p == f));

    let mut rng = rand::thread_rng();

    let faction = factions.choose(&mut rng).unwrap();
    let mut e = commands.spawn_bundle((faction, GameEntity));

    if faction == Faction::BeneGesserit {
        e.insert(Prediction {
            faction: None,
            turn: None,
        });
    }
}
