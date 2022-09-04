mod phases;
mod systems;

use std::{
    collections::{HashMap, HashSet, VecDeque},
    f32::consts::PI,
    hash::Hash,
    mem::Discriminant,
};

use bevy::{
    ecs::system::Resource,
    prelude::*,
    render::camera::{Camera, OrthographicProjection},
};
use rand::prelude::SliceRandom;

use self::{
    phases::{setup::SetupPhase, storm::StormPhase},
    systems::*,
};
use crate::{
    components::{Deck, Disorganized, LocationSector, Player, Prediction, Spice, Storm, Troop, Unique, UniqueBundle},
    lerper::{Lerp, LerpType},
    resources::{Data, Info},
    util::{card_jitter, divide_spice, hand_positions, shuffle_deck},
    Screen,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(State::<Phase>::get_driver());

        app.add_system_set(
            SystemSet::on_update(Screen::Game)
                .with_system(phase_text_system)
                .with_system(public_troop_system)
                .with_system(trigger_stack_troops)
                .with_system(shuffle_system),
        );

        app.add_system_set_to_stage(
            CoreStage::Last,
            SystemSet::on_exit(Screen::Game).with_system(reset_system),
        );
    }
}

#[derive(Component)]
pub struct PhaseText;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Phase {
    Setup(SetupPhase),
    Storm(StormPhase),
    SpiceBlow,
    Nexus,
    Bidding,
    Revival,
    Movement,
    Battle,
    Collection,
    Control,
    EndGame,
}

// impl Phase {
//     pub fn next(&self) -> Self {
//         match self {
//             Phase::Setup(subphase) => match subphase {
//                 SetupPhase::ChooseFactions => Phase::Setup(SetupPhase::Prediction),
//                 SetupPhase::Prediction => Phase::Setup(SetupPhase::AtStart),
//                 SetupPhase::AtStart => Phase::Setup(SetupPhase::DealTraitors),
//                 SetupPhase::DealTraitors => Phase::Setup(SetupPhase::PickTraitors),
//                 SetupPhase::PickTraitors => Phase::Setup(SetupPhase::DealTreachery),
//                 SetupPhase::DealTreachery => Phase::Storm(StormPhase::Reveal),
//             },
//             Phase::Storm(subphase) => match subphase {
//                 StormPhase::Reveal => Phase::Storm(StormPhase::WeatherControl),
//                 StormPhase::WeatherControl => Phase::Storm(StormPhase::FamilyAtomics),
//                 StormPhase::FamilyAtomics => Phase::Storm(StormPhase::MoveStorm),
//                 StormPhase::MoveStorm => Phase::SpiceBlow,
//             },
//             Phase::SpiceBlow => Phase::Nexus,
//             Phase::Nexus => Phase::Bidding,
//             Phase::Bidding => Phase::Revival,
//             Phase::Revival => Phase::Movement,
//             Phase::Movement => Phase::Battle,
//             Phase::Battle => Phase::Collection,
//             Phase::Collection => Phase::Control,
//             Phase::Control => Phase::Storm(StormPhase::Reveal),
//             Phase::EndGame => Phase::EndGame,
//         }
//     }
// }

fn init_game(mut commands: Commands) {
    commands.insert_resource(State::new(Phase::Setup(SetupPhase::ChooseFactions)));
}

fn reset_system() {
    todo!()
}

#[derive(Component)]
pub struct Shuffling(pub usize);

pub fn init_shuffle_decks(mut commands: Commands, decks: Query<Entity, With<Deck>>) {
    println!("Enter: shuffle_decks");
    for deck in decks.iter() {
        commands.entity(deck).insert(Shuffling(5));
    }
}

pub fn shuffle_system(
    mut commands: Commands,
    mut decks: Query<(Entity, &mut Deck, &Children, &mut Shuffling)>,
    lerps: Query<&Lerp>,
) {
    let mut rng = rand::thread_rng();
    for (e, mut deck, children, mut shuffling) in decks.iter_mut() {
        if children.iter().any(|c| lerps.get(*c).is_ok()) {
            shuffling.0 -= 1;
            if shuffling.0 == 0 {
                commands.entity(e).remove::<Shuffling>();
            }
            continue;
        }
        let mut cards = children.iter().enumerate().collect::<Vec<_>>();
        cards.shuffle(&mut rng);
        deck.0 = cards.iter().map(|(_, e)| **e).collect();
        for (i, card) in cards {
            let transform = Transform::from_translation(Vec3::Y * 0.001 * (i as f32)) * card_jitter();
            commands
                .entity(*card)
                .insert(Lerp::new(LerpType::world_to(transform), 0.2, 0.0));
        }
    }
}
