use super::game_data::*;

use items_data::{items::NULL_ITEM, *};
use runes_data::{RuneShard, RunesPage};
use units_data::*;

use enumset::{enum_set, EnumSet};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use rustc_hash::{FxBuildHasher, FxHashMap};

use std::iter::zip;
use std::num::{NonZero, NonZeroU8};
use std::time::Duration;

/// Meaningless to go above this value, also computation times may become very long.
pub const MAX_FIGHT_DURATION: f32 = 3600.;
/// Value under which results may become inaccurate and that is not recommended to use.
pub const LOW_FIGHT_DURATION_VALUE_WARNING: f32 = 2.;
/// Value under which results may become inaccurate and that is not recommended to use.
pub const LOW_SEARCH_THRESHOLD_VALUE_WARNING: f32 = 0.15;
/// Value above which computation times may become very long and that is not recommended to use.
pub const HIGH_SEARCH_THRESHOLD_VALUE_WARNING: f32 = 0.25;

//optimizer dummy, used as a shared, read only target to compute dmg from during the optimisation process
//here we want every stats to be close to those of a real champion (unlike in game dummy)
const OPTIMIZER_DUMMY_RUNES_PAGE: RunesPage = RunesPage {
    shard1: RuneShard::Left,
    shard2: RuneShard::Left,
    shard3: RuneShard::Left,
};

const OPTIMIZER_DUMMY_SKILL_ORDER: SkillOrder = SkillOrder::const_default(); //does nothing since dummy has no spell (except passing validity checks when creating the dummy)

/// Using Ahri stats for squishy dummy.
const SQUISHY_OPTIMIZER_DUMMY_PROPERTIES: UnitProperties = UnitProperties {
    name: "squishy (e.g. Ahri)",
    as_limit: Unit::DEFAULT_AS_LIMIT,
    as_ratio: 0.625,
    windup_percent: 0.20,
    windup_modifier: 1.,
    base_stats: UnitStats {
        hp: 590.,
        mana: 418.,
        base_ad: 53.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 21.,
        mr: 30.,
        base_as: 0.668,
        bonus_as: 0.,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: Unit::BASE_CRIT_DMG,
        ms_flat: 330.,
        ms_percent: 0.,
        lethality: 0.,
        armor_pen_percent: 0.,
        magic_pen_flat: 0.,
        magic_pen_percent: 0.,
        armor_red_flat: 0.,
        armor_red_percent: 0.,
        mr_red_flat: 0.,
        mr_red_percent: 0.,
        life_steal: 0.,
        omnivamp: 0.,
    },
    growth_stats: UnitStats {
        hp: 104.,
        mana: 25.,
        base_ad: 3.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 4.7,
        mr: 1.3,
        base_as: 0.,
        bonus_as: 0.022,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.,
        lethality: 0.,
        armor_pen_percent: 0.,
        magic_pen_flat: 0.,
        magic_pen_percent: 0.,
        armor_red_flat: 0.,
        armor_red_percent: 0.,
        mr_red_flat: 0.,
        mr_red_percent: 0.,
        life_steal: 0.,
        omnivamp: 0.,
    },
    on_lvl_set: None,
    init_spells: None,
    basic_attack: null_basic_attack,
    q: NULL_SPELL,
    w: NULL_SPELL,
    e: NULL_SPELL,
    r: NULL_SPELL,
    fight_scenarios: &[(null_simulate_fight, "null")],
    unit_defaults: UnitDefaults {
        runes_pages: &OPTIMIZER_DUMMY_RUNES_PAGE,
        skill_order: &OPTIMIZER_DUMMY_SKILL_ORDER,
        legendary_items_pool: &ALL_LEGENDARY_ITEMS,
        boots_pool: &ALL_BOOTS,
        support_items_pool: &ALL_SUPPORT_ITEMS,
    },
};

/// Using Riven stats for bruiser dummy with additionnal stats from bruiser items.
const BRUISER_OPTIMIZER_DUMMY_PROPERTIES: UnitProperties = UnitProperties {
    name: "bruiser (e.g. Riven)",
    as_limit: Unit::DEFAULT_AS_LIMIT,
    as_ratio: 0.625,
    windup_percent: 0.16667,
    windup_modifier: 1.,
    base_stats: UnitStats {
        hp: 630.,
        mana: 263., //using darius mana
        base_ad: 64.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 33.,
        mr: 32.,
        base_as: 0.625,
        bonus_as: 0.,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: Unit::BASE_CRIT_DMG,
        ms_flat: 340.,
        ms_percent: 0.,
        lethality: 0.,
        armor_pen_percent: 0.,
        magic_pen_flat: 0.,
        magic_pen_percent: 0.,
        armor_red_flat: 0.,
        armor_red_percent: 0.,
        mr_red_flat: 0.,
        mr_red_percent: 0.,
        life_steal: 0.,
        omnivamp: 0.,
    },
    //assumes 1 defensive item in ennemy bruiser build
    growth_stats: UnitStats {
        hp: 100.
            + (AVG_ITEM_COST_WITH_BOOTS * 1.4 / 3.) / (HP_GOLD_VALUE * (MAX_UNIT_LVL_F32 - 1.)), //additionnal stat from bruiser items
        mana: 58., //using darius mana
        base_ad: 3.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 4.4
            + (AVG_ITEM_COST_WITH_BOOTS * 0.8 / 3.) / (ARMOR_GOLD_VALUE * (MAX_UNIT_LVL_F32 - 1.)), //additionnal stat from bruiser items
        mr: 2.05
            + (AVG_ITEM_COST_WITH_BOOTS * 0.8 / 3.) / (MR_GOLD_VALUE * (MAX_UNIT_LVL_F32 - 1.)), //additionnal stat from bruiser items
        base_as: 0.,
        bonus_as: 0.035,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.,
        lethality: 0.,
        armor_pen_percent: 0.,
        magic_pen_flat: 0.,
        magic_pen_percent: 0.,
        armor_red_flat: 0.,
        armor_red_percent: 0.,
        mr_red_flat: 0.,
        mr_red_percent: 0.,
        life_steal: 0.,
        omnivamp: 0.,
    },
    on_lvl_set: None,
    init_spells: None,
    basic_attack: null_basic_attack,
    q: NULL_SPELL,
    w: NULL_SPELL,
    e: NULL_SPELL,
    r: NULL_SPELL,
    fight_scenarios: &[(null_simulate_fight, "null")],
    unit_defaults: UnitDefaults {
        runes_pages: &OPTIMIZER_DUMMY_RUNES_PAGE,
        skill_order: &OPTIMIZER_DUMMY_SKILL_ORDER,
        legendary_items_pool: &ALL_LEGENDARY_ITEMS,
        boots_pool: &ALL_BOOTS,
        support_items_pool: &ALL_SUPPORT_ITEMS,
    },
};

