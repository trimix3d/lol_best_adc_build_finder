pub mod items;

use crate::OnActionFns;

use super::*;
use units_data::{UnitStats, MAX_UNIT_ITEMS};

use constcat::concat_slices;
use enumset::{EnumSet, EnumSetType};
#[allow(unused_imports)]
use strum::EnumCount; //this import is necessary for strum_macros::EnumCount to work but it triggers the lint for some reason
use strum_macros::EnumCount as EnumCountMacro;

use core::cmp::Ordering;
use core::fmt;
use core::ops::{Deref, DerefMut};

use items::*;

/// Holds every item id (or name).
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, EnumCountMacro)]
pub enum ItemId {
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
    pub id: ItemId,
    pub full_name: &'static str,
    pub short_name: &'static str,
    pub cost: f32, //f32 because exclusively used in f32 calculations
    pub item_groups: EnumSet<ItemGroups>,
    pub utils: EnumSet<ItemUtils>,

    //stats
    pub stats: UnitStats,

    //on action fns (passives/actives)
    pub on_action_fns: OnActionFns,
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
    &ABYSSAL_MASK,
    &AXIOM_ARC,
    &BANSHEES_VEIL,
    &BLACK_CLEAVER,
    &BLACKFIRE_TORCH,
    &BLADE_OF_THE_RUINED_KING,
    &BLOODTHIRSTER,
    &CHEMPUNK_CHAINSWORD,
    &COSMIC_DRIVE,
    &CRYPTBLOOM,
    &DEAD_MANS_PLATE,
    &DEATHS_DANCE,
    &ECLIPSE,
    &EDGE_OF_NIGHT,
    &ESSENCE_REAVER,
    &EXPERIMENTAL_HEXPLATE,
    &FROZEN_HEART,
    &GUARDIAN_ANGEL,
    &GUINSOOS_RAGEBLADE,
    &HEXTECH_ROCKETBELT,
    &HORIZON_FOCUS,
    &HUBRIS,
    &HULLBREAKER,
    &ICEBORN_GAUNTLET,
    &IMMORTAL_SHIELDBOW,
    &INFINITY_EDGE,
    &JAKSHO,
    &KAENIC_ROOKERN,
    &KRAKEN_SLAYER,
    &LIANDRYS_TORMENT,
    &LICH_BANE,
    &LORD_DOMINIKS_REGARDS,
    &LUDENS_COMPANION,
    &MALIGNANCE,
    &MAW_OF_MALMORTIUS,
    &MERCURIAL_SCIMITAR,
    &MORELLONOMICON,
    &MORTAL_REMINDER,
    &MURAMANA,
    &NASHORS_TOOTH,
    &NAVORI_FLICKERBLADE,
    &OPPORTUNITY,
    &OVERLORDS_BLOODMAIL,
    &PHANTOM_DANCER,
    &PROFANE_HYDRA,
    &RABADONS_DEATHCAP,
    &RANDUINS_OMEN,
    &RAPID_FIRECANNON,
    &RAVENOUS_HYDRA,
    &RIFTMAKER,
    &ROD_OF_AGES,
    &RUNAANS_HURRICANE,
    &RYLAIS_CRYSTAL_SCEPTER,
    &SERAPHS_EMBRACE,
    &SERPENTS_FANG,
    &SERYLDAS_GRUDGE,
    &SHADOWFLAME,
    &SPEAR_OF_SHOJIN,
    &STATIKK_SHIV,
    &STERAKS_GAGE,
    &STORMSURGE,
    &STRIDEBREAKER,
    &SUNDERED_SKY,
    &TERMINUS,
    &THE_COLLECTOR,
    &TITANIC_HYDRA,
    &TRINITY_FORCE,
    &UMBRAL_GLAIVE,
    &VOID_STAFF,
    &VOLTAIC_CYCLOSWORD,
    &WITS_END,
    &YOUMUUS_GHOSTBLADE,
    &YUN_TAL_WILDARROWS,
    &ZHONYAS_HOURGLASS,
];

/// Lists all boots.
pub const ALL_BOOTS: [&Item; 6] = [
    &BERSERKERS_GREAVES,
    &BOOTS_OF_SWIFTNESS,
    &IONIAN_BOOTS_OF_LUCIDITY,
    &MERCURYS_TREADS,
    &PLATED_STEELCAPS,
    &SORCERERS_SHOES,
];

/// Lists support items.
pub const ALL_SUPPORT_ITEMS: [&Item; 0] = [];

