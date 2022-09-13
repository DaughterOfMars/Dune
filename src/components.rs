use std::fmt::Display;

use bevy::{math::Vec3, prelude::Component};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(Copy, Clone, Component)]
pub struct Spice {
    pub value: i32,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Component)]
pub struct Troop {
    pub is_special: bool,
}

#[derive(Default, Component)]
pub struct Storm {
    pub sector: i32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Component, Serialize, Deserialize)]
pub struct LocationSector {
    pub location: Location,
    pub sector: u8,
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

#[derive(Component)]
pub struct CanRespond;

#[derive(Clone, Component)]
pub struct StormCardValue {
    pub val: u8,
}

#[derive(Clone, Component)]
pub struct TurnPredictionCard {
    pub turn: u8,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Hash, Display, Component, EnumIter)]
pub enum Faction {
    Atreides,
    Harkonnen,
    Emperor,
    SpacingGuild,
    Fremen,
    BeneGesserit,
}

impl Faction {
    pub fn code(&self) -> &str {
        match self {
            Self::Atreides => "at",
            Self::Harkonnen => "hk",
            Self::Emperor => "em",
            Self::SpacingGuild => "sg",
            Self::Fremen => "fr",
            Self::BeneGesserit => "bg",
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Hash, Display, Component, EnumIter)]
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

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Hash, Display, Component, EnumIter)]
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

impl Location {
    pub fn with_sector(self, sector: u8) -> LocationSector {
        LocationSector { location: self, sector }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Display, Hash, Component)]
pub enum Terrain {
    Sand,
    Rock,
    Stronghold,
    PolarSink,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Display, Hash, Component)]
pub enum Bonus {
    Carryalls,
    Ornothopters,
    Smugglers,
    Harvesters,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Display, Hash, Component)]
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

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Display, Hash)]
pub enum TreacheryCardKind {
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
pub struct TreacheryCard {
    pub kind: TreacheryCardKind,
    pub variant: usize,
}

impl Display for TreacheryCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "kind: {}, variant: {}", self.kind, self.variant)
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Display, Hash, Component)]
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

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Display, Hash, Component)]
pub struct FactionPredictionCard {
    pub faction: Faction,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Display, Hash, Component)]
pub struct FactionChoiceCard {
    pub faction: Faction,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Hash, Display, Component)]
pub struct TraitorCard {
    pub leader: Leader,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Display, Hash, Component)]
pub struct StormCard {
    pub val: u8,
}

#[derive(Component)]
pub struct TraitorDeck;

#[derive(Component)]
pub struct TreacheryDeck;

#[derive(Component)]
pub struct StormDeck;

#[derive(Component)]
pub struct SpiceDeck;
