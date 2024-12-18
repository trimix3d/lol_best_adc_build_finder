use crate::builds_analyzer::sort_builds_by_score;
use crate::RuneKeystone;

use super::game_data::*;

use items_data::*;
use runes_data::*;
use units_data::*;

use enumset::{enum_set, EnumSet};
use indicatif::{ProgressBar, ProgressFinish, ProgressIterator, ProgressStyle};
use rayon::prelude::*;
use rustc_hash::{FxBuildHasher, FxHashMap};

use core::fmt;
use core::iter::zip;
use core::num::NonZeroUsize;
use core::time::Duration;

/// Meaningless to go above this value (in seconds).
pub(crate) const MAX_FIGHT_DURATION: f32 = 60.;
/// Value (in seconds) under which results may become inaccurate and that is not recommended to use.
pub(crate) const LOW_FIGHT_DURATION_VALUE_WARNING: f32 = 2.;
/// Value under which results may become inaccurate and that is not recommended to use.
pub(crate) const LOW_SEARCH_THRESHOLD_VALUE_WARNING: f32 = 0.05;
/// Value above which computation times may become very long and that is not recommended to use.
pub(crate) const HIGH_SEARCH_THRESHOLD_VALUE_WARNING: f32 = 0.25;

//optimizer dummy, used as a shared, read only target to compute dmg from during the optimisation process
//here we want every stats to be close to those of a real champion (unlike in game dummy)
const OPTIMIZER_DUMMY_RUNES_PAGE: RunesPage = RunesPage::const_default();

const OPTIMIZER_DUMMY_SKILL_ORDER: SkillOrder = SkillOrder::const_default(); //does nothing since dummy has no ability

#[allow(clippy::cast_precision_loss)]
const MAX_UNIT_LVL_F32: f32 = MAX_UNIT_LVL as f32; //`MAX_UNIT_LVL_F32` is well within f32 precision range

/// Using Ahri stats for squishy dummy.
pub const SQUISHY_OPTIMIZER_DUMMY_PROPERTIES: UnitProperties = UnitProperties {
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
        ap_percent: 0.,
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
        ability_dmg_modifier: 0.,
        phys_dmg_modifier: 0.,
        magic_dmg_modifier: 0.,
        true_dmg_modifier: 0.,
        tot_dmg_modifier: 0.,
    },
    growth_stats: UnitStats {
        hp: 104.,
        mana: 25.,
        base_ad: 3.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_percent: 0.,
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
        ability_dmg_modifier: 0.,
        phys_dmg_modifier: 0.,
        magic_dmg_modifier: 0.,
        true_dmg_modifier: 0.,
        tot_dmg_modifier: 0.,
    },
    basic_attack: null_basic_attack,
    q: NULL_BASIC_ABILITY,
    w: NULL_BASIC_ABILITY,
    e: NULL_BASIC_ABILITY,
    r: NULL_ULTIMATE_ABILITY,
    on_action_fns: OnActionFns {
        on_lvl_set: None,
        on_fight_init: None,
        special_active: None,
        on_ability_cast: None,
        on_ultimate_cast: None,
        on_ability_hit: None,
        on_ultimate_hit: None,
        on_basic_attack_cast: None,
        on_basic_attack_hit: None,
        on_phys_hit: None,
        on_magic_hit: None,
        on_true_dmg_hit: None,
        on_any_hit: None,
    },
    fight_scenarios: &[(null_simulate_fight, "null")],
    defaults: UnitDefaults {
        runes_pages: OPTIMIZER_DUMMY_RUNES_PAGE,
        skill_order: OPTIMIZER_DUMMY_SKILL_ORDER,
        legendary_items_pool: &ALL_LEGENDARY_ITEMS,
        boots_pool: &ALL_BOOTS,
        supp_items_pool: &ALL_SUPP_ITEMS,
    },
};

/// Using Riven stats for bruiser dummy with additionnal stats from bruiser items.
pub const BRUISER_OPTIMIZER_DUMMY_PROPERTIES: UnitProperties = UnitProperties {
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
        ap_percent: 0.,
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
        ability_dmg_modifier: 0.,
        phys_dmg_modifier: 0.,
        magic_dmg_modifier: 0.,
        true_dmg_modifier: 0.,
        tot_dmg_modifier: 0.,
    },
    //assumes 1 defensive item in ennemy bruiser build
    growth_stats: UnitStats {
        hp: 100.
            + (AVG_ITEM_COST_WITH_BOOTS * 1.4 / 3.) / (HP_GOLD_VALUE * (MAX_UNIT_LVL_F32 - 1.)), //additionnal stat from bruiser items
        mana: 58., //using darius mana
        base_ad: 3.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_percent: 0.,
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
        ability_dmg_modifier: 0.,
        phys_dmg_modifier: 0.,
        magic_dmg_modifier: 0.,
        true_dmg_modifier: 0.,
        tot_dmg_modifier: 0.,
    },
    basic_attack: null_basic_attack,
    q: NULL_BASIC_ABILITY,
    w: NULL_BASIC_ABILITY,
    e: NULL_BASIC_ABILITY,
    r: NULL_ULTIMATE_ABILITY,
    on_action_fns: OnActionFns {
        on_lvl_set: None,
        on_fight_init: None,
        special_active: None,
        on_ability_cast: None,
        on_ultimate_cast: None,
        on_ability_hit: None,
        on_ultimate_hit: None,
        on_basic_attack_cast: None,
        on_basic_attack_hit: None,
        on_phys_hit: None,
        on_magic_hit: None,
        on_true_dmg_hit: None,
        on_any_hit: None,
    },
    fight_scenarios: &[(null_simulate_fight, "null")],
    defaults: UnitDefaults {
        runes_pages: OPTIMIZER_DUMMY_RUNES_PAGE,
        skill_order: OPTIMIZER_DUMMY_SKILL_ORDER,
        legendary_items_pool: &ALL_LEGENDARY_ITEMS,
        boots_pool: &ALL_BOOTS,
        supp_items_pool: &ALL_SUPP_ITEMS,
    },
};