pub const ALL_ITEMS: &[&Item] = concat_slices!(
    [&'static Item]:
    &ALL_LEGENDARY_ITEMS,
    &ALL_BOOTS,
    &ALL_SUPPORT_ITEMS
);

pub const AVG_LEGENDARY_ITEM_COST: f32 = 2994.;
pub const AVG_BOOTS_COST: f32 = 1100.;
pub const AVG_SUPPORT_ITEM_COST: f32 = 0.;

#[allow(clippy::cast_precision_loss)]
const MAX_UNIT_ITEMS_F32: f32 = MAX_UNIT_ITEMS as f32; //`MAX_UNIT_ITEMS` is well whithin f32's range to avoid precision loss

/// Assumes 1 build slot for boots and the remaining slots for legendary items.
pub const AVG_ITEM_COST_WITH_BOOTS: f32 =
    ((MAX_UNIT_ITEMS_F32 - 1.) * AVG_LEGENDARY_ITEM_COST + AVG_BOOTS_COST) / MAX_UNIT_ITEMS_F32;
/// Assumes 1 build slot for support item, 1 for boots and the remaining slots for legendary items.
pub const AVG_ITEM_COST_WITH_BOOTS_AND_SUPP_ITEM: f32 =
    ((MAX_UNIT_ITEMS_F32 - 2.) * AVG_LEGENDARY_ITEM_COST + AVG_BOOTS_COST + AVG_SUPPORT_ITEM_COST)
        / MAX_UNIT_ITEMS_F32;

#[derive(Debug, Clone, Copy)]
pub struct Build(pub [&'static Item; MAX_UNIT_ITEMS]);

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
        Build([&NULL_ITEM; MAX_UNIT_ITEMS])
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

pub type BuildHash = [ItemId; MAX_UNIT_ITEMS];

impl Build {
    /// Returns the item count in the build (ignoring `NULL_ITEMs`).
    #[must_use]
    pub fn item_count(&self) -> usize {
        let mut item_count: usize = 0;
        for &item in self.iter() {
            if *item != NULL_ITEM {
                item_count += 1;
            }
        }
        item_count
    }

    /// Returns the build cost
    #[must_use]
    pub fn cost(&self) -> f32 {
        self.iter().fold(0., |acc, &item| acc + item.cost)
    }

    /// Returns the build hash. Builds with same items but in different item order will produce the same hash.
    /// If there is no id collision between items, this function doesn't produces collisions either
    /// (except for the case above which is intended).
    #[must_use]
    pub fn get_hash(&self) -> BuildHash {
        let mut ids: [ItemId; MAX_UNIT_ITEMS] = [
            self[0].id, self[1].id, self[2].id, self[3].id, self[4].id, self[5].id,
        ];
        ids.sort_unstable();
        ids
    }

    pub fn check_validity(&self) -> Result<(), String> {
        let mut ids: [ItemId; MAX_UNIT_ITEMS] = [
            self[0].id, self[1].id, self[2].id, self[3].id, self[4].id, self[5].id,
        ];
        ids.sort_unstable();
        for window in ids.windows(2) {
            if window[0] == window[1] && window[0] != NULL_ITEM.id {
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
        for item in self[1..].iter().filter(|&&item| *item != NULL_ITEM) {
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
    use items::NULL_ITEM;

    /// Check that there isn't any id collisions in any items of the crate.
    /// Panics if a collision is encountered.
    /// The program won't run correctly if there are collisions between item ids.
    #[test]
    pub fn test_items_ids_collisions() {
        //create Vec of all items (don't forget NULL_ITEM)
        let all_items: Vec<&'static Item> = [
            &ALL_LEGENDARY_ITEMS[..],
            &ALL_BOOTS[..],
            &ALL_SUPPORT_ITEMS[..],
            &[&NULL_ITEM][..],
        ]
        .concat();

        //get ids and sort them
        let mut items_ids: Vec<ItemId> = all_items.iter().map(|item| item.id).collect();
        items_ids.sort_unstable();

        //compare adjacent elements of sorted vec to find id collisions
        for window in items_ids.windows(2) {
            if window[0] == window[1] {
                panic!("Item id collision encountered: {:?}", window[0])
            }
        }
    }

    #[test]
    pub fn test_average_legendary_item_cost() {
        let true_legendary_avg: f32 = ALL_LEGENDARY_ITEMS
            .iter()
            .map(|item| item.cost)
            .sum::<f32>()
            / (ALL_LEGENDARY_ITEMS.len() as f32);

        assert!(((AVG_LEGENDARY_ITEM_COST) - true_legendary_avg).abs() < 1.,
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

        assert!(((AVG_BOOTS_COST) - true_boots_avg).abs() < 1.,
            "Constant `AVG_BOOTS_COST` of value {} is too far from the true average boots cost of {} (-> put its value to {:.0})",
            AVG_BOOTS_COST,
            true_boots_avg,
            true_boots_avg
        );
    }

    #[test]
    pub fn test_average_support_item_cost() {
        let true_support_avg: f32 =
            ALL_SUPPORT_ITEMS.iter().map(|item| item.cost).sum::<f32>() / (ALL_BOOTS.len() as f32);

        assert!(((AVG_SUPPORT_ITEM_COST) - true_support_avg).abs() < 1.,
            "Constant `AVG_SUPPORT_ITEM_COST` of value {} is too far from the true average boots cost of {} (-> put its value to {:.0})",
            AVG_SUPPORT_ITEM_COST,
            true_support_avg,
            true_support_avg
        );
    }

    #[test]
    pub fn test_all_items_are_correctly_listed() {
        assert!(
            ALL_ITEMS.len() + 1 == ItemId::COUNT, //+1 to account for `NULL_ITEM`
            "Number of items in `ALL_ITEMS` ({} + 1 for `NULL_ITEM`) is different that the number of variants in `ItemId` enum ({})",
            ALL_ITEMS.len(),
            ItemId::COUNT
        );
    }
}
