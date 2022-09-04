use bevy::{
    ecs::{bundle::Bundle, entity::Entity},
    math::Vec3,
    prelude::Component,
    render::view::Visibility,
};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Component)]
pub struct Spice {
    pub value: i32,
}

#[derive(Copy, Clone, Component)]
pub struct Troop {
    pub value: i32,
    pub location: Option<Entity>,
}

#[derive(Default, Component)]
pub struct Storm {
    pub sector: i32,
}

#[derive(Component)]
pub struct LocationSector {
    pub location: Location,
    pub sector: i32,
}

#[derive(Component)]
pub struct Disorganized;

#[derive(Copy, Clone, Debug, Default, Component)]
pub struct SpiceNode {
    pub pos: Vec3,
    pub val: i32,
}

impl SpiceNode {
    pub fn new(pos: Vec3) -> Self {
        Self {
            pos,
            ..Default::default()
        }
    }
}

#[derive(Copy, Clone, Debug, Component)]
pub struct Unique {
    pub faction: Faction,
    pub public: bool,
}

#[derive(Bundle)]
pub struct UniqueBundle {
    unique: Unique,
    visible: Visibility,
}

impl UniqueBundle {
    pub fn new(faction: Faction) -> Self {
        Self {
            unique: Unique { faction, public: false },
            visible: Visibility { is_visible: true },
        }
    }
}

#[derive(Copy, Clone, Default, Debug, Component)]
pub struct Prediction {
    pub faction: Option<Faction>,
    pub turn: Option<i32>,
}

#[derive(Component)]
pub struct Player {
    pub name: String,
    pub traitor_cards: Vec<Entity>,
    pub treachery_cards: Vec<Entity>,
}

impl Player {
    pub fn new(name: String) -> Self {
        Player {
            name,
            traitor_cards: Vec::new(),
            treachery_cards: Vec::new(),
        }
    }
}

#[derive(Component)]
pub struct Deck(pub Vec<Entity>);

#[derive(Component)]
pub struct Card;

#[derive(Component)]
pub struct Active;

#[derive(Component)]
pub struct CanRespond;

#[derive(Clone, Component)]
pub struct StormCardValue {
    pub val: i32,
}

#[derive(Clone, Component)]
pub struct TurnPredictionCard {
    pub turn: i32,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Hash, Component)]
pub enum Faction {
    Atreides,
    Harkonnen,
    Emperor,
    SpacingGuild,
    Fremen,
    BeneGesserit,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Hash, Component)]
pub enum Leader {
    GurneyHalleck,
    ThufirHawat,
    DuncanIdaho,
    LadyJessica,
    DrWellingtonYueh,
    Alia,
    MargotLadyFenring,
    PrincessIrulan,
    MotherRamallo,
    WannaMarcus,
    CaptainAramsham,
    Bashar,
    Burseg,
    Caid,
    HasimirFenring,
    Chani,
    Jamis,
    ShadoutMapes,
    Otheym,
    Stilgar,
    UmmanKudu,
    CaptainIakinNefud,
    BeastRabban,
    FeydRautha,
    PiterDeVries,
    MasterBewt,
    EsmarTuek,
    GuildRep,
    SooSooSook,
    StabanTuek,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Hash, Component)]
pub enum Location {
    Arrakeen,
    Arsunt,
    Basin,
    BightOfTheCliff,
    BrokenLand,
    Carthag,
    CielagoDepression,
    CielagoEast,
    CielagoNorth,
    CielagoSouth,
    CielagoWest,
    FalseWallEast,
    FalseWallSouth,
    FalseWallWest,
    FuneralPlain,
    GaraKulon,
    HabbanyaErg,
    HabbanyaRidgeFlat,
    HabbanyaSietch,
    HaggaBasin,
    HargPass,
    HoleInTheRock,
    ImperialBasin,
    Meridian,
    OldGap,
    PastyMesa,
    PlasticBasin,
    RedChasm,
    RimWallWest,
    RockOutcroppings,
    SihayaRidge,
    ShieldWall,
    SietchTabr,
    SouthMesa,
    TheGreatFlat,
    TheGreaterFlat,
    TheMinorErg,
    Tsimpo,
    TueksSietch,
    WindPassNorth,
    WindPass,
    PolarSink,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Hash, Component)]
pub enum Terrain {
    Sand,
    Rock,
    Stronghold,
    PolarSink,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Hash, Component)]
pub enum Bonus {
    Carryalls,
    Ornothopters,
    Smugglers,
    Harvesters,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Hash, Component)]
pub enum CardEffect {
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

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Hash, Component)]
pub enum TreacheryCard {
    Lasgun,
    Chrysknife,
    MaulaPistol,
    SlipTip,
    Stunner,
    Chaumas,
    Chaumurky,
    EllacaDrug,
    GomJabbar,
    Shield,
    Snooper,
    CheapHero,
    CheapHeroine,
    TleilaxuGhola,
    FamilyAtomics,
    Hajr,
    Karama,
    Truthtrance,
    WeatherControl,
    Baliset,
    JubbaCloak,
    Kulon,
    LaLaLa,
    TripToGamont,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Hash, Component)]
pub enum SpiceCard {
    BrokenLand,
    CielagoNorth,
    CielagoSouth,
    FuneralPlain,
    TheGreatFlat,
    HabbanyaErg,
    HabbanyaRidgeFlat,
    HaggaBasin,
    TheMinorErg,
    OldGap,
    RedChasm,
    RockOutcroppings,
    SihayaRidge,
    SouthMesa,
    WindPassNorth,
    ShaiHalud,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Hash, Component)]
pub struct FactionPredictionCard {
    pub faction: Faction,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Hash, Component)]
pub struct TraitorCard {
    pub leader: Leader,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Hash, Component)]
pub struct StormCard {
    pub val: u8,
}
