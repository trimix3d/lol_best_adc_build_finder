pub mod items;

use crate::OnActionFns;

use super::*;
use units_data::{UnitStats, MAX_UNIT_ITEMS};

use enumset::{EnumSet, EnumSetType};
#[allow(unused_imports)]
use strum::EnumCount; //this import is necessary for strum_macros::EnumCount to work but it triggers the lint for some reason
use strum_macros::EnumCount as EnumCountMacro;

use core::cmp::Ordering;
use core::fmt;
use core::ops::{Deref, DerefMut};

/// Holds every item id (or name).
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, EnumCountMacro)]
pub(crate) enum ItemId {
    NullItem,
    AbyssalMask,
    AxiomArc,
    BansheesVeil,
    BlackCleaver,
    BlackfireTorch,
    BladeOfTheRuinedKing,
    Bloodthirster,
    ChempunkChainsword,
    CosmicDrive,
    Cryptbloom,
    DeadMansPlate,
    DeathsDance,
    Eclipse,
    EdgeOfNight,
    EssenceReaver,
    ExperimentalHexplate,
    FrozenHeart,
    GuardianAngel,
    GuinsoosRageblade,
    HextechRocketbelt,
    HorizonFocus,
    Hubris,
    Hullbreaker,
    IcebornGauntlet,
    ImmortalShieldbow,
    InfinityEdge,
    Jaksho,
    KaenicRookern,
    KrakenSlayer,
    LiandrysTorment,
    LichBane,
    LordDominiksRegards,
    LudensCompanion,
    Malignance,
    MawOfMalmortius,
    MercurialScimitar,
    Morellonomicon,
    MortalReminder,
    Muramana,
    NashorsTooth,
    NavoriFlickerblade,
    Opportunity,
    OverlordsBloodmail,
    PhantomDancer,
    ProfaneHydra,
    RabadonsDeathcap,
    RanduinsOmen,
    RapidFirecannon,
    RavenousHydra,
    Riftmaker,
    RodOfAges,
    RunaansHurricane,
    RylaisCrystalScepter,
    SerpentsFang,
    SeraphsEmbrace,
    SeryldasGrudge,
    Shadowflame,
    SpearOfShojin,
    StatikkShiv,
    SteraksGage,
    Stormsurge,
    Stridebreaker,
    SunderedSky,
    Terminus,
    TheCollector,
    TitanicHydra,
    TrinityForce,
    UmbralGlaive,
    VoidStaff,
    VoltaicCyclosword,
    WitsEnd,
    YoumuusGhostblade,
    YunTalWildarrows,
    ZhonyasHourglass,
    BerserkersGreaves,
    BootsOfSwiftness,
    IonianBootsOfLucidity,
    MercurysTreads,
    PlatedSteelcaps,
    SorcerersShoes,
}

/// Holds item groups of an item, an item can have multiple item groups (implemented using an `EnumSet`).
///
/// A build cannot have multiple items of the same item group.
/// <https://leagueoflegends.fandom.com/wiki/Item_group>
#[derive(EnumSetType, Debug)]
pub enum ItemGroups {
    Annul,
    Blight,
    Boots,
    Eternity,
    Fatality,
    Hydra,
    Immolate,
    Support,
    Lifeline,
    Manaflow,
    Momentum,
    Quicksilver,
    Spellblade,
    Stasis,
}

/// Describe if item has specific actives/passives utilities.
///
/// An item can have a variant only for important effects like big powerspikes or one that unlocks a win condition.
#[derive(EnumSetType, Debug)]
pub enum ItemUtils {
    AntiHealShield,
    Survivability,
    Special,
}

#[derive(Debug)]
pub struct Item {
    //attributes
    id: ItemId,
    pub full_name: &'static str,
    pub short_name: &'static str,
    pub cost: f32, //f32 because exclusively used in f32 calculations
    pub item_groups: EnumSet<ItemGroups>,
    pub utils: EnumSet<ItemUtils>,

    //stats
    pub stats: UnitStats,

    //on action fns (passives/actives)
    pub(crate) on_action_fns: OnActionFns,
}

//no impl Default for Item because they are compile time constants and can't use non-constant functions

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.write_str(self.full_name)?;
            if self.full_name != self.short_name {
                f.write_str(" (")?;
                f.write_str(self.short_name)?;
                f.write_str(")")?;
            }
        } else {
            f.write_str(self.short_name)?;
        }
        Ok(())
    }
}