/// Using Ornn stats for bruiser dummy with additionnal stats from bruiser items.
pub const TANKY_OPTIMIZER_DUMMY_PROPERTIES: UnitProperties = UnitProperties {
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
        ap_percent: 0.,
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
        ability_dmg_modifier: 0.,
        phys_dmg_modifier: 0.,
        magic_dmg_modifier: 0.,
        true_dmg_modifier: 0.,
        tot_dmg_modifier: 0.,
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
        ap_percent: 0.,
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
        ability_dmg_modifier: 0.,
        phys_dmg_modifier: 0.,
        magic_dmg_modifier: 0.,
        true_dmg_modifier: 0.,
        tot_dmg_modifier: 0.,
    },
    basic_attack: null_basic_attack,
    q: NULL_BASIC_ABILITY,
    w: NULL_BASIC_ABILITY,
    e: NULL_BASIC_ABILITY,
    r: NULL_ULTIMATE_ABILITY,
    on_action_fns: OnActionFns {
        on_lvl_set: None,
        on_fight_init: None,
        special_active: None,
        on_ability_cast: None,
        on_ultimate_cast: None,
        on_ability_hit: None,
        on_ultimate_hit: None,
        on_basic_attack_cast: None,
        on_basic_attack_hit: None,
        on_phys_hit: None,
        on_magic_hit: None,
        on_true_dmg_hit: None,
        on_any_hit: None,
    },
    fight_scenarios: &[(null_simulate_fight, "null")],
    defaults: UnitDefaults {
        runes_pages: OPTIMIZER_DUMMY_RUNES_PAGE,
        skill_order: OPTIMIZER_DUMMY_SKILL_ORDER,
        legendary_items_pool: &ALL_LEGENDARY_ITEMS,
        boots_pool: &ALL_BOOTS,
        supp_items_pool: &ALL_SUPP_ITEMS,
    },
};