/// Using Ornn stats for bruiser dummy with additionnal stats from bruiser items.
const TANKY_OPTIMIZER_DUMMY_PROPERTIES: UnitProperties = UnitProperties {
    name: "tank (e.g. Ornn)",
    as_limit: Unit::DEFAULT_AS_LIMIT,
    as_ratio: 0.625,
    windup_percent: 0.21875,
    windup_modifier: 1.,
    base_stats: UnitStats {
        hp: 660.,
        mana: 341.,
        base_ad: 69.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 33.,
        mr: 32.,
        base_as: 0.625,
        bonus_as: 0.,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: Unit::BASE_CRIT_DMG,
        ms_flat: 335.,
        ms_percent: 0.,
        lethality: 0.,
        armor_pen_percent: 0.,
        magic_pen_flat: 0.,
        magic_pen_percent: 0.,
        armor_red_flat: 0.,
        armor_red_percent: 0.,
        mr_red_flat: 0.,
        mr_red_percent: 0.,
        life_steal: 0.,
        omnivamp: 0.,
    },
    //assumes 2 defensive items in ennemy tank build
    growth_stats: UnitStats {
        hp: 109.
            + (AVG_ITEM_COST_WITH_BOOTS * 2. * 1.4 / 3.)
                / (HP_GOLD_VALUE * (MAX_UNIT_LVL_F32 - 1.)), //additionnal stat from bruiser items
        mana: 65.,
        base_ad: 3.5,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 5.2
            + (AVG_ITEM_COST_WITH_BOOTS * 2. * 0.8 / 3.)
                / (ARMOR_GOLD_VALUE * (MAX_UNIT_LVL_F32 - 1.)), //additionnal stat from bruiser items
        mr: 2.05
            + (AVG_ITEM_COST_WITH_BOOTS * 2. * 0.8 / 3.)
                / (MR_GOLD_VALUE * (MAX_UNIT_LVL_F32 - 1.)), //additionnal stat from bruiser items
        base_as: 0.,
        bonus_as: 0.02,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.,
        lethality: 0.,
        armor_pen_percent: 0.,
        magic_pen_flat: 0.,
        magic_pen_percent: 0.,
        armor_red_flat: 0.,
        armor_red_percent: 0.,
        mr_red_flat: 0.,
        mr_red_percent: 0.,
        life_steal: 0.,
        omnivamp: 0.,
    },
    on_lvl_set: None,
    init_spells: None,
    basic_attack: null_basic_attack,
    q: NULL_SPELL,
    w: NULL_SPELL,
    e: NULL_SPELL,
    r: NULL_SPELL,
    fight_scenarios: &[(null_simulate_fight, "null")],
    unit_defaults: UnitDefaults {
        runes_pages: &OPTIMIZER_DUMMY_RUNES_PAGE,
        skill_order: &OPTIMIZER_DUMMY_SKILL_ORDER,
        legendary_items_pool: &ALL_LEGENDARY_ITEMS,
        boots_pool: &ALL_BOOTS,
        support_items_pool: &ALL_SUPPORT_ITEMS,
    },
};

pub const TARGET_OPTIONS: [&UnitProperties; 3] = [
    &SQUISHY_OPTIMIZER_DUMMY_PROPERTIES,
    &BRUISER_OPTIMIZER_DUMMY_PROPERTIES,
    &TANKY_OPTIMIZER_DUMMY_PROPERTIES,
];

/// Amount of experience gained farming for the average legendary item.
/// We approximate that the gold income is only from cs golds and passive golds generation.
const XP_PER_LEGENDARY_ITEM: f32 = AVG_XP_PER_CS * AVG_LEGENDARY_ITEM_COST * CS_PER_MIN
    / (AVG_GOLDS_PER_CS * CS_PER_MIN + PASSIVE_GOLDS_GEN_PER_MIN);
/// Amount of experience gained farming for the average boots item.
/// We approximate that the gold income is only from cs golds and passive golds generation.
const XP_PER_BOOTS_ITEM: f32 = AVG_XP_PER_CS * AVG_BOOTS_COST * CS_PER_MIN
    / (AVG_GOLDS_PER_CS * CS_PER_MIN + PASSIVE_GOLDS_GEN_PER_MIN);
/// Amount of experience gained farming for the average support item.
/// We approximate that the gold income is only from cs golds and passive golds generation.
const XP_PER_SUPPORT_ITEM: f32 = AVG_XP_PER_CS * AVG_SUPPORT_ITEM_COST * CS_PER_MIN
    / (AVG_GOLDS_PER_CS * CS_PER_MIN + PASSIVE_GOLDS_GEN_PER_MIN);