//we use the Eq trait to compare items with their id for fast comparison,
//this imply that there should be no id collision for any item.
//This requirement is checked at the start of the main function of the program (if i didn't delete this out of profound retardation)
impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Item {}

//we use the Ord trait to compare items with their id for fast comparison.
impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Item {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

/// Lists all (non-support and non-boots) items.
pub const ALL_LEGENDARY_ITEMS: [&Item; 74] = [
    &Item::ABYSSAL_MASK,
    &Item::AXIOM_ARC,
    &Item::BANSHEES_VEIL,
    &Item::BLACK_CLEAVER,
    &Item::BLACKFIRE_TORCH,
    &Item::BLADE_OF_THE_RUINED_KING,
    &Item::BLOODTHIRSTER,
    &Item::CHEMPUNK_CHAINSWORD,
    &Item::COSMIC_DRIVE,
    &Item::CRYPTBLOOM,
    &Item::DEAD_MANS_PLATE,
    &Item::DEATHS_DANCE,
    &Item::ECLIPSE,
    &Item::EDGE_OF_NIGHT,
    &Item::ESSENCE_REAVER,
    &Item::EXPERIMENTAL_HEXPLATE,
    &Item::FROZEN_HEART,
    &Item::GUARDIAN_ANGEL,
    &Item::GUINSOOS_RAGEBLADE,
    &Item::HEXTECH_ROCKETBELT,
    &Item::HORIZON_FOCUS,
    &Item::HUBRIS,
    &Item::HULLBREAKER,
    &Item::ICEBORN_GAUNTLET,
    &Item::IMMORTAL_SHIELDBOW,
    &Item::INFINITY_EDGE,
    &Item::JAKSHO,
    &Item::KAENIC_ROOKERN,
    &Item::KRAKEN_SLAYER,
    &Item::LIANDRYS_TORMENT,
    &Item::LICH_BANE,
    &Item::LORD_DOMINIKS_REGARDS,
    &Item::LUDENS_COMPANION,
    &Item::MALIGNANCE,
    &Item::MAW_OF_MALMORTIUS,
    &Item::MERCURIAL_SCIMITAR,
    &Item::MORELLONOMICON,
    &Item::MORTAL_REMINDER,
    &Item::MURAMANA,
    &Item::NASHORS_TOOTH,
    &Item::NAVORI_FLICKERBLADE,
    &Item::OPPORTUNITY,
    &Item::OVERLORDS_BLOODMAIL,
    &Item::PHANTOM_DANCER,
    &Item::PROFANE_HYDRA,
    &Item::RABADONS_DEATHCAP,
    &Item::RANDUINS_OMEN,
    &Item::RAPID_FIRECANNON,
    &Item::RAVENOUS_HYDRA,
    &Item::RIFTMAKER,
    &Item::ROD_OF_AGES,
    &Item::RUNAANS_HURRICANE,
    &Item::RYLAIS_CRYSTAL_SCEPTER,
    &Item::SERAPHS_EMBRACE,
    &Item::SERPENTS_FANG,
    &Item::SERYLDAS_GRUDGE,
    &Item::SHADOWFLAME,
    &Item::SPEAR_OF_SHOJIN,
    &Item::STATIKK_SHIV,
    &Item::STERAKS_GAGE,
    &Item::STORMSURGE,
    &Item::STRIDEBREAKER,
    &Item::SUNDERED_SKY,
    &Item::TERMINUS,
    &Item::THE_COLLECTOR,
    &Item::TITANIC_HYDRA,
    &Item::TRINITY_FORCE,
    &Item::UMBRAL_GLAIVE,
    &Item::VOID_STAFF,
    &Item::VOLTAIC_CYCLOSWORD,
    &Item::WITS_END,
    &Item::YOUMUUS_GHOSTBLADE,
    &Item::YUN_TAL_WILDARROWS,
    &Item::ZHONYAS_HOURGLASS,
];

/// Lists all boots.
pub const ALL_BOOTS: [&Item; 6] = [
    &Item::BERSERKERS_GREAVES,
    &Item::BOOTS_OF_SWIFTNESS,
    &Item::IONIAN_BOOTS_OF_LUCIDITY,
    &Item::MERCURYS_TREADS,
    &Item::PLATED_STEELCAPS,
    &Item::SORCERERS_SHOES,
];

/// Lists support items.
pub const ALL_SUPP_ITEMS: [&Item; 0] = [];

//set manually because f32 calcs are forbidden in constants :)))
pub const AVG_LEGENDARY_ITEM_COST: f32 = 2979.;
pub const AVG_BOOTS_COST: f32 = 1100.;
pub const AVG_SUPP_ITEM_COST: f32 = 0.;

/// Amount of experience gained farming for the average legendary item.
/// We approximate that the gold income is only from cs golds and passive golds generation.
pub const XP_PER_LEGENDARY_ITEM: f32 =
    AVG_XP_PER_CS * CS_PER_MIN * AVG_LEGENDARY_ITEM_COST / TOT_GOLDS_PER_MIN;
/// Amount of experience gained farming for the average boots item.
/// We approximate that the gold income is only from cs golds and passive golds generation.
pub const XP_PER_BOOTS_ITEM: f32 = AVG_XP_PER_CS * CS_PER_MIN * AVG_BOOTS_COST / TOT_GOLDS_PER_MIN;
/// Amount of experience gained farming for the average support item.
/// We approximate that the gold income is only from cs golds and passive golds generation.
pub const XP_PER_SUPP_ITEM: f32 =
    AVG_XP_PER_CS * CS_PER_MIN * AVG_SUPP_ITEM_COST / TOT_GOLDS_PER_MIN;

/// Assumes 1 build slot for boots and the remaining slots for legendary items.
#[allow(clippy::cast_precision_loss)] //`MAX_UNIT_ITEMS` is well whithin f32's range to avoid precision loss
pub const AVG_ITEM_COST_WITH_BOOTS: f32 = (((MAX_UNIT_ITEMS - 1) as f32) * AVG_LEGENDARY_ITEM_COST
    + AVG_BOOTS_COST)
    / (MAX_UNIT_ITEMS as f32);
/// Assumes 1 build slot for support item, 1 for boots and the remaining slots for legendary items.
//pub const AVG_ITEM_COST_WITH_BOOTS_AND_SUPP_ITEM: f32 =
//    ((MAX_UNIT_ITEMS_F32 - 2.) * AVG_LEGENDARY_ITEM_COST + AVG_BOOTS_COST + AVG_SUPP_ITEM_COST)
//        / MAX_UNIT_ITEMS_F32;

#[derive(Debug, Clone, Copy)]
pub struct Build(pub [&'static Item; MAX_UNIT_ITEMS]);
pub(crate) type BuildHash = [ItemId; MAX_UNIT_ITEMS];

impl Deref for Build {
    type Target = [&'static Item; MAX_UNIT_ITEMS];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Build {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for Build {
    /// Returns an empty build (filled with `NULL_ITEM`).
    fn default() -> Self {
        Build([&Item::NULL_ITEM; MAX_UNIT_ITEMS])
    }
}

impl fmt::Display for Build {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[")?;
        f.write_str(self[0].short_name)?;
        for item in &self[1..] {
            f.write_str(", ")?;
            f.write_str(item.short_name)?;
        }
        f.write_str("]")
    }
}

impl Build {
    /// Returns the item count in the build (ignoring `NULL_ITEMs`).
    #[must_use]
    pub fn item_count(&self) -> usize {
        let mut item_count: usize = 0;
        for item in self.iter().copied() {
            if *item != Item::NULL_ITEM {
                item_count += 1;
            }
        }
        item_count
    }

    /// Returns the build cost
    #[must_use]
    pub fn cost(&self) -> f32 {
        self.iter().fold(0., |acc, item| acc + item.cost)
    }

    /// Returns the build hash. Builds with same items but in different item order will produce the same hash.
    /// If there is no id collision between items, this function doesn't produces collisions either
    #[must_use]
    pub(crate) fn get_hash(&self) -> BuildHash {
        let mut ids: [ItemId; MAX_UNIT_ITEMS] = [
            self[0].id, self[1].id, self[2].id, self[3].id, self[4].id, self[5].id,
        ];
        ids.sort_unstable();
        ids
    }

    pub fn check_validity(&self) -> Result<(), String> {
        //ids being the same as the build hash is a coincidence, the method to compute the hash may change in the future
        let mut ids: [ItemId; MAX_UNIT_ITEMS] = [
            self[0].id, self[1].id, self[2].id, self[3].id, self[4].id, self[5].id,
        ];
        ids.sort_unstable();
        for window in ids.windows(2) {
            if window[0] == window[1] && window[0] != Item::NULL_ITEM.id {
                return Err(format!("Duplicates in build: {:?}", window[0]));
            }
        }
        if self.has_item_groups_overlap() {
            return Err("Item group overlap in build".to_string());
        }
        Ok(())
    }

    #[must_use]
    pub fn has_item_groups_overlap(&self) -> bool {
        let mut cum_item_groups: EnumSet<ItemGroups> = self[0].item_groups;
        for item in self[1..].iter() {
            if !((cum_item_groups & item.item_groups).is_empty()) {
                return true;
            }
            cum_item_groups |= item.item_groups;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    use super::*;

    use constcat::concat_slices;

    const ALL_ITEMS_PLUS_NULL_ITEM: &[&Item] = concat_slices!(
        [&'static Item]:
        &ALL_LEGENDARY_ITEMS,
        &ALL_BOOTS,
        &ALL_SUPP_ITEMS,
        &[&Item::NULL_ITEM],
    );

    /// Accepted difference between the real computed average item cost and the one entered manually when testing.
    const ITEMS_AVG_COST_TOL: f32 = 1.;

    /// Check that there isn't any id collisions in any items of the crate.
    /// Panics if a collision is encountered.
    /// The program won't run correctly if there are collisions between item ids.
    #[test]
    pub fn test_items_dupes_and_id_collisions() {
        //get ids and sort them
        let mut items_ids: Vec<ItemId> = ALL_ITEMS_PLUS_NULL_ITEM
            .iter()
            .map(|item| item.id)
            .collect();

        if let Some(id) = crate::find_dupes_in_slice(&mut items_ids) {
            panic!("Item id collision encountered: {:?}", id)
        }
    }

    #[test]
    pub fn test_all_items_are_correctly_listed() {
        assert!(
            ALL_ITEMS_PLUS_NULL_ITEM.len() == ItemId::COUNT,
            "Number of items in `ALL_ITEMS` ({}) is different that the number of variants in `ItemId` enum ({})",
            ALL_ITEMS_PLUS_NULL_ITEM.len(),
            ItemId::COUNT
        );
    }

    #[test]
    pub fn test_average_legendary_item_cost() {
        let true_legendary_avg: f32 = ALL_LEGENDARY_ITEMS
            .iter()
            .map(|item| item.cost)
            .sum::<f32>()
            / (ALL_LEGENDARY_ITEMS.len() as f32);

        assert!(((AVG_LEGENDARY_ITEM_COST) - true_legendary_avg).abs() < ITEMS_AVG_COST_TOL,
            "Constant `AVG_LEGENDARY_ITEM_COST` of value {} is too far from the true average legendary item cost of {} (-> put its value to {:.0})",
            AVG_LEGENDARY_ITEM_COST,
            true_legendary_avg,
            true_legendary_avg
        );
    }

    #[test]
    pub fn test_average_boots_cost() {
        let true_boots_avg: f32 =
            ALL_BOOTS.iter().map(|item| item.cost).sum::<f32>() / (ALL_BOOTS.len() as f32);

        assert!(((AVG_BOOTS_COST) - true_boots_avg).abs() < ITEMS_AVG_COST_TOL,
            "Constant `AVG_BOOTS_COST` of value {} is too far from the true average boots cost of {} (-> put its value to {:.0})",
            AVG_BOOTS_COST,
            true_boots_avg,
            true_boots_avg
        );
    }

    #[test]
    pub fn test_average_supp_item_cost() {
        let true_supp_avg: f32 = ALL_SUPP_ITEMS.iter().map(|item| item.cost).sum::<f32>()
            / (ALL_SUPP_ITEMS.len() as f32);

        assert!(((AVG_SUPP_ITEM_COST) - true_supp_avg).abs() < ITEMS_AVG_COST_TOL,
            "Constant `AVG_SUPP_ITEM_COST` of value {} is too far from the true average boots cost of {} (-> put its value to {:.0})",
            AVG_SUPP_ITEM_COST,
            true_supp_avg,
            true_supp_avg
        );
    }
}