pub(crate) const TARGET_OPTIONS: [&UnitProperties; 3] = [
    &SQUISHY_OPTIMIZER_DUMMY_PROPERTIES,
    &BRUISER_OPTIMIZER_DUMMY_PROPERTIES,
    &TANKY_OPTIMIZER_DUMMY_PROPERTIES,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemSlot {
    Slot(usize),
    Any,
    None,
}

impl fmt::Display for ItemSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Slot(slot) => write!(f, "{slot}"),
            Self::Any => f.write_str("Any"),
            Self::None => f.write_str("None"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BuildsGenerationSettings {
    pub target_properties: &'static UnitProperties,
    pub fight_scenario_number: NonZeroUsize,
    pub fight_duration: f32,
    pub phys_dmg_received_percent: f32,
    pub runes_page: RunesPage,
    pub n_items: usize,
    pub mandatory_items: Build,
    pub boots_slot: ItemSlot,
    pub supp_item_slot: ItemSlot,
    pub legendary_items_pool: Vec<&'static Item>,
    pub boots_pool: Vec<&'static Item>,
    pub supp_items_pool: Vec<&'static Item>,
    pub allow_manaflow_first_item: bool, //only effective if manaflow items in items pool, overridden by mandatory items
    pub weights: (f32, f32, f32),
    pub search_threshold: f32,
}

const DEFAULT_FIGHT_DURATION: f32 = 6.0;
impl Default for BuildsGenerationSettings {
    fn default() -> Self {
        BuildsGenerationSettings {
            target_properties: &SQUISHY_OPTIMIZER_DUMMY_PROPERTIES,
            fight_scenario_number: NonZeroUsize::new(1).unwrap(),
            fight_duration: DEFAULT_FIGHT_DURATION,
            phys_dmg_received_percent: 0.60,
            runes_page: RunesPage::default(),
            n_items: 4,
            mandatory_items: Build::default(),
            boots_slot: ItemSlot::Slot(2),
            supp_item_slot: ItemSlot::None,
            legendary_items_pool: Vec::from(ALL_LEGENDARY_ITEMS),
            boots_pool: Vec::from(ALL_BOOTS),
            supp_items_pool: Vec::from(ALL_SUPP_ITEMS),
            allow_manaflow_first_item: false, //may change this to true, idk
            weights: (1., 0.25, 0.5),
            search_threshold: 0.15,
        }
    }
}

impl BuildsGenerationSettings {
    pub fn default_by_champion(properties: &UnitProperties) -> Self {
        let settings: BuildsGenerationSettings = BuildsGenerationSettings {
            runes_page: properties.defaults.runes_pages,
            legendary_items_pool: Vec::from(properties.defaults.legendary_items_pool),
            boots_pool: Vec::from(properties.defaults.boots_pool),
            supp_items_pool: Vec::from(properties.defaults.supp_items_pool),
            ..Default::default()
        };

        //champions specific settings here
        /*
        '⠀⣞⢽⢪⢣⢣⢣⢫⡺⡵⣝⡮⣗⢷⢽⢽⢽⣮⡷⡽⣜⣜⢮⢺⣜⢷⢽⢝⡽⣝
         ⠸⡸⠜⠕⠕⠁⢁⢇⢏⢽⢺⣪⡳⡝⣎⣏⢯⢞⡿⣟⣷⣳⢯⡷⣽⢽⢯⣳⣫⠇
         ⠀⠀⢀⢀⢄⢬⢪⡪⡎⣆⡈⠚⠜⠕⠇⠗⠝⢕⢯⢫⣞⣯⣿⣻⡽⣏⢗⣗⠏⠀
         ⠀⠪⡪⡪⣪⢪⢺⢸⢢⢓⢆⢤⢀⠀⠀⠀⠀⠈⢊⢞⡾⣿⡯⣏⢮⠷⠁⠀⠀
         ⠀⠀⠀⠈⠊⠆⡃⠕⢕⢇⢇⢇⢇⢇⢏⢎⢎⢆⢄⠀⢑⣽⣿⢝⠲⠉⠀⠀⠀⠀
         ⠀⠀⠀⠀⠀⡿⠂⠠⠀⡇⢇⠕⢈⣀⠀⠁⠡⠣⡣⡫⣂⣿⠯⢪⠰⠂⠀⠀⠀⠀
         ⠀⠀⠀⠀⡦⡙⡂⢀⢤⢣⠣⡈⣾⡃⠠⠄⠀⡄⢱⣌⣶⢏⢊⠂⠀⠀⠀⠀⠀⠀
         ⠀⠀⠀⠀⢝⡲⣜⡮⡏⢎⢌⢂⠙⠢⠐⢀⢘⢵⣽⣿⡿⠁⠁⠀⠀⠀⠀⠀⠀⠀
         ⠀⠀⠀⠀⠨⣺⡺⡕⡕⡱⡑⡆⡕⡅⡕⡜⡼⢽⡻⠏⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
         ⠀⠀⠀⠀⣼⣳⣫⣾⣵⣗⡵⡱⡡⢣⢑⢕⢜⢕⡝⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
         ⠀⠀⠀⣴⣿⣾⣿⣿⣿⡿⡽⡑⢌⠪⡢⡣⣣⡟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
         ⠀⠀⠀⡟⡾⣿⢿⢿⢵⣽⣾⣼⣘⢸⢸⣞⡟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
         ⠀⠀⠀⠀⠁⠇⠡⠩⡫⢿⣝⡻⡮⣒⢽⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
        No switches?
        cannot match with string litterals inside constants :,(
        */
        //if *properties == Unit::APHELIOS_PROPERTIES {
        //    settings.search_threshold = 0.15;
        //}
        settings
    }

    pub fn check_settings(&self, champ_properties: &UnitProperties) -> Result<(), String> {
        if !TARGET_OPTIONS
            .iter()
            .any(|&properties| *self.target_properties == *properties)
        {
            return Err(format!(
                "'{}' is not a recognized target",
                self.target_properties.name
            ));
        }

        if self.fight_scenario_number.get() > champ_properties.fight_scenarios.len() {
            return Err(format!(
                "Fight scenario number for {} must be lower than {} (got {})",
                champ_properties.name,
                champ_properties.fight_scenarios.len(),
                self.fight_scenario_number.get(),
            ));
        }

        if !self.fight_duration.is_finite()
            || !(0.0..=MAX_FIGHT_DURATION).contains(&self.fight_duration)
        {
            return Err(format!(
                "Fight duration must be greater than 0 and under {MAX_FIGHT_DURATION} (got {})",
                self.fight_duration
            ));
        }

        if !self.phys_dmg_received_percent.is_finite()
            || !(0.0..=1.0).contains(&self.phys_dmg_received_percent)
        {
            return Err(format!(
                "Percentage of physical dmg received must be greater than 0% and under 100% (got {}%)",
                100. * self.phys_dmg_received_percent
            ));
        }

        if let Err(error_msg) = self.runes_page.check_validity() {
            return Err(format!("Invalid runes page: {error_msg}"));
        }

        if !(1..=MAX_UNIT_ITEMS).contains(&self.n_items) {
            return Err(format!(
                "Number of items per build must be between 1 and {MAX_UNIT_ITEMS} (got {})",
                self.n_items
            ));
        }

        if let Err(error_msg) = self.mandatory_items.check_validity() {
            return Err(format!(
                "{} is an invalid combination of items: {error_msg}",
                self.mandatory_items
            ));
        }

        let mut additional_items: usize = 0; //number of items additional to the legendary items in the generated build

        if let ItemSlot::Slot(boots_slot_number) = self.boots_slot {
            if (1..=self.n_items).contains(&boots_slot_number) {
                additional_items += 1;
            }
            if !(1..=MAX_UNIT_ITEMS).contains(&boots_slot_number) {
                return Err(format!(
                    "Boots slot must be between 1 and {MAX_UNIT_ITEMS} (got {})",
                    boots_slot_number
                ));
            }
            if let ItemSlot::Slot(supp_item_slot_number) = self.supp_item_slot {
                if boots_slot_number == supp_item_slot_number {
                    return Err(format!(
                        "Cannot have boots and support item at the same slot (slot {})",
                        boots_slot_number
                    ));
                }
            }
        }
        if self.boots_slot == ItemSlot::Any {
            additional_items += 1;
        }
        if self.boots_slot != ItemSlot::None {
            if self
                .mandatory_items
                .iter()
                .any(|item| item.item_groups.contains(ItemGroups::Boots))
            {
                return Err(
                    "Cannot have boots in mandatory items if the boots slot is already set"
                        .to_string(),
                );
            }
            if self.boots_pool.is_empty() {
                return Err("Boots pool is empty".to_string());
            }
        }

        if let ItemSlot::Slot(supp_item_slot_number) = self.supp_item_slot {
            if (1..=self.n_items).contains(&supp_item_slot_number) {
                additional_items += 1;
            }

            if !(1..=MAX_UNIT_ITEMS).contains(&supp_item_slot_number) {
                return Err(format!(
                    "Support item slot must be between 1 and {MAX_UNIT_ITEMS} (got {})",
                    supp_item_slot_number
                ));
            }
            if let ItemSlot::Slot(boots_slot_number) = self.boots_slot {
                if supp_item_slot_number == boots_slot_number {
                    return Err(format!(
                        "Cannot have boots and support item at the same slot (slot {})",
                        supp_item_slot_number
                    ));
                }
            }
        }
        if self.supp_item_slot == ItemSlot::Any {
            additional_items += 1;
        }
        if self.supp_item_slot != ItemSlot::None {
            if self
                .mandatory_items
                .iter()
                .any(|item| item.item_groups.contains(ItemGroups::Support))
            {
                return Err(
                    "Cannot have a support item in mandatory items if the support item slot is already set"
                    .to_string()
                );
            }
            if self.supp_items_pool.is_empty() {
                return Err("Support items pool is empty".to_string());
            }
        }

        //this check must be done after being sure that `boots_slot` and `supp_item_slot` are different
        if self.legendary_items_pool.len() + additional_items < self.n_items {
            return Err(format!(
                "Not enough legendary items in pool to fill {} items slots",
                self.n_items
            ));
        }
        if self
            .legendary_items_pool
            .iter()
            .any(|&item| *item == Item::NULL_ITEM)
            || self.boots_pool.iter().any(|&item| *item == Item::NULL_ITEM)
            || self
                .supp_items_pool
                .iter()
                .any(|&item| *item == Item::NULL_ITEM)
        {
            return Err("Items pools cannot contain `NULL_ITEM`".to_string());
        }
        if let Some(item) = crate::find_dupes_in_slice(&mut self.legendary_items_pool.clone()) {
            return Err(format!("Duplicates in legendary items pool: {:#}", item));
        }

        if let Some(item) = crate::find_dupes_in_slice(&mut self.boots_pool.clone()) {
            return Err(format!("Duplicates in boots pool: {:#}", item));
        }

        if let Some(item) = crate::find_dupes_in_slice(&mut self.supp_items_pool.clone()) {
            return Err(format!("Duplicates in support items pool: {:#}", item));
        }

        if !self.weights.0.is_finite()
            || !self.weights.1.is_finite()
            || !self.weights.2.is_finite()
            || self.weights.0 < 0.
            || self.weights.1 < 0.
            || self.weights.2 < 0.
        {
            return Err(format!(
                "Weights must be finite and positive (got 'DPS {}, defense {}, mobility {}')",
                self.weights.0, self.weights.1, self.weights.2,
            ));
        }
        if (self.weights.0 == 0.) && (self.weights.1 == 0.) && (self.weights.2 == 0.) {
            return Err("At least one weight must be non-zero".to_string());
        }

        if !self.search_threshold.is_finite() || !(0.0..=1.0).contains(&self.search_threshold) {
            return Err(format!(
                "Search threshold must be greater than 0% and under 100% (got {})",
                100. * self.search_threshold
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct BuildContainer {
    pub build: Build,
    pub cum_utils: EnumSet<ItemUtils>,
    pub golds: [f32; MAX_UNIT_ITEMS + 1], //starting golds + 1 value per item
    pub dps: [f32; MAX_UNIT_ITEMS + 1],   //starting dps + 1 value per item
    pub def: [f32; MAX_UNIT_ITEMS + 1],   //starting defense + 1 value per item
    pub ms: [f32; MAX_UNIT_ITEMS + 1],    //starting ms + 1 value per item
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

impl BuildContainer {
    /// Returns the build score at the given item count.
    #[allow(dead_code)]
    pub fn get_item_slot_score(&self, item_slot: usize, weights: (f32, f32, f32)) -> f32 {
        let normalized_weights: (f32, f32, f32) = get_normalized_weights(weights);
        score_formula_with_normalized_weights(
            self.dps[item_slot],
            self.def[item_slot],
            self.ms[item_slot],
            normalized_weights,
        )
    }

    /// Returns the build score at the given item count.
    /// `weights` must be >= 0 and normalized (their sum must be 1.0) for the formula to be correct.
    #[inline]
    pub(crate) fn _get_item_slot_score_with_normalized_weights(
        &self,
        item_slot: usize,
        normalized_weights: (f32, f32, f32),
    ) -> f32 {
        score_formula_with_normalized_weights(
            self.dps[item_slot],
            self.def[item_slot],
            self.ms[item_slot],
            normalized_weights,
        )
    }

    /// Returns the build average score over the requested item slots.
    #[allow(dead_code)]
    pub fn get_avg_score(&self, n_items: usize, max_golds: f32, weights: (f32, f32, f32)) -> f32 {
        self._get_avg_score_with_normalized_weights(
            n_items,
            max_golds,
            get_normalized_weights(weights),
        )
    }

    /// Returns the build average score over the requested item slots.
    /// `weights` must be >= 0 and normalized (their sum must be 1.0) for the formula to be correct.
    pub(crate) fn _get_avg_score_with_normalized_weights(
        &self,
        n_items: usize,
        max_golds: f32,
        normalized_weights: (f32, f32, f32),
    ) -> f32 {
        //sanity check
        assert!(
            n_items != 0,
            "Number of items to compute average score from must be at least 1"
        );
        let len: usize = n_items + 1;
        let mut scores: Vec<f32> = Vec::with_capacity(len);
        for i in 0..len {
            scores.push(self._get_item_slot_score_with_normalized_weights(i, normalized_weights));
        }
        gold_weighted_average(&scores, &self.golds[0..len], max_golds)
    }
}

#[inline]
pub fn get_normalized_weights(
    (dps_value_weight, def_weight, ms_weight): (f32, f32, f32),
) -> (f32, f32, f32) {
    let sum: f32 = dps_value_weight + def_weight + ms_weight;
    (dps_value_weight / sum, def_weight / sum, ms_weight / sum)
}

/// Formula for the the score of a build.
/// `weights` must be >= 0 and normalized (their sum must be 1.0) for the formula to be correct
/// (these requirements are not checked when calling this function for performance reasons).
#[inline]
fn score_formula_with_normalized_weights(
    dps: f32,
    def: f32,
    ms: f32,
    (norm_dps_value_weight, norm_def_weight, norm_ms_weight): (f32, f32, f32),
) -> f32 {
    dps.powf(norm_dps_value_weight) * def.powf(norm_def_weight) * ms.powf(norm_ms_weight)
}

/// Generate the next 'layer' of builds from current builds, returns None if next layer is empty (never returns an empty Vec).
#[must_use]
fn generate_build_layer(
    current_builds: Vec<BuildContainer>,
    pool: &[&'static Item],
    layer_to_fill_idx: usize,
    normalized_weights: (f32, f32, f32),
) -> Option<Vec<BuildContainer>> {
    let mut new_builds: Vec<BuildContainer> = Vec::with_capacity(current_builds.len()); //new_builds will probably have at least this size
    let mut hashes: FxHashMap<BuildHash, usize> =
        FxHashMap::with_capacity_and_hasher(current_builds.len(), FxBuildHasher);

    let max_golds: f32 = current_builds
        .iter()
        .map(|build| build.golds[layer_to_fill_idx])
        .max_by(|a, b| a.partial_cmp(b).expect("Failed to compare floats"))
        .unwrap_or(STARTING_GOLDS); //needed later

    for container in current_builds {
        for &pool_item in pool {
            let mut candidate: BuildContainer = container.clone();

            //candidate build must have no duplicates
            if candidate.build.iter().any(|&x| *x == *pool_item) {
                continue;
            }
            //candidate build must have no item groups overlap
            candidate.build[layer_to_fill_idx] = pool_item;
            if candidate.build.has_item_groups_overlap() {
                continue;
            }
            //check hash of candidate build
            let candidate_hash: BuildHash = candidate.build.get_hash();
            if let Some(&other_idx) = hashes.get(&candidate_hash) {
                //if hash already exists
                let other: &BuildContainer = &new_builds[other_idx];
                if candidate._get_avg_score_with_normalized_weights(
                    layer_to_fill_idx,
                    max_golds,
                    normalized_weights,
                ) > other._get_avg_score_with_normalized_weights(
                    layer_to_fill_idx,
                    max_golds,
                    normalized_weights,
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
/// The number of elements per chunk will be the most evenly distributed possible.
fn compute_chunk_size(n_elements: usize, thread_count: NonZeroUsize) -> usize {
    usize::max(
        1,
        (n_elements + (thread_count.get() - 1)) / thread_count.get(),
    )
}

fn get_scores_from_sim_results(champ: &Unit, phys_dmg_received_percent: f32) -> (f32, f32, f32) {
    let actual_time: f32 = champ.get_time(); //take champ.time instead of fight_duration in scores calculations, since simulation can be slighlty extended

    let dps: f32 = champ.get_dmg_done().as_sum() / actual_time; //average dps of the unit over the fight simulation

    let effective_hp: f32 = (champ.get_stats().hp
        + champ.get_single_use_heals_shields()
        + DEFAULT_FIGHT_DURATION * champ.get_periodic_heals_shields() / actual_time)
        / (phys_dmg_received_percent * resistance_formula(champ.get_stats().armor)
            + (1. - phys_dmg_received_percent) * resistance_formula(champ.get_stats().mr));

    let move_speed: f32 = champ.get_units_travelled() / actual_time; //average move speed of the unit over the fight simulation

    (dps, effective_hp, move_speed)
}

/// Number of pareto scores to consider. Must be consistent with the number of elements in the `ParetoPoint` type.
/// f32 because only used in f32 calculations.
const N_PARETO_SCORES: f32 = 7.;
struct ParetoSpacePoint {
    utils: EnumSet<ItemUtils>, //represents 3 scores
    golds: f32,
    dps: f32,
    def: f32,
    ms: f32,
}

impl ParetoSpacePoint {
    /// Returns true if self has reasons to be kept against another point, false otherwise.
    /// This doesn't use the usual definition of pareto efficiency but a variation to keep points
    /// that are close to the pareto front as well (up to a given limit, `discard_percent`).
    fn is_pareto_efficient(&self, other: &Self, discard_percent: f32) -> bool {
        //Points with dps, defense and ms close to the pareto front are kept because they
        //can be subject to some little variance between different simulations and scenarios.
        !((self.utils & !other.utils).is_empty())
            || self.golds < other.golds
            || self.dps > discard_percent * other.dps
            || self.def > discard_percent * other.def
            || self.ms > discard_percent * other.ms
    }

    fn from_fight_simulation(
        build: &Build,
        champ: &mut Unit,
        target_stats: &UnitStats,
        settings: &BuildsGenerationSettings,
    ) -> Self {
        champ.set_build_unchecked(*build); //assumes builds have been cheched prior (when generating combinations)
        let mut avg_dps: f32 = 0.;
        let mut avg_def: f32 = 0.;
        let mut avg_ms: f32 = 0.;

        //to avoid combinations of items that are local optimums for the given fight_duration,
        //we simulate for 3 fight durations scattered across a normal distribution around the original fight_duration
        //the final scores are calculated from the weighted sum of each simulation result (weight according to the normal distribution)
        let std_dev: f32 = 0.15 * settings.fight_duration; //chosen arbitrarily, but it works

        //weights for a value at 1.25 std_dev from the mean
        champ.simulate_fight(
            target_stats,
            settings.fight_scenario_number.get() - 1,
            settings.fight_duration - 1.25 * std_dev,
        );
        let (dps, def, ms): (f32, f32, f32) =
            get_scores_from_sim_results(champ, settings.phys_dmg_received_percent);
        avg_dps += 0.25 * dps;
        avg_def += 0.25 * def;
        avg_ms += 0.25 * ms;

        //weights for a value at the mean
        champ.simulate_fight(
            target_stats,
            settings.fight_scenario_number.get() - 1,
            settings.fight_duration,
        );
        let (dps, def, ms): (f32, f32, f32) =
            get_scores_from_sim_results(champ, settings.phys_dmg_received_percent);
        avg_dps += 0.50 * dps;
        avg_def += 0.50 * def;
        avg_ms += 0.50 * ms;

        //weights for a value at 1.25 std_dev from the mean
        champ.simulate_fight(
            target_stats,
            settings.fight_scenario_number.get() - 1,
            settings.fight_duration + 1.25 * std_dev,
        );
        let (dps, def, ms): (f32, f32, f32) =
            get_scores_from_sim_results(champ, settings.phys_dmg_received_percent);
        avg_dps += 0.25 * dps;
        avg_def += 0.25 * def;
        avg_ms += 0.25 * ms;

        Self {
            utils: build
                .iter()
                .map(|item| item.utils)
                .reduce(|acc, utils| acc | utils)
                .unwrap(),
            golds: champ.get_build().cost(),
            dps: avg_dps,
            def: avg_def,
            ms: avg_ms,
        }
    }
}

fn simulate_chunk_of_builds(
    chunk: &[BuildContainer],
    champ: &mut Unit,
    target_stats: &UnitStats,
    settings: &BuildsGenerationSettings,
) -> Vec<ParetoSpacePoint> {
    chunk
        .iter()
        .map(|container| {
            ParetoSpacePoint::from_fight_simulation(&container.build, champ, target_stats, settings)
        })
        .collect()
}

/// returns a Vec<bool> indicating if each point at the corresponding index
/// is pareto efficient compared to the reference point.
fn pareto_compare_chunk_to_point(
    chunk: &[ParetoSpacePoint],
    ref_point: &ParetoSpacePoint,
    discard_percent: f32,
) -> Vec<bool> {
    chunk
        .iter()
        .map(|chunk_point| chunk_point.is_pareto_efficient(ref_point, discard_percent))
        .collect()
}

/// Returns a boolean mask indicating if a given point in part of the pareto front.
/// Also modifies the points Vec in place to only keep pareto points
/// (in the end, remaining values in points are mapped to indices that are true in `pareto_mask`).
/// Multi-threaded.
fn pareto_front_multithread(
    points: &mut Vec<ParetoSpacePoint>,
    discard_percent: f32,
    thread_count: NonZeroUsize,
) -> Vec<bool> {
    let input_len: usize = points.len();
    let mut pareto_mask: Vec<bool> = Vec::with_capacity(input_len);
    let mut pareto_indices: Vec<usize> = (0..input_len).collect();

    let mut idx: usize = 0;
    while idx < points.len() {
        let current_point: &ParetoSpacePoint = &points[idx];

        //update pareto mask, divide points into chunks to process them in parralel
        let chunk_size: usize = compute_chunk_size(points.len(), thread_count);
        pareto_mask.clear();
        pareto_mask = points
            .par_chunks(chunk_size)
            .flat_map_iter(|chunk| {
                pareto_compare_chunk_to_point(chunk, current_point, discard_percent)
            })
            .collect();
        pareto_mask[idx] = true; //keep self

        //pareto_mask.shrink_to_fit(); //useless because we will re-use the full capacity later

        let mut to_keep1 = pareto_mask.iter().copied();
        let mut to_keep2 = to_keep1.clone();
        //i think this can be faster if done in parallel, todo: try scoped threads? (tradeoff might be worth it)
        points.retain(|_| to_keep1.next().unwrap()); //will never panic as to_keep1 has the same length
        pareto_indices.retain(|_| to_keep2.next().unwrap()); //same

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

/// From number of items, returns the associated unit lvl.
#[must_use]
fn lvl_from_number_of_items(n_items: usize, boots_slot: ItemSlot, supp_item_slot: ItemSlot) -> u8 {
    #[allow(clippy::cast_precision_loss)] //`MAX_UNIT_ITEMS` is well within f32 precision range
    let xp_per_non_special_item: f32 = match (boots_slot, supp_item_slot) {
        (ItemSlot::Any, ItemSlot::Any) => {
            XP_PER_LEGENDARY_ITEM * ((MAX_UNIT_ITEMS - 2) as f32) / (MAX_UNIT_ITEMS as f32)
                + XP_PER_BOOTS_ITEM / (MAX_UNIT_ITEMS as f32)
                + XP_PER_SUPP_ITEM / (MAX_UNIT_ITEMS as f32)
        }
        (ItemSlot::Any, _) => {
            XP_PER_LEGENDARY_ITEM * ((MAX_UNIT_ITEMS - 1) as f32) / (MAX_UNIT_ITEMS as f32)
                + XP_PER_BOOTS_ITEM / (MAX_UNIT_ITEMS as f32)
        }
        (_, ItemSlot::Any) => {
            XP_PER_LEGENDARY_ITEM * ((MAX_UNIT_ITEMS - 1) as f32) / (MAX_UNIT_ITEMS as f32)
                + XP_PER_SUPP_ITEM / (MAX_UNIT_ITEMS as f32)
        }
        (_, _) => XP_PER_LEGENDARY_ITEM,
    };

    let boots_slot: usize = if let ItemSlot::Slot(boots_slot_number) = boots_slot {
        boots_slot_number
    } else {
        0
    };
    let supp_item_slot: usize = if let ItemSlot::Slot(supp_item_slot_number) = supp_item_slot {
        supp_item_slot_number
    } else {
        0
    };

    let mut cum_xp: f32 = 0.;
    for i in 1..=n_items {
        if i == boots_slot {
            cum_xp += XP_PER_BOOTS_ITEM;
        } else if i == supp_item_slot {
            cum_xp += XP_PER_SUPP_ITEM;
        } else {
            cum_xp += xp_per_non_special_item;
        }
    }

    let mut lvl: u8 = MIN_UNIT_LVL; //lvl cannot be below `MIN_UNIT_LVL`, so start at this value
    while usize::from(lvl - 1) < MAX_UNIT_LVL - 1
        && cum_xp >= CUM_XP_NEEDED_FOR_LVL_UP_BY_LVL[usize::from(lvl - 1)]
    {
        lvl += 1;
    }
    lvl
}

/// Returns a Vec containing the best builds generated for the given champion.
/// If the generation process fails, return an Err with the corresponding error message.
///
/// Use the silent argument to control if the function shows a progress bar to stdout or not.
pub fn find_best_builds(
    champ_properties: &'static UnitProperties,
    settings: &BuildsGenerationSettings,
    silent: bool,
) -> Result<Vec<BuildContainer>, String> {
    //check input arguments
    settings.check_settings(champ_properties)?;

    //create unit
    let mut champ: Unit = Unit::from_properties_defaults(champ_properties, 6, Build::default())
        .expect("Failed to create unit");
    champ.set_runes(settings.runes_page)?;

    //get number of available threads
    let thread_count: NonZeroUsize =
        std::thread::available_parallelism().expect("Failed to get amount of available threads");

    //start progress bar
    let progress_bar: ProgressBar = if silent {
        ProgressBar::hidden()
    } else {
        let bar: ProgressBar = ProgressBar::new(settings.n_items as u64)
            .with_style(
                ProgressStyle::with_template(
                    "{msg} {spinner} [{elapsed_precise}] {bar} {pos}/{len} items",
                )
                .expect("Failed to create progress bar style"),
            )
            .with_message(format!(
                "Generating best builds for {}",
                champ_properties.name
            ))
            .with_finish(ProgressFinish::AbandonWithMessage(
                format!("Done generating builds for {}", champ_properties.name).into(),
            ));
        bar.enable_steady_tick(Duration::from_millis(200));
        bar
    };

    //create target dummy
    let lvl: u8 = 6; //use lvl 6 for the empty build scores
    let mut target: Unit = Unit::new(
        settings.target_properties,
        OPTIMIZER_DUMMY_RUNES_PAGE,
        OPTIMIZER_DUMMY_SKILL_ORDER,
        lvl,
        Build::default(),
    )
    .expect("Failed to create target dummy");

    //create empty build base scores
    champ.set_lvl(lvl).expect("Failed to set lvl");
    let mut init_build: BuildContainer = BuildContainer {
        build: Build::default(),
        cum_utils: enum_set!(),
        golds: [STARTING_GOLDS; MAX_UNIT_ITEMS + 1],
        dps: [0.; MAX_UNIT_ITEMS + 1],
        def: [0.; MAX_UNIT_ITEMS + 1],
        ms: [0.; MAX_UNIT_ITEMS + 1],
    };
    let init_point: ParetoSpacePoint = ParetoSpacePoint::from_fight_simulation(
        &init_build.build,
        &mut champ,
        target.get_stats(),
        settings,
    );
    init_build.dps[0] = init_point.dps;
    init_build.def[0] = init_point.def;
    init_build.ms[0] = init_point.ms;
    //no need to change other fields

    //initialize best builds generation
    let normalized_weights: (f32, f32, f32) = get_normalized_weights(settings.weights);
    //treat boots/support item as legendary items if no slot specified
    let mut legendary_items: &[&Item] = &settings.legendary_items_pool;
    let mut extended_legendary_items_buffer: Vec<&Item>;
    if settings.boots_slot == ItemSlot::Any {
        extended_legendary_items_buffer = [legendary_items, &settings.boots_pool].concat();
        legendary_items = &extended_legendary_items_buffer;
    }
    if settings.supp_item_slot == ItemSlot::Any {
        extended_legendary_items_buffer = [legendary_items, &settings.supp_items_pool].concat();
        legendary_items = &extended_legendary_items_buffer;
    }
    let discard_percent: f32 = 1. - settings.search_threshold;
    let mut best_builds: Vec<BuildContainer> = vec![init_build];
    //start iterating on each item slot
    for item_idx in (0..settings.n_items).progress_with(progress_bar) {
        let item_slot: usize = item_idx + 1;

        //set champion & dummy lvl
        let lvl: u8 =
            lvl_from_number_of_items(item_slot, settings.boots_slot, settings.supp_item_slot);
        champ.set_lvl(lvl).expect("Failed to set lvl"); //no need to init (automatically done later when simulating fights)
        target.set_lvl(lvl).expect("Failed to set lvl");
        target.init_fight();

        //set item pool
        let mut pool: &[&Item] = &[settings.mandatory_items[item_idx]]; //need to assign temporary value outside of if else brackets
        let pool_without_manaflow_buffer: Vec<&Item>;
        if *settings.mandatory_items[item_idx] == Item::NULL_ITEM {
            pool = if let ItemSlot::Slot(boots_slot_number) = settings.boots_slot {
                if item_slot == boots_slot_number {
                    &settings.boots_pool
                } else {
                    legendary_items
                }
            } else if let ItemSlot::Slot(supp_item_slot_number) = settings.supp_item_slot {
                if item_slot == supp_item_slot_number {
                    &settings.supp_items_pool
                } else {
                    legendary_items
                }
            } else {
                legendary_items
            };

            if item_slot == 1 && !settings.allow_manaflow_first_item {
                pool_without_manaflow_buffer = pool
                    .iter()
                    .filter(|item| !item.item_groups.contains(ItemGroups::Manaflow))
                    .copied()
                    .collect();
                pool = &pool_without_manaflow_buffer;
            }
        }

        //generate next builds layer from pool
        if let Some(new_builds) =
            generate_build_layer(best_builds, pool, item_idx, normalized_weights)
        {
            best_builds = new_builds;
        } else {
            return Err(format!(
                "Can't reach requested item slot (stopped at slot {item_slot} because not enough items in pool/too much items incompatible with each other)"
            ));
        }

        //divide builds into chunks and simulate them in parralel
        let chunk_size: usize = compute_chunk_size(best_builds.len(), thread_count);
        let mut pareto_space_points: Vec<ParetoSpacePoint> = best_builds
            .par_chunks(chunk_size)
            .flat_map_iter(|chunk| {
                simulate_chunk_of_builds(chunk, &mut champ.clone(), target.get_stats(), settings)
            })
            .collect();

        //remove low gold value builds
        let max_gold_value: f32 = pareto_space_points
            .iter()
            .map(|scores| {
                score_formula_with_normalized_weights(
                    scores.dps / scores.golds,
                    scores.def,
                    scores.ms,
                    normalized_weights,
                ) //we divide dps by golds because if dps is perfectly correlated with golds, we want def and ms to be the only deciding factors
            })
            .max_by(|a, b| a.partial_cmp(b).expect("Failed to compare floats"))
            .unwrap(); //points will never be empty (generate_build_layer will return an Err first)
        let mut idx: usize = 0;
        while idx < pareto_space_points.len() {
            let scores: &ParetoSpacePoint = &pareto_space_points[idx];
            if score_formula_with_normalized_weights(
                scores.dps / scores.golds,
                scores.def,
                scores.ms,
                normalized_weights,
            ) < discard_percent * max_gold_value
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
                discard_percent.powf(1. / N_PARETO_SCORES) //heuristic criteria for `N_PARETO_SCORES` dimensions
            },
            thread_count,
        );
        let mut to_keep = pareto_mask.into_iter();
        best_builds.retain(|_| to_keep.next().unwrap()); //will never panic as to_keep has the same length

        //fill remaining build containers
        for (container, scores) in zip(best_builds.iter_mut(), pareto_space_points.iter()) {
            container.cum_utils = scores.utils;
            container.golds[item_slot] = scores.golds;
            container.dps[item_slot] = scores.dps;
            container.def[item_slot] = scores.def;
            container.ms[item_slot] = scores.ms;
        }
    }

    //return builds
    Ok(best_builds)
}

/// Returns a `Vec<(&'static RuneKeystone, f32)>` containing the best runes keystones at the requested number of items with their corresponding score.
/// The keystones are in order (first element == keystone with the best score).
pub fn find_best_runes_keystones(
    champ_properties: &'static UnitProperties,
    settings: &BuildsGenerationSettings,
    n_items: usize,
) -> Result<Vec<(&'static RuneKeystone, f32)>, String> {
    //sanity check
    #[allow(clippy::const_is_empty)]
    if runes_data::ALL_RUNES_KEYSTONES.is_empty() {
        return Err("No runes keystones available to test".to_string());
    }

    let mut test_settings: BuildsGenerationSettings = settings.clone();
    test_settings.n_items = n_items;
    test_settings.weights = get_normalized_weights(test_settings.weights);

    let mut best_keystones: Vec<(&'static RuneKeystone, f32)> = Vec::new();
    for &keystone in runes_data::ALL_RUNES_KEYSTONES.iter() {
        test_settings.runes_page.keystone = keystone;
        let mut best_builds: Vec<BuildContainer> =
            find_best_builds(champ_properties, &test_settings, true)?;
        sort_builds_by_score(&mut best_builds, test_settings.weights);

        let max_golds: f32 = best_builds
            .iter()
            .map(|build| build.golds[n_items])
            .max_by(|a, b| a.partial_cmp(b).expect("Failed to compare floats"))
            .unwrap_or(STARTING_GOLDS);
        let n_to_take: usize = usize::min(5, best_builds.len()); //in case too few generated builds

        #[allow(clippy::cast_precision_loss)] //`n_to_take` is well within f32 precision range
        let avg_score: f32 = (best_builds
            .iter()
            .take(n_to_take)
            .map(|container| {
                container._get_avg_score_with_normalized_weights(
                    n_items,
                    max_golds,
                    test_settings.weights,
                )
            })
            .sum::<f32>())
            / (n_to_take as f32);

        best_keystones.push((keystone, avg_score));
    }
    //sort in reverse order
    best_keystones
        .sort_unstable_by(|k1, k2| (k2.1).partial_cmp(&k1.1).expect("Failed to compare floats"));
    Ok(best_keystones)
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    use super::*;

    #[test]
    pub fn test_default_build_generation_settings() {
        //test for every champion
        for properties in Unit::ALL_CHAMPIONS.iter() {
            if let Err(error_msg) =
                BuildsGenerationSettings::default_by_champion(properties).check_settings(properties)
            {
                panic!(
                    "Default build generation settings for '{}' are not valid: {}",
                    properties.name, error_msg
                )
            }
        }
    }
}