/// Amount of cumulative xp required to reach the given lvl.
const CUM_XP_NEEDED_FOR_LVL_UP_BY_LVL: [f32; MAX_UNIT_LVL - 1] = [
    280.,   //needed to reach lvl 2
    660.,   //needed to reach lvl 3
    1140.,  //needed to reach lvl 4
    1720.,  //needed to reach lvl 5
    2400.,  //needed to reach lvl 6
    3180.,  //needed to reach lvl 7
    4060.,  //needed to reach lvl 8
    5040.,  //needed to reach lvl 9
    6120.,  //needed to reach lvl 10
    7300.,  //needed to reach lvl 11
    8580.,  //needed to reach lvl 12
    9960.,  //needed to reach lvl 13
    11440., //needed to reach lvl 14
    13020., //needed to reach lvl 15
    14700., //needed to reach lvl 16
    16480., //needed to reach lvl 17
    18360., //needed to reach lvl 18
];

/// From number of items, returns the associated unit lvl.
fn lvl_from_number_of_items(item_slot: usize, boots_slot: usize, support_item_slot: usize) -> u8 {
    let mut cum_xp: f32 = 0.;
    for i in 1..=item_slot {
        if i == boots_slot {
            cum_xp += XP_PER_BOOTS_ITEM;
        } else if i == support_item_slot {
            cum_xp += XP_PER_SUPPORT_ITEM;
        } else {
            cum_xp += XP_PER_LEGENDARY_ITEM;
        }
    }

    let mut lvl: u8 = MIN_UNIT_LVL; //lvl cannot be below MIN_UNIT_LVL, so start at this value
    while usize::from(lvl - 1) < MAX_UNIT_LVL - 1
        && cum_xp >= CUM_XP_NEEDED_FOR_LVL_UP_BY_LVL[usize::from(lvl - 1)]
    {
        lvl += 1;
    }
    lvl
}

/// Sorts a clone of the slice and compares adjacent elements to find if there is duplicates.
/// Return the index of the first duplicate found, if any.
fn has_duplicates(slice: &[&'static Item]) -> Option<usize> {
    let mut data: Vec<&Item> = Vec::from(slice);
    data.sort_unstable();
    for (idx, window) in data.windows(2).enumerate() {
        if *window[0] == *window[1] {
            return Some(idx);
        }
    }
    None
}

/// Returns the average of the curve formed by the given points.
/// values is a slice over values for y.
/// golds is a slice over associated values for x (must be in increasing order and same length as values),
/// Together they represent a series of points (x,y).
///
/// The curve is defined piecewise for windows of two consecutive points (xA,yA) and (xB,yB),
/// the formula for a piece is: y = yA + (yB - yA)*((x - xA)/(xB - xA))^2.
/// The curve is the union of every window.
///
/// If the last value in golds (x) is lower than `max_golds`,
/// the last value in values (y) is used to prolong the curve to reach `max_golds`.
/// The returned area is equal to the integral of the curve over its domain: [golds[0], max(golds[len-1], `max_golds`)]
/// (assumes x is sorted in increasing order).
///
/// If golds (x) or values (y) contains less than two values, or if golds (x) is not sorted,
/// a wrong area may be returned (but the function will run fine).
/// If values (y) is shorter than golds (x), the function may try to access data out of array bounds and crash.
fn gold_weighted_average(values: &[f32], golds: &[f32], max_golds: f32) -> f32 {
    //piece by piece area calculation
    let mut area: f32 = 0.;
    for (x, y) in zip(golds.windows(2), values.windows(2)) {
        //integral of yA + (yB - yA)*((x - xA)/(xB - xA))^2 from xA to xB = (xB - xA)*(2*yA + yB)/3
        area += (x[1] - x[0]) * (2. * y[0] + y[1]) / 3.;
    }
    //prolong area up to max_golds
    let last_idx: usize = golds.len() - 1;
    if golds[last_idx] < max_golds {
        area += (max_golds - golds[last_idx]) * (values[last_idx]);
    }
    area / (max_golds - golds[0])
}

#[derive(Debug, Clone)]
pub struct BuildContainer {
    pub build: Build,
    pub cumulated_utils: EnumSet<ItemUtils>,
    pub golds: [f32; MAX_UNIT_ITEMS + 1], //starting golds + 1 value per item
    pub dps: [f32; MAX_UNIT_ITEMS + 1],   //starting dps + 1 value per item
    pub defense: [f32; MAX_UNIT_ITEMS + 1], //starting defense + 1 value per item
    pub ms: [f32; MAX_UNIT_ITEMS + 1],    //starting ms + 1 value per item
}

impl BuildContainer {
    /// Returns the build score at the given item count.
    /// `Judgment_weights` must be >= 0 and normalized (their sum must be 3.0) for the formula to be correct.
    pub fn get_score_with_normalized_weights(
        &self,
        item_count: usize,
        normalized_judgment_weights: (f32, f32, f32),
    ) -> f32 {
        score_formula_with_normalized_weights(
            self.golds[item_count],
            self.dps[item_count],
            self.defense[item_count],
            self.ms[item_count],
            normalized_judgment_weights,
        )
    }

    /// Returns the build average score over the requested item slots.
    /// `Judgment_weights` must be >= 0 and normalized (their sum must be 3.0) for the formula to be correct.
    pub fn get_avg_score_with_normalized_weights(
        &self,
        n_items: usize,
        max_golds: f32,
        normalized_judgment_weights: (f32, f32, f32),
    ) -> f32 {
        //sanity check
        assert!(
            n_items != 0,
            "number of items to compute average score from must be at least 1"
        );
        let len: usize = n_items + 1;
        let mut scores: Vec<f32> = Vec::with_capacity(len);
        for i in 0..len {
            scores.push(self.get_score_with_normalized_weights(i, normalized_judgment_weights));
        }
        gold_weighted_average(&scores, &self.golds[0..len], max_golds)
    }
}

#[derive(Debug)]
pub struct BuildsGenerationSettings {
    pub target_properties: &'static UnitProperties,
    pub fight_duration: f32,
    pub ad_taken_percent: f32,
    pub judgment_weights: (f32, f32, f32),
    pub n_items: usize,
    pub boots_slot: usize,
    pub allow_boots_if_no_slot: bool,
    pub support_item_slot: usize,
    pub legendary_items_pool: Vec<&'static Item>,
    pub boots_pool: Vec<&'static Item>,
    pub support_items_pool: Vec<&'static Item>,
    pub allow_manaflow_first_item: bool, //overrides items pools, but not mandatory items
    pub mandatory_items: Build,
    pub search_threshold: f32,
}

impl Default for BuildsGenerationSettings {
    fn default() -> Self {
        BuildsGenerationSettings {
            target_properties: &SQUISHY_OPTIMIZER_DUMMY_PROPERTIES,
            fight_duration: 8.,
            ad_taken_percent: 0.60,
            judgment_weights: (1., 0.25, 0.5),
            n_items: 5,
            boots_slot: 2,
            allow_boots_if_no_slot: true,
            support_item_slot: 0,
            legendary_items_pool: Vec::from(ALL_LEGENDARY_ITEMS),
            boots_pool: Vec::from(ALL_BOOTS),
            support_items_pool: Vec::from(ALL_SUPPORT_ITEMS),
            allow_manaflow_first_item: false, //may change this to true, idk
            mandatory_items: Build::default(),
            search_threshold: 0.20,
        }
    }
}

impl BuildsGenerationSettings {
    pub fn default_by_champion(properties_ref: &UnitProperties) -> Self {
        #[allow(clippy::if_same_then_else)]
        if *properties_ref == Unit::ASHE_PROPERTIES {
            BuildsGenerationSettings {
                //boots_slot: 1, //gives questionable results
                legendary_items_pool: Vec::from(properties_ref.unit_defaults.legendary_items_pool),
                boots_pool: Vec::from(properties_ref.unit_defaults.boots_pool),
                support_items_pool: Vec::from(properties_ref.unit_defaults.support_items_pool),
                search_threshold: 0.15,
                ..Default::default()
            }
        } else if *properties_ref == Unit::DRAVEN_PROPERTIES {
            BuildsGenerationSettings {
                legendary_items_pool: Vec::from(properties_ref.unit_defaults.legendary_items_pool),
                boots_pool: Vec::from(properties_ref.unit_defaults.boots_pool),
                support_items_pool: Vec::from(properties_ref.unit_defaults.support_items_pool),
                search_threshold: 0.15,
                ..Default::default()
            }
        } else if *properties_ref == Unit::KAISA_PROPERTIES {
            BuildsGenerationSettings {
                legendary_items_pool: Vec::from(properties_ref.unit_defaults.legendary_items_pool),
                boots_pool: Vec::from(properties_ref.unit_defaults.boots_pool),
                support_items_pool: Vec::from(properties_ref.unit_defaults.support_items_pool),
                search_threshold: 0.15,
                ..Default::default()
            }
        } else if *properties_ref == Unit::LUCIAN_PROPERTIES {
            BuildsGenerationSettings {
                legendary_items_pool: Vec::from(properties_ref.unit_defaults.legendary_items_pool),
                boots_pool: Vec::from(properties_ref.unit_defaults.boots_pool),
                support_items_pool: Vec::from(properties_ref.unit_defaults.support_items_pool),
                ..Default::default()
            }
        } else if *properties_ref == Unit::VARUS_PROPERTIES {
            BuildsGenerationSettings {
                legendary_items_pool: Vec::from(properties_ref.unit_defaults.legendary_items_pool),
                boots_pool: Vec::from(properties_ref.unit_defaults.boots_pool),
                support_items_pool: Vec::from(properties_ref.unit_defaults.support_items_pool),
                search_threshold: 0.15,
                ..Default::default()
            }
        } else {
            BuildsGenerationSettings {
                legendary_items_pool: Vec::from(properties_ref.unit_defaults.legendary_items_pool),
                boots_pool: Vec::from(properties_ref.unit_defaults.boots_pool),
                support_items_pool: Vec::from(properties_ref.unit_defaults.support_items_pool),
                ..Default::default()
            }
        }
    }

    pub fn check_settings(&self) -> Result<(), String> {
        if !TARGET_OPTIONS
            .iter()
            .any(|properties| self.target_properties == *properties)
        {
            return Err(format!(
                "'{}' is not a recognized target",
                self.target_properties.name
            ));
        }

        if !self.fight_duration.is_finite()
            || !(0.0..=MAX_FIGHT_DURATION).contains(&self.fight_duration)
        {
            return Err(format!(
                "fight duration must be greater than 0 and under {MAX_FIGHT_DURATION} (got {})",
                self.fight_duration
            ));
        }
        if !self.ad_taken_percent.is_finite() || !(0.0..=1.0).contains(&self.ad_taken_percent) {
            return Err(format!(
                "proportion of ad dmg taken must be greater than 0% and under 100% (got {}%)",
                100. * self.ad_taken_percent
            ));
        }
        if !self.judgment_weights.0.is_finite()
            || !self.judgment_weights.1.is_finite()
            || !self.judgment_weights.2.is_finite()
            || self.judgment_weights.0 < 0.
            || self.judgment_weights.1 < 0.
            || self.judgment_weights.2 < 0.
        {
            return Err(format!(
                "judgment weights must be finite and positive (got 'DPS {}, defense {}, mobility {}')",
                self.judgment_weights.0, self.judgment_weights.1, self.judgment_weights.2,
            ));
        }
        if !self.search_threshold.is_finite() || !(0.0..=1.0).contains(&self.search_threshold) {
            return Err(format!(
                "search threshold must be greater than 0% and under 100% (got {})",
                100. * self.search_threshold
            ));
        }

        if !(1..=MAX_UNIT_ITEMS).contains(&self.n_items) {
            return Err(format!(
                "number of items per build must be between 1 and {MAX_UNIT_ITEMS} (got {})",
                self.n_items
            ));
        }
        if !(0..=MAX_UNIT_ITEMS).contains(&self.boots_slot) {
            return Err(format!(
                "boots slot must be between 1 and {MAX_UNIT_ITEMS} (or 0 if none, got {})",
                self.boots_slot
            ));
        }
        if !(0..=MAX_UNIT_ITEMS).contains(&self.support_item_slot) {
            return Err(format!(
                "support item slot must be between 1 and {MAX_UNIT_ITEMS} (or 0 if none, got {})",
                self.support_item_slot
            ));
        }
        if self.boots_slot != 0 {
            if self.boots_slot == self.support_item_slot {
                return Err(format!(
                    "cannot have boots and support item at the same slot (slot {})",
                    self.boots_slot
                ));
            }
            if self.boots_pool.is_empty() {
                return Err("boots pool is empty".to_string());
            }
            if *self.mandatory_items[self.boots_slot - 1] != NULL_ITEM {
                return Err(format!(
                    "cannot have a mandatory item at the boots slot (slot {})",
                    self.boots_slot
                ));
            }
            if self
                .mandatory_items
                .iter()
                .any(|item| item.item_groups.contains(ItemGroups::Boots))
            {
                return Err("cannot have another boots in mandatory items if the boots slot is already set to something different than none".to_string());
            }
        }
        if self.support_item_slot != 0 {
            if self.support_items_pool.is_empty() {
                return Err("support items pool is empty".to_string());
            }
            if *self.mandatory_items[self.support_item_slot - 1] != NULL_ITEM {
                return Err(format!(
                    "cannot have a mandatory item at the support item slot (slot {})",
                    self.support_item_slot
                ));
            }
            if self
                .mandatory_items
                .iter()
                .any(|item| item.item_groups.contains(ItemGroups::Support))
            {
                return Err("cannot have another support item in mandatory items if the support item slot is already set to something different than none".to_string());
            }
        }
        //this check must be done after being sure that `boots_slot` and `support_item_slot` are different
        if self.legendary_items_pool.len()
            + usize::from((1..=self.n_items).contains(&self.boots_slot))
            + usize::from((1..=self.n_items).contains(&self.support_item_slot))
            < self.n_items
        {
            return Err(format!(
                "not enough legendary items in pool to fill {} items slots",
                self.n_items
            ));
        }
        if self
            .legendary_items_pool
            .iter()
            .any(|&item_ref| *item_ref == NULL_ITEM)
            || self
                .boots_pool
                .iter()
                .any(|&item_ref| *item_ref == NULL_ITEM)
            || self
                .support_items_pool
                .iter()
                .any(|&item_ref| *item_ref == NULL_ITEM)
        {
            return Err("items pools cannot contain NULL_ITEM".to_string());
        }
        if let Some(idx) = has_duplicates(&self.legendary_items_pool) {
            return Err(format!(
                "duplicates in legendary items pool: {:#}",
                self.legendary_items_pool[idx]
            ));
        }
        if let Some(idx) = has_duplicates(&self.boots_pool) {
            return Err(format!(
                "duplicates in boots pool: {:#}",
                self.legendary_items_pool[idx]
            ));
        }
        if let Some(idx) = has_duplicates(&self.support_items_pool) {
            return Err(format!(
                "duplicates in support items pool: {:#}",
                self.legendary_items_pool[idx]
            ));
        }
        if let Err(error_msg) = self.mandatory_items.check_validity() {
            return Err(format!(
                "{} is an invalid combination of items: {error_msg}",
                self.mandatory_items
            ));
        }
        Ok(())
    }
}

pub fn get_normalized_judgment_weights(
    (dps_value_weight, defense_weight, ms_weight): (f32, f32, f32),
) -> (f32, f32, f32) {
    let sum: f32 = dps_value_weight + defense_weight + ms_weight;
    (
        dps_value_weight / sum,
        defense_weight / sum,
        ms_weight / sum,
    )
}

/// Formula for the the score of a build.
/// `Judgment_weights` must be >= 0 and normalized (their sum must be 3.0) for the formula to be correct
/// (these requirements are not checked when calling this function for performance reasons).
#[inline]
pub fn score_formula_with_normalized_weights(
    golds: f32,
    dps: f32,
    defense: f32,
    ms: f32,
    (norm_dps_value_weight, norm_defense_weight, norm_ms_weight): (f32, f32, f32),
) -> f32 {
    (AVG_ITEM_COST_WITH_BOOTS * dps / golds).powf(norm_dps_value_weight) //divide by the number of 'equivalent' items instead of golds so that the 'dps value' obtained is not a too small number
        * defense.powf(norm_defense_weight)
        * ms.powf(norm_ms_weight)
}

fn generate_build_layer(
    current_builds: Vec<BuildContainer>,
    pool: &[&'static Item],
    layer_idx: usize,
    normalized_judgment_weights: (f32, f32, f32),
) -> Option<Vec<BuildContainer>> {
    let mut new_builds: Vec<BuildContainer> = Vec::with_capacity(current_builds.len()); //new_builds will probably have at least this size
    let mut hashes: FxHashMap<BuildHash, usize> =
        FxHashMap::with_capacity_and_hasher(current_builds.len(), FxBuildHasher);

    let max_golds: f32 = current_builds
        .iter()
        .map(|build| build.golds[layer_idx])
        .max_by(|a, b| a.partial_cmp(b).expect("failed to compare floats"))
        .unwrap_or(STARTING_GOLDS); //needed later

    for container in current_builds {
        for &pool_item in pool {
            let mut candidate: BuildContainer = container.clone();

            //candidate build must have no duplicates
            if candidate.build.iter().any(|&x| *x == *pool_item) {
                continue;
            }
            //candidate build must have no item groups overlap
            candidate.build[layer_idx] = pool_item;
            if candidate.build.has_item_groups_overlap() {
                continue;
            }
            //check hash of candidate build
            let candidate_hash: BuildHash = candidate.build.get_hash();
            if let Some(&other_idx) = hashes.get(&candidate_hash) {
                //if hash already exists
                let other: &BuildContainer = &new_builds[other_idx];
                if candidate.get_avg_score_with_normalized_weights(
                    layer_idx,
                    max_golds,
                    normalized_judgment_weights,
                ) > other.get_avg_score_with_normalized_weights(
                    layer_idx,
                    max_golds,
                    normalized_judgment_weights,
                ) {
                    //if candidate path is better than other path, replace other build with candidate
                    new_builds[other_idx] = candidate;
                }
                //else, drop candidate
            } else {
                //if hash is unique, add candidate to new builds
                hashes.insert(candidate_hash, new_builds.len()); //use len before pushing element to get correct index
                new_builds.push(candidate);
            }
        }
    }
    if new_builds.is_empty() {
        return None;
    }
    Some(new_builds)
}

/// Get the size of the chunks needed to process a given amount of elements in parallel with the specified amount of workers.
/// The chunk size will be choosen so that the number of elements per chunk is the most evenly distributed possible.
fn get_chunksize_from_thread_count(n_elements: usize, thread_count: NonZero<usize>) -> usize {
    usize::max(
        1,
        (n_elements + (thread_count.get() - 1)) / thread_count.get(),
    )
}

/// Number of pareto scores to consider. Must be consistent with the number of elements in the `ParetoPoint` type.
/// f32 because only used in f32 calculations.
const N_PARETO_SCORES: f32 = 7.;
struct ParetoSpacePoint {
    utils: EnumSet<ItemUtils>, //represents 3 scores
    golds: f32,
    dps: f32,
    defense: f32,
    ms: f32,
}

impl ParetoSpacePoint {
    /// Returns true if self has reasons to be kept against a reference point, false otherwise.
    /// We do not use the usual definition of pareto efficiency but a variation to keep points
    /// that are close to the pareto front as well (up to a given limit, `discard_percent`).
    fn is_pareto_efficient(&self, ref_point: &Self, discard_percent: f32) -> bool {
        !((self.utils & !ref_point.utils).is_empty())
            || self.golds < ref_point.golds
            || self.dps > discard_percent * ref_point.dps
            || self.defense > discard_percent * ref_point.defense
            || self.ms > discard_percent * ref_point.ms
    }

    fn from_build_fight_simulation(
        container: &BuildContainer,
        item_idx: usize,
        champ: &mut Unit,
        target_stats: &UnitStats,
        fight_duration: f32,
        ad_taken_percent: f32,
    ) -> Self {
        champ
            .set_build(container.build, true)
            .expect("failed to set build");
        let mut avg_dps: f32 = 0.;
        let mut avg_defense: f32 = 0.;
        let mut avg_ms: f32 = 0.;

        //to avoid combinations of items that are local optimums for the given fight_duration,
        //we simulate for 3 fight durations scattered across a normal distribution around the original fight_duration
        //the final scores are calculated from the weighted sum of each simulation result (weight according to the normal distribution)
        let std_dev: f32 = 0.15 * fight_duration; //chosen arbitrarily, but it works

        //weights for a value at 1 std_dev from the mean
        let (dps, defense, ms): (f32, f32, f32) =
            champ.simulate_fight(target_stats, fight_duration - std_dev, ad_taken_percent);
        avg_dps += 0.274 * dps;
        avg_defense += 0.274 * defense;
        avg_ms += 0.274 * ms;

        //weights for a value at the mean
        let (dps, defense, ms): (f32, f32, f32) =
            champ.simulate_fight(target_stats, fight_duration, ad_taken_percent);
        avg_dps += 0.452 * dps;
        avg_defense += 0.452 * defense;
        avg_ms += 0.452 * ms;

        //weights for a value at 1 std_dev from the mean
        let (dps, defense, ms): (f32, f32, f32) =
            champ.simulate_fight(target_stats, fight_duration + std_dev, ad_taken_percent);
        avg_dps += 0.274 * dps;
        avg_defense += 0.274 * defense;
        avg_ms += 0.274 * ms;

        Self {
            utils: container.cumulated_utils | container.build[item_idx].utils, //only check current item, as containers should records items utils cumulatively from previous items
            golds: champ.build_cost,
            dps: avg_dps,
            defense: avg_defense,
            ms: avg_ms,
        }
    }
}

fn simulate_chunk_of_builds(
    chunk: &[BuildContainer],
    champ: &mut Unit,
    target_stats: &UnitStats,
    fight_duration: f32,
    ad_taken_percent: f32,
    item_idx: usize,
) -> Vec<ParetoSpacePoint> {
    chunk
        .iter()
        .map(|container| {
            ParetoSpacePoint::from_build_fight_simulation(
                container,
                item_idx,
                champ,
                target_stats,
                fight_duration,
                ad_taken_percent,
            )
        })
        .collect()
}

/// returns a Vec<bool> indicating if each point at the corresponding index
/// is pareto efficient compared to the `reference_point`.
fn pareto_compare_chunk_to_ref_point(
    chunk: &[ParetoSpacePoint],
    ref_point: &ParetoSpacePoint,
    discard_percent: f32,
) -> Vec<bool> {
    chunk
        .iter()
        .map(|other_point| other_point.is_pareto_efficient(ref_point, discard_percent))
        .collect()
}

/// Returns a boolean mask indicating if a given point in part of the pareto front.
/// Also modifies the points Vec in place to only keep pareto points
/// (in the end, remaining values in points are mapped to indices that are true in `pareto_mask`).
/// Multi-threaded.
fn pareto_front_multithread(
    points: &mut Vec<ParetoSpacePoint>,
    discard_percent: f32,
    n_threads: NonZero<usize>,
) -> Vec<bool> {
    let input_len: usize = points.len();
    let mut pareto_mask: Vec<bool> = Vec::with_capacity(input_len);
    let mut pareto_indices: Vec<usize> = (0..input_len).collect();

    let mut idx: usize = 0;
    while idx < points.len() {
        let current_point: &ParetoSpacePoint = &points[idx];

        //update pareto mask, divide points into chunks to process them in parralel
        let chunk_size: usize = get_chunksize_from_thread_count(points.len(), n_threads);
        pareto_mask.clear();
        pareto_mask = points
            .par_chunks(chunk_size)
            .flat_map_iter(|chunk| {
                pareto_compare_chunk_to_ref_point(chunk, current_point, discard_percent)
            })
            .collect();
        pareto_mask[idx] = true; //keep self

        //pareto_mask.shrink_to_fit(); //useless because we will re-use the full capacity later

        let mut to_keep1 = pareto_mask.iter();
        let mut to_keep2 = to_keep1.clone();
        //i think this can be faster if done in parallel, todo: try scoped threads? (tradeoff might be worth it)
        points.retain(|_| *to_keep1.next().unwrap()); //will never panic as to_keep1 has the same length
        pareto_indices.retain(|_| *to_keep2.next().unwrap()); //same

        idx = pareto_mask[0..idx]
            .iter()
            .map(|&x| usize::from(x))
            .sum::<usize>()
            + 1;
    }
    //use the old vec in place for the return value
    pareto_mask.clear();
    pareto_mask.resize(input_len, false);
    for pareto_idx in pareto_indices {
        pareto_mask[pareto_idx] = true;
    }
    pareto_mask
}

/// Generates the best builds for a champion, using a multithreaded approach.
///
/// # Arguments
///
/// * `champ` - The champion for whom to generate the builds.
/// * `settings` - The settings to use for the builds generation.
///
/// # Returns
///
/// A vector of `BuildContainer` holding the best builds for the champion.
/// If the generation fails, returns an error message.
pub fn find_best_builds(
    champ: &mut Unit,
    settings: &BuildsGenerationSettings,
) -> Result<Vec<BuildContainer>, String> {
    //check input arguments
    settings.check_settings()?;

    //backup original champion configuration
    let original_lvl: NonZeroU8 = champ.lvl;
    let original_build: Build = champ.build;

    //get number of available threads
    let n_threads: NonZero<usize> =
        std::thread::available_parallelism().expect("failed to get amount of available threads");

    //start progress bar
    let progress_bar: ProgressBar = ProgressBar::new(settings.n_items as u64)
        .with_style(
            ProgressStyle::with_template(
                "{msg}\n[{elapsed_precise}] {bar} {pos}/{len} items {spinner}",
            )
            .expect("failed to create progress bar style"),
        )
        .with_message(format!(
            "calculating best builds for {}...",
            champ.properties.name
        ));
    progress_bar.enable_steady_tick(Duration::from_millis(200));

    //create target dummy
    let lvl: u8 = 6; //use lvl 6 for the empty build scores
    let mut target: Unit = Unit::new(
        settings.target_properties,
        OPTIMIZER_DUMMY_RUNES_PAGE,
        OPTIMIZER_DUMMY_SKILL_ORDER,
        lvl,
        Build::default(),
    )
    .expect("failed to create target dummy");

    //create empty build base scores
    champ.set_lvl(lvl).expect("failed to set lvl");
    let mut empty_build: BuildContainer = BuildContainer {
        build: Build::default(),
        cumulated_utils: enum_set!(),
        golds: [STARTING_GOLDS; MAX_UNIT_ITEMS + 1],
        dps: [0.; MAX_UNIT_ITEMS + 1],
        defense: [0.; MAX_UNIT_ITEMS + 1],
        ms: [0.; MAX_UNIT_ITEMS + 1],
    };
    let empty_build_point: ParetoSpacePoint = ParetoSpacePoint::from_build_fight_simulation(
        &empty_build,
        0,
        champ,
        &target.stats,
        settings.fight_duration,
        settings.ad_taken_percent,
    );
    empty_build.dps[0] = empty_build_point.dps;
    empty_build.defense[0] = empty_build_point.defense;
    empty_build.ms[0] = empty_build_point.ms;
    //no need to change other fields

    //initialize best builds generation
    let normalized_judgment_weights: (f32, f32, f32) =
        get_normalized_judgment_weights(settings.judgment_weights);
    let legendary_items_pool_with_boots_maybe: &[&Item] =
        if (settings.boots_slot == 0) && (settings.allow_boots_if_no_slot) {
            &[&settings.legendary_items_pool[..], &settings.boots_pool[..]].concat()
        } else {
            &settings.legendary_items_pool
        }; //treat boots as legendary items if no slot specified
    let discard_percent: f32 = 1. - settings.search_threshold;
    let mut best_builds: Vec<BuildContainer> = vec![empty_build];
    //start iterating on each item slot
    for item_idx in 0..settings.n_items {
        let item_slot: usize = item_idx + 1;

        //set champion & dummy lvl
        let lvl: u8 =
            lvl_from_number_of_items(item_slot, settings.boots_slot, settings.support_item_slot);
        champ.set_lvl(lvl).expect("failed to set lvl"); //no need to init (automatically done later when simulating fights)
        target.set_lvl(lvl).expect("failed to set lvl");
        target.init_fight();

        //set item pool
        let mut pool_ref: &[&Item] = &[settings.mandatory_items[item_idx]]; //need to assign temporary value outside of if else brackets
        let pool_without_manaflow: Vec<&Item>;
        if *settings.mandatory_items[item_idx] == NULL_ITEM {
            pool_ref = if item_slot == settings.boots_slot {
                &settings.boots_pool
            } else if item_slot == settings.support_item_slot {
                &settings.support_items_pool
            } else {
                legendary_items_pool_with_boots_maybe
            };

            if item_slot == 1 && !settings.allow_manaflow_first_item {
                pool_without_manaflow = pool_ref
                    .iter()
                    .filter(|item| !item.item_groups.contains(ItemGroups::Manaflow))
                    .copied()
                    .collect();
                pool_ref = &pool_without_manaflow;
            }
        }

        //generate next builds layer from pool
        if let Some(new_builds) =
            generate_build_layer(best_builds, pool_ref, item_idx, normalized_judgment_weights)
        {
            best_builds = new_builds;
        } else {
            //restore original champion configuration
            champ
                .set_lvl(original_lvl.get())
                .expect("failed to set lvl");
            champ
                .set_build(original_build, false)
                .expect("failed to set build");
            champ.init_fight();
            return Err(format!("can't reach requested item slot (stopped at slot {item_slot} because not enough items in pool/too much items incompatible with each other)"));
        }

        //divide builds into chunks to process them in parralel
        let chunk_size: usize = get_chunksize_from_thread_count(best_builds.len(), n_threads);
        let mut pareto_space_points: Vec<ParetoSpacePoint> = best_builds
            .par_chunks(chunk_size)
            .flat_map_iter(|chunk| {
                simulate_chunk_of_builds(
                    chunk,
                    &mut champ.clone(),
                    &target.stats,
                    settings.fight_duration,
                    settings.ad_taken_percent,
                    item_idx,
                )
            })
            .collect();

        //remove low value builds
        let max_score: f32 = pareto_space_points
            .iter()
            .map(|scores| {
                score_formula_with_normalized_weights(
                    scores.golds,
                    scores.dps,
                    scores.defense,
                    scores.ms,
                    normalized_judgment_weights,
                )
            })
            .max_by(|a, b| a.partial_cmp(b).expect("failed to compare floats"))
            .unwrap(); //points will never be empty (generate_build_layer will return an Err first)
        let mut idx: usize = 0;
        while idx < pareto_space_points.len() {
            let scores: &ParetoSpacePoint = &pareto_space_points[idx];
            if score_formula_with_normalized_weights(
                scores.golds,
                scores.dps,
                scores.defense,
                scores.ms,
                normalized_judgment_weights,
            ) < discard_percent * max_score
            {
                pareto_space_points.swap_remove(idx);
                best_builds.swap_remove(idx);
            } else {
                idx += 1;
            }
        }

        //keep pareto efficient builds
        let pareto_mask: Vec<bool> = pareto_front_multithread(
            &mut pareto_space_points,
            if item_slot == settings.n_items {
                1.
            } else {
                discard_percent.powf(1. / N_PARETO_SCORES) //heuristic criteria for N_PARETO_SCORES dimensions
            },
            n_threads,
        );
        let mut to_keep = pareto_mask.into_iter();
        best_builds.retain(|_| to_keep.next().unwrap()); //will never panic as to_keep has the same length

        //fill remaining build containers
        for (container, scores) in zip(best_builds.iter_mut(), pareto_space_points.iter()) {
            container.cumulated_utils = scores.utils;
            container.golds[item_slot] = scores.golds;
            container.dps[item_slot] = scores.dps;
            container.defense[item_slot] = scores.defense;
            container.ms[item_slot] = scores.ms;
        }

        //update progress bar
        progress_bar.inc(1);
    }
    //finish progress bar
    progress_bar.finish();
    println!(
        "found {} optimized builds for {}",
        best_builds.len(),
        champ.properties.name
    );

    //restore original champion configuration
    champ
        .set_lvl(original_lvl.get())
        .expect("failed to set lvl");
    champ
        .set_build(original_build, false)
        .expect("failed to set build");
    champ.init_fight();

    //return builds
    Ok(best_builds)
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    use super::*;

    #[test]
    pub fn test_default_build_generation_settings() {
        //test for an unknown champion
        if let Err(error_msg) =
            BuildsGenerationSettings::default_by_champion(&BRUISER_OPTIMIZER_DUMMY_PROPERTIES)
                .check_settings()
        {
            panic!(
                "default build generation settings (for an unspecified champion) are not valid: {}",
                error_msg
            )
        }

        //test for every champion
        for properties in Unit::ALL_CHAMPIONS.iter() {
            if let Err(error_msg) =
                BuildsGenerationSettings::default_by_champion(properties).check_settings()
            {
                panic!(
                    "default build generation settings for '{}' are not valid: {}",
                    properties.name, error_msg
                )
            }
        }
    }
}
