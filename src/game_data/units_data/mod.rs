mod champions;
mod effects_data;
pub mod items_data;
pub mod runes_data;

use super::*;
use effects_data::*;
use items_data::{items::RUNAANS_HURRICANE_WINDS_FURY_AVG_BOLTS, Build, Item};
use runes_data::RunesPage;

use enum_map::EnumMap;
use enumset::{enum_set, EnumSet, EnumSetType};
use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;

use core::fmt;
use core::num::NonZeroU8;

//todo: lvls NonZeroU8?

//units constants
/// Maximum lvl value of a Unit.
pub const MAX_UNIT_LVL: usize = 18;
/// Minimum lvl value of a Unit in the program (can't be below lvl 6 because we want all abilities to be available).
pub const MIN_UNIT_LVL: u8 = 6;
/// Maximum number of items an Unit can hold.
pub(crate) const MAX_UNIT_ITEMS: usize = 6;
/// Mean missing hp% for a champion, assuming the probability density function for the hp% is 2*x (from x=0 to x=1).
const MEAN_MISSING_HP_PERCENT: f32 = 1. / 3.;

/// Amount of cumulative xp required to reach the given lvl.
pub const CUM_XP_NEEDED_FOR_LVL_UP_BY_LVL: [f32; MAX_UNIT_LVL - 1] = [
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

/// amount of AD one adaptive AP is worth.
const ADAPTIVE_AP_TO_AD_RATIO: f32 = 0.6;

/// Travel distance required to fully charge energized attacks (rapid firecanon, fleet footwork, ...).
const ENERGIZED_ATTACKS_TRAVEL_REQUIRED: f32 = 100. * 24.;
const ENERGIZED_STACKS_PER_BASIC_ATTACK: f32 = 6.;

/// Reference area used to compute the average number of targets hit by basic attacks aoe effects.
/// Should have a value so that an aoe basic attack effect with this range hits on average
/// `AOE_BASIC_ATTACK_REFERENCE_AVG_ADDITIONNAL_TARGETS` additionnal targets.
const AOE_BASIC_ATTACK_REFERENCE_RADIUS: f32 = 500.;
/// Reference number of additionnal targets hit by an aoe basic attack effect
/// with a range of `AOE_BASIC_ATTACK_REFERENCE_RADIUS`.
const AOE_BASIC_ATTACK_REFERENCE_AVG_ADDITIONNAL_TARGETS: f32 = 0.30;

/// From the radius of the aoe basic attack effect, gives the number of targets hit
/// (additionnal to the target that was originally hit by the basic attack).
macro_rules! basic_attack_aoe_effect_avg_additionnal_targets {
    ($radius:expr) => {
        crate::game_data::units_data::AOE_BASIC_ATTACK_REFERENCE_AVG_ADDITIONNAL_TARGETS
            * $radius
            * $radius
            / (crate::game_data::units_data::AOE_BASIC_ATTACK_REFERENCE_RADIUS
                * crate::game_data::units_data::AOE_BASIC_ATTACK_REFERENCE_RADIUS)
    };
}
use basic_attack_aoe_effect_avg_additionnal_targets; //to make it accessible in submodules

//default target dummy properties & stats
const TARGET_DUMMY_BASE_AS: f32 = 0.658;
pub const TARGET_DUMMY_PROPERTIES: UnitProperties = UnitProperties {
    name: "Target dummy",
    as_limit: Unit::DEFAULT_AS_LIMIT,
    as_ratio: TARGET_DUMMY_BASE_AS,
    windup_percent: 0.5,
    windup_modifier: 1.,
    base_stats: UnitStats {
        //in game default values
        hp: 1000.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_percent: 0.,
        armor: 0.,
        mr: 0.,
        base_as: TARGET_DUMMY_BASE_AS,
        bonus_as: 0.,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: Unit::BASE_CRIT_DMG,
        ms_flat: 370.,
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
    growth_stats: UnitStats::const_default(), //no growth stats so they remain constant (lvl doesn't matter)
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
        runes_pages: RunesPage::const_default(),
        skill_order: SkillOrder::const_default(), //does nothing since dummy has null abilities
        legendary_items_pool: &items_data::ALL_LEGENDARY_ITEMS,
        boots_pool: &items_data::ALL_BOOTS,
        supp_items_pool: &items_data::ALL_SUPP_ITEMS,
    },
};

/// From base and growth stat, returns final stat value at the given lvl.
/// <https://leagueoflegends.fandom.com/wiki/Champion_statistic#Growth_statistic_per_level>
#[must_use]
fn growth_stat_formula(lvl: NonZeroU8, base: f32, growth: f32) -> f32 {
    base + growth * (f32::from(lvl.get() - 1)) * (0.7025 + 0.0175 * (f32::from(lvl.get() - 1)))
}

/// Returns final MS of an unit after soft cap.
/// <https://leagueoflegends.fandom.com/wiki/Movement_speed>
#[must_use]
fn capped_ms(raw_ms: f32) -> f32 {
    if raw_ms >= 490. {
        raw_ms * 0.5 + 230.
    } else if raw_ms >= 415. {
        //between 415 and 490
        raw_ms * 0.8 + 83.
    } else if raw_ms >= 220. {
        //between 220 and 415
        raw_ms
    } else if raw_ms >= 0. {
        //between 0 and 220
        110. + raw_ms * 0.5
    } else {
        //under 0
        110. + raw_ms * 0.01
    }
}

/// Returns coefficient multiplying dmg dealt, in the case when resistance stat is positive.
/// <https://leagueoflegends.fandom.com/wiki/Armor> <https://leagueoflegends.fandom.com/wiki/Magic_resistance>
#[must_use]
#[inline]
fn resistance_formula_pos(res: f32) -> f32 {
    100. / (100. + res)
}

/// Returns coefficient multiplying dmg dealt, in the case when resistance stat is negative.
/// <https://leagueoflegends.fandom.com/wiki/Armor> <https://leagueoflegends.fandom.com/wiki/Magic_resistance>
#[must_use]
#[inline]
fn resistance_formula_neg(res: f32) -> f32 {
    2. - 100. / (100. - res)
}

/// Returns coefficient multiplying dmg dealt, automatically choose formula for positive or negative resistance according to the argument.
/// <https://leagueoflegends.fandom.com/wiki/Armor> <https://leagueoflegends.fandom.com/wiki/Magic_resistance>
#[must_use]
pub fn resistance_formula(res: f32) -> f32 {
    if res >= 0. {
        resistance_formula_pos(res)
    } else {
        resistance_formula_neg(res)
    }
}

/// Returns coefficient multiplying base cooldown to give the actual cooldown reduced by haste.
/// <https://leagueoflegends.fandom.com/wiki/Haste>
#[must_use]
#[inline]
fn haste_formula(haste: f32) -> f32 {
    100. / (100. + haste)
}

/// Returns the ideal windup time (= basic attack cast time) for the given unit.
#[must_use]
#[inline]
fn windup_formula(
    windup_percent: f32,
    windup_modifier: f32,
    base_as: f32,
    attack_speed: f32,
) -> f32 {
    (windup_percent / base_as) * (1. - windup_modifier * (1. - base_as / attack_speed))
}

/// From ideal windup time, returns the actual time spent waiting when casting an basic attack.
/// This is to account for player clicks when basic attacking at very high attack speeds, as the player likely waits
/// a bit longer than the ideal windup time before clicking to move again, to avoid cancelling basic attacks.
/// The real windup time cannot go below `TIME_BETWEEN_CLICKS`.
#[must_use]
#[inline]
fn real_windup_time(windup_time: f32) -> f32 {
    windup_time + TIME_BETWEEN_CLICKS * TIME_BETWEEN_CLICKS / (TIME_BETWEEN_CLICKS + windup_time)
}

/// Returns HP given by runes at the given lvl.
/// <https://leagueoflegends.fandom.com/wiki/Rune_(League_of_Legends)#Rune_paths>
#[must_use]
#[inline]
fn runes_hp_by_lvl(lvl: NonZeroU8) -> f32 {
    10. * f32::from(lvl.get())
}

#[inline]
fn increase_multiplicatively_scaling_stat(stat: &mut f32, amount: f32) {
    *stat += (1. - *stat) * amount;
}

#[inline]
fn decrease_multiplicatively_scaling_stat(stat: &mut f32, amount: f32) {
    *stat = 1. - (1. - *stat) / (1. - amount);
}

#[inline]
fn increase_exponentially_scaling_stat(stat: &mut f32, amount: f32) {
    *stat += (1. + *stat) * amount;
}

#[inline]
fn decrease_exponentially_scaling_stat(stat: &mut f32, amount: f32) {
    *stat = (1. + *stat) / (1. + amount) - 1.;
}

#[derive(Debug, Clone)]
pub struct UnitStats {
    pub hp: f32,                   //health points
    pub mana: f32,                 //mana
    pub base_ad: f32,              //base attack damage
    pub bonus_ad: f32,             //bonus attack damage
    pub ap_flat: f32,              //ability power
    pub ap_percent: f32,           //ap coef
    pub armor: f32,                //armor
    pub mr: f32,                   //magic resistance
    pub base_as: f32,              //base attack speed
    pub bonus_as: f32,             //bonus attack speed
    pub ability_haste: f32,        //ability haste
    pub basic_haste: f32,          //basic ability haste (only affects basic abilities)
    pub ultimate_haste: f32,       //ultimate haste (only affects ultimate)
    pub item_haste: f32,           //item haste
    pub crit_chance: f32,          //crit chance
    pub crit_dmg: f32,             //crit damage
    pub ms_flat: f32,              //flat movement speed
    pub ms_percent: f32,           //% movement speed
    pub lethality: f32,            //lethality (kinda "flat armor penetration")
    pub armor_pen_percent: f32,    //% armor penetration, stacks multiplicatively
    pub magic_pen_flat: f32,       //flat magic penetration
    pub magic_pen_percent: f32,    //% magic penetration, stacks multiplicatively
    pub armor_red_flat: f32,       //flat armor reduction
    pub armor_red_percent: f32,    //% armor reduction, stacks multiplicatively
    pub mr_red_flat: f32,          //flat magic reduction
    pub mr_red_percent: f32,       //% magic reduction, stacks multiplicatively
    pub life_steal: f32,           //life steal
    pub omnivamp: f32,             //omnivamp
    pub ability_dmg_modifier: f32, //ability dmg modifier, stacks exponentially
    pub phys_dmg_modifier: f32,    //physical dmg modifier, stacks exponentially
    pub magic_dmg_modifier: f32,   //magic dmg modifier, stacks exponentially
    pub true_dmg_modifier: f32,    //true dmg modifier, stacks exponentially
    pub tot_dmg_modifier: f32,     //total dmg modifier, stacks exponentially
}

impl Default for UnitStats {
    fn default() -> Self {
        Self::const_default()
    }
}

impl UnitStats {
    /// Provides a default value for `UnitStats` usable in compile time constants (unlike `Default::default()` which is not const).
    #[must_use]
    pub const fn const_default() -> Self {
        Self {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 0.,
            mr: 0.,
            base_as: 0.,
            bonus_as: 0.,
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
        }
    }

    /// Returns the total attack damage of the unit. It's a getter function instead of being stored like other stats
    /// because it depends on already existing stats and it could cause bugs if we update one but forget the other.
    #[must_use]
    #[inline]
    pub fn ad(&self) -> f32 {
        self.base_ad + self.bonus_ad
    }

    /// Returns total amount of ap.
    /// It's a getter function instead of being stored like other stats because it depends on
    /// already existing stats and it could cause bugs if we update one but forget the other.
    #[must_use]
    #[inline]
    pub fn ap(&self) -> f32 {
        self.ap_flat * (1. + self.ap_percent)
    }

    /// Returns the attack speed of the unit. It's a getter function instead of being stored like other stats
    /// because it depends on already existing stats and it could cause bugs if we update one but forget the other.
    #[must_use]
    #[inline]
    pub fn attack_speed(&self, as_ratio: f32) -> f32 {
        f32::min(
            Unit::DEFAULT_AS_LIMIT,
            self.base_as + as_ratio * self.bonus_as,
        )
    }

    /// Returns the basic haste (ability haste for basic abilities only) of the unit. It's a getter function instead of being stored like other stats
    /// because it depends on already existing stats and it could cause bugs if we update one but forget the other.
    #[must_use]
    #[inline]
    pub fn ability_haste_basic(&self) -> f32 {
        self.ability_haste + self.basic_haste
    }

    /// Returns the ultimate haste (ability haste for ultimate only) of the unit. It's a getter function instead of being stored like other stats
    /// because it depends on already existing stats and it could cause bugs if we update one but forget the other.
    #[must_use]
    #[inline]
    pub fn ability_haste_ultimate(&self) -> f32 {
        self.ability_haste + self.ultimate_haste
    }

    /// Returns the true movement speed of the unit. It's a getter function instead of being stored like other stats
    /// because it depends on already existing stats and it could cause bugs if we update one but forget the other.
    /// Current code doesn't handle slows.
    #[must_use]
    #[inline]
    pub fn ms(&self) -> f32 {
        capped_ms(self.ms_flat * (1. + self.ms_percent))
    }

    /// Returns the average damage amplification for crit hits. i.e. if a basic attack does 100 dmg without crit,
    /// it will do on average 100 * `self.crit_coef()` when taking crits into account.
    #[must_use]
    #[inline]
    pub fn crit_coef(&self) -> f32 {
        1. + self.crit_chance * (self.crit_dmg - 1.)
    }

    fn add(&mut self, other: &Self) {
        self.hp += other.hp;
        self.mana += other.mana;
        self.base_ad += other.base_ad;
        self.bonus_ad += other.bonus_ad;
        self.ap_flat += other.ap_flat;
        self.ap_percent += other.ap_percent;
        self.armor += other.armor;
        self.mr += other.mr;
        self.base_as += other.base_as;
        self.bonus_as += other.bonus_as;
        self.ability_haste += other.ability_haste;
        self.basic_haste += other.basic_haste;
        self.ultimate_haste += other.ultimate_haste;
        self.item_haste += other.item_haste;

        self.crit_chance += other.crit_chance;
        self.crit_chance = f32::min(1., self.crit_chance); //crit chance capped at 100%

        self.crit_dmg += other.crit_dmg;
        self.ms_flat += other.ms_flat;
        self.ms_percent += other.ms_percent;
        self.lethality += other.lethality;

        increase_multiplicatively_scaling_stat(
            &mut self.armor_pen_percent,
            other.armor_pen_percent,
        ); //stacks multiplicatively
        self.magic_pen_flat += other.magic_pen_flat;
        increase_multiplicatively_scaling_stat(
            &mut self.magic_pen_percent,
            other.magic_pen_percent,
        ); //stacks multiplicatively
        self.armor_red_flat += other.armor_red_flat;
        increase_multiplicatively_scaling_stat(
            &mut self.armor_red_percent,
            other.armor_red_percent,
        ); //stacks multiplicatively
        self.mr_red_flat += other.mr_red_flat;
        increase_multiplicatively_scaling_stat(&mut self.mr_red_percent, other.mr_red_percent); //stacks multiplicatively

        self.life_steal += other.life_steal;
        self.omnivamp += other.omnivamp;

        increase_exponentially_scaling_stat(
            &mut self.ability_dmg_modifier,
            other.ability_dmg_modifier,
        ); //stacks exponentially
        increase_exponentially_scaling_stat(&mut self.phys_dmg_modifier, other.phys_dmg_modifier); //stacks exponentially
        increase_exponentially_scaling_stat(&mut self.magic_dmg_modifier, other.magic_dmg_modifier); //stacks exponentially
        increase_exponentially_scaling_stat(&mut self.true_dmg_modifier, other.true_dmg_modifier); //stacks exponentially
        increase_exponentially_scaling_stat(&mut self.tot_dmg_modifier, other.tot_dmg_modifier);
        //stacks exponentially
    }

    fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug)]
pub(crate) struct BasicAbility {
    /// Returns ability dmg and triggers effects.
    cast: fn(&mut Unit, &UnitStats) -> PartDmg,
    cast_time: f32,
    base_cooldown_by_ability_lvl: [f32; 6], //length 6 to account aphelios case, normal abilities only use the first 5 values
}

#[derive(Debug)]
pub(crate) struct UltimateAbility {
    /// Returns ability dmg and triggers effects.
    /// Should call `Unit.dmg_on_target()` only for the return value at the end of the function !
    cast: fn(&mut Unit, &UnitStats) -> PartDmg,
    cast_time: f32,
    base_cooldown_by_ability_lvl: [f32; 3], //ultimate has 3 lvls
}

pub(crate) type FightScenario = (fn(&mut Unit, &UnitStats, f32), &'static str);

#[derive(Debug)]
pub struct UnitDefaults {
    pub runes_pages: RunesPage,
    pub skill_order: SkillOrder,
    pub legendary_items_pool: &'static [&'static Item],
    pub boots_pool: &'static [&'static Item],
    pub supp_items_pool: &'static [&'static Item],
}

/// Holds properties that don't change at runtime for a given unit.
#[derive(Debug)]
pub struct UnitProperties {
    pub name: &'static str,
    pub as_limit: f32, //as limit of the unit (can be practical limit, e.g. kalista passive is not effective after a certain attack speed value, default as limit is 2.5)
    pub as_ratio: f32, //attack speed ratio, if not specified, same as base AS
    pub windup_percent: f32, //% attack wind up
    pub windup_modifier: f32, //get it from <https://leagueoflegends.fandom.com/wiki/List_of_champions/Basic_attacks>
    pub base_stats: UnitStats,
    pub growth_stats: UnitStats,
    pub(crate) basic_attack: fn(&mut Unit, &UnitStats) -> PartDmg, //returns basic attack dmg and triggers effects
    //no field for passive (implemented directly in the Unit abilities)
    pub q: BasicAbility,
    pub w: BasicAbility,
    pub e: BasicAbility,
    pub r: UltimateAbility,
    pub(crate) on_action_fns: OnActionFns,
    pub(crate) fight_scenarios: &'static [FightScenario],
    pub defaults: UnitDefaults,
}

impl PartialEq for UnitProperties {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name //assumes every Unit name is different, or rather that i'm not too retaaarded to put the same name on different units
    }
}
impl Eq for UnitProperties {}

#[derive(Debug, Clone)]
pub struct SkillOrder {
    //arrays below only hold 0s or 1s, we don't use bools because we need to perform sums over the array and bools are the same size as u8 in rust ¯\_(ツ)_/¯
    pub q: [u8; MAX_UNIT_LVL],
    pub w: [u8; MAX_UNIT_LVL],
    pub e: [u8; MAX_UNIT_LVL],
    pub r: [u8; MAX_UNIT_LVL],
}

impl Default for SkillOrder {
    /// Returns classic skill order with q->w->e lvl-up priority.
    fn default() -> Self {
        Self::const_default()
    }
}

impl SkillOrder {
    /// Returns classic skill order with q->w->e lvl-up priority.
    /// Provides a default valid value for `SkillOrder` usable in compile time constants (unlike `Default::default()` which is not const).
    #[must_use]
    pub const fn const_default() -> Self {
        Self {
            //lvls:
            //  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18
            q: [1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            w: [0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0],
            e: [0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1],
            r: [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0],
        }
    }

    /// Returns Ok if the given `skill_order` is valid, Err with an error message otherwise.
    /// A valid `skill_order` is one with 1 lvl-up per Unit lvl and in total 5 lvl-ups per ability (3 for ultimate).
    /// Aphelios special case is also treated when the `is_aphelios` arg is set to true.
    fn check_skill_order_validity(&self, is_aphelios: bool) -> Result<(), String> {
        //u8 will never overflow since we enforce values in skill order to be only 0s or 1s (=> max sum we can encounter is `MAX_UNIT_LVL`)
        let mut q_sum: u8 = 0;
        let mut w_sum: u8 = 0;
        let mut e_sum: u8 = 0;
        let mut r_sum: u8 = 0;
        for i in 0..self.q.len() {
            //aphelios can lvl up an ability at lvl 6, 11, 16 in addition to his ult (beware i == lvl-1)
            let lvl_ups: u8 = if is_aphelios && (i == 5 || i == 10 || i == 15) {
                2
            } else {
                1
            };
            if (self.q[i] > 1) || (self.w[i] > 1) || (self.e[i] > 1) || (self.r[i] > 1) {
                return Err("Values in skill order must be only 1s or 0s".to_string());
            } else if self.q[i] + self.w[i] + self.e[i] + self.r[i] != lvl_ups {
                return Err(
                    "There should be exactly 1 skill point for each lvl, except for aphelios"
                        .to_string(),
                );
            }
            q_sum += self.q[i];
            w_sum += self.w[i];
            e_sum += self.e[i];
            r_sum += self.r[i];
        }
        let max_ability_lvl: u8 = if is_aphelios { 6 } else { 5 };
        if (q_sum != max_ability_lvl)
            || (w_sum != max_ability_lvl)
            || (e_sum != max_ability_lvl)
            || (r_sum != 3)
        {
            return Err("Wrong number of skill points distributed across abilities".to_string());
        }
        Ok(())
    }
}

/// Holds different function that must be executed on the `Unit` after specific trigger events.
/// Trigger event are basic attacks, ability hits, ...
///
/// The prime use case is on-hit from an item passive :
/// after each auto attack, a function returning the dmg from the on hit passive is called.
///
/// In not specified, for every function in this struct : first argument of type `&Unit` is the direct source of the action provoqued by the trigger event.
/// Second argument of type `&Unit` (if any) is the 'receiver' of the action (e.g. the target for on hit).
///
/// For program correctness, these function should NEVER modify the `Unit` outside of temporary effects and effect variables.
#[derive(Debug)]
#[allow(clippy::type_complexity)]
pub(crate) struct OnActionFns {
    /// Perform specific actions required when setting the Unit lvl (exemple: add veigar passive stacks ap to `lvl_stats`).
    pub(crate) on_lvl_set: Option<fn(&mut Unit)>,

    /// Init `Unit`/`Item` effect variables and temporary effects on the `Unit`. These function should ensure that all effect
    /// variables used later during the fight are properly initialized (in `Unit.effect_values` or `Unit.effects_stacks`).
    /// NEVER use `Unit.stats` as source of stat for effects in these function as it can be modified by previous other init functions
    /// (instead, sum `Unit.lvl_stats` and `Unit.items_stats`).
    pub(crate) on_fight_init: Option<fn(&mut Unit)>,

    /// Triggers special actives and returns dmg done.
    pub(crate) special_active: Option<fn(&mut Unit, &UnitStats) -> PartDmg>,

    /// Applies effects triggered when an ability is casted (updates effect variables accordingly).
    pub(crate) on_ability_cast: Option<fn(&mut Unit)>,
    /// Applies effects triggered when ultimate is casted (additionnal to `on_ability_cast`).
    pub(crate) on_ultimate_cast: Option<fn(&mut Unit)>,

    /// Returns on-ability-hit dmg and applies effects (updates effect variables accordingly).
    /// 3rd argument (f32) is the number of targets hit by the ability (affected by on-ability-hit).
    pub(crate) on_ability_hit: Option<fn(&mut Unit, &UnitStats, f32) -> PartDmg>,
    /// Returns on-ultimate-hit dmg (additionnal to `on_ability_cast`) and applies effects (updates effect variables accordingly).
    /// 3rd argument (f32) is the number of targets hit by the ultimate (affected by on-ultimate-hit).
    pub(crate) on_ultimate_hit: Option<fn(&mut Unit, &UnitStats, f32) -> PartDmg>,

    /// Applies effects triggered when a basic attack is casted (updates effect variables accordingly).
    pub(crate) on_basic_attack_cast: Option<fn(&mut Unit)>,
    /// Returns on-basic_attack-hit dmg and applies effects (updates effect variables accordingly).
    /// 3rd argument (f32) is the number of targets hit by the basic attack (affected by on-basic-attack-hit).
    /// 4th argument (bool) indicates if the function is called internally from other on-action-fns.
    pub(crate) on_basic_attack_hit: Option<fn(&mut Unit, &UnitStats, f32, bool) -> PartDmg>,

    /// Applies effects on the unit triggered when phys dmg is done (updates effect variables accordingly).
    pub(crate) on_phys_hit: Option<fn(&mut Unit)>,
    /// Applies effects on the unit triggered when magic dmg is done (updates effect variables accordingly).
    pub(crate) on_magic_hit: Option<fn(&mut Unit)>,
    /// Applies effects on the unit triggered when true dmg is done (updates effect variables accordingly).
    pub(crate) on_true_dmg_hit: Option<fn(&mut Unit)>,
    /// Applies effects on the unit triggered when any dmg is done (updates effect variables accordingly).
    /// This function is called every hit, in addition to others on_..._hit functions.
    pub(crate) on_any_hit: Option<fn(&mut Unit, &UnitStats) -> PartDmg>,
}

/// This is a struct used as container for holding multiple `OnActionFns`.
/// For the documentation of the fields, see `OnActionFns`.
#[derive(Debug, Clone)]
#[allow(clippy::type_complexity)]
struct OnActionFnsHolder {
    /// For the documentation of the fields, see `OnActionFns`.
    on_lvl_set: Vec<fn(&mut Unit)>,
    /// For the documentation of the fields, see `OnActionFns`.
    on_fight_init: Vec<fn(&mut Unit)>,
    /// For the documentation of the fields, see `OnActionFns`.
    special_active: Vec<fn(&mut Unit, &UnitStats) -> PartDmg>,
    /// For the documentation of the fields, see `OnActionFns`.
    on_ability_cast: Vec<fn(&mut Unit)>,
    /// For the documentation of the fields, see `OnActionFns`.
    on_ultimate_cast: Vec<fn(&mut Unit)>,
    /// For the documentation of the fields, see `OnActionFns`.
    on_ability_hit: Vec<fn(&mut Unit, &UnitStats, f32) -> PartDmg>,
    /// For the documentation of the fields, see `OnActionFns`.
    on_ultimate_hit: Vec<fn(&mut Unit, &UnitStats, f32) -> PartDmg>,
    /// For the documentation of the fields, see `OnActionFns`.
    on_basic_attack_cast: Vec<fn(&mut Unit)>,
    /// For the documentation of the fields, see `OnActionFns`.
    on_basic_attack_hit: Vec<fn(&mut Unit, &UnitStats, f32, bool) -> PartDmg>,
    /// For the documentation of the fields, see `OnActionFns`.
    on_phys_hit: Vec<fn(&mut Unit)>,
    /// For the documentation of the fields, see `OnActionFns`.
    on_magic_hit: Vec<fn(&mut Unit)>,
    /// For the documentation of the fields, see `OnActionFns`.
    on_true_dmg_hit: Vec<fn(&mut Unit)>,
    /// For the documentation of the fields, see `OnActionFns`.
    on_any_hit: Vec<fn(&mut Unit, &UnitStats) -> PartDmg>,
}

impl OnActionFnsHolder {
    /// Add the functions to self.
    fn extend(&mut self, on_action_fns: &OnActionFns) {
        if let Some(function) = on_action_fns.on_lvl_set {
            self.on_lvl_set.push(function);
        }
        if let Some(function) = on_action_fns.on_fight_init {
            self.on_fight_init.push(function);
        }
        if let Some(function) = on_action_fns.special_active {
            self.special_active.push(function);
        }
        if let Some(function) = on_action_fns.on_ability_cast {
            self.on_ability_cast.push(function);
        }
        if let Some(function) = on_action_fns.on_ultimate_cast {
            self.on_ultimate_cast.push(function);
        }
        if let Some(function) = on_action_fns.on_ability_hit {
            self.on_ability_hit.push(function);
        }
        if let Some(function) = on_action_fns.on_ultimate_hit {
            self.on_ultimate_hit.push(function);
        }
        if let Some(function) = on_action_fns.on_basic_attack_cast {
            self.on_basic_attack_cast.push(function);
        }
        if let Some(function) = on_action_fns.on_basic_attack_hit {
            self.on_basic_attack_hit.push(function);
        }
        if let Some(function) = on_action_fns.on_phys_hit {
            self.on_phys_hit.push(function);
        }
        if let Some(function) = on_action_fns.on_magic_hit {
            self.on_magic_hit.push(function);
        }
        if let Some(function) = on_action_fns.on_true_dmg_hit {
            self.on_true_dmg_hit.push(function);
        }
        if let Some(function) = on_action_fns.on_any_hit {
            self.on_any_hit.push(function);
        }
    }

    //clear every function from self.
    fn clear(&mut self) {
        self.on_lvl_set.clear();
        self.on_fight_init.clear();
        self.special_active.clear();
        self.on_ability_cast.clear();
        self.on_ultimate_cast.clear();
        self.on_ability_hit.clear();
        self.on_ultimate_hit.clear();
        self.on_basic_attack_cast.clear();
        self.on_basic_attack_hit.clear();
        self.on_phys_hit.clear();
        self.on_magic_hit.clear();
        self.on_true_dmg_hit.clear();
        self.on_any_hit.clear();
    }
}

impl Unit {
    fn all_on_lvl_set(&mut self) {
        //we iterate over an index because we can't mut borrow self twice (since we pass a mutable reference to on-action-functions)
        //this is hacky but fine as long as the on-action-function doesn't change self.on_action_fns_holder
        let n: usize = self.on_action_fns_holder.on_lvl_set.len();
        for i in 0..n {
            (self.on_action_fns_holder.on_lvl_set[i])(self);
        }
    }

    fn all_on_fight_init(&mut self) {
        //we iterate over an index because we can't mut borrow self twice (since we pass a mutable reference to on-action-functions)
        //this is hacky but fine as long as the on-action-function doesn't change self.on_action_fns_holder
        let n: usize = self.on_action_fns_holder.on_fight_init.len();
        for i in 0..n {
            (self.on_action_fns_holder.on_fight_init[i])(self);
        }
    }

    #[must_use]
    fn all_special_active(&mut self, target_stats: &UnitStats) -> PartDmg {
        //we iterate over an index because we can't mut borrow self twice (since we pass a mutable reference to on-action-functions)
        //this is hacky but fine as long as the on-action-function doesn't change self.on_action_fns_holder
        let n: usize = self.on_action_fns_holder.special_active.len();
        let mut sum: PartDmg = PartDmg(0., 0., 0.);
        for i in 0..n {
            sum += (self.on_action_fns_holder.special_active[i])(self, target_stats);
        }
        sum
    }

    fn all_on_ability_cast(&mut self) {
        //we iterate over an index because we can't mut borrow self twice (since we pass a mutable reference to on-action-functions)
        //this is hacky but fine as long as the on-action-function doesn't change self.on_action_fns_holder
        let n: usize = self.on_action_fns_holder.on_ability_cast.len();
        for i in 0..n {
            (self.on_action_fns_holder.on_ability_cast[i])(self);
        }
    }

    fn all_on_ultimate_cast(&mut self) {
        //we iterate over an index because we can't mut borrow self twice (since we pass a mutable reference to on-action-functions)
        //this is hacky but fine as long as the on-action-function doesn't change self.on_action_fns_holder
        let n: usize = self.on_action_fns_holder.on_ultimate_cast.len();
        for i in 0..n {
            (self.on_action_fns_holder.on_ultimate_cast[i])(self);
        }
    }

    #[must_use]
    fn all_on_ability_hit(&mut self, target_stats: &UnitStats, n_targets: f32) -> PartDmg {
        //we iterate over an index because we can't mut borrow self twice (since we pass a mutable reference to on-action-functions)
        //this is hacky but fine as long as the on-action-function doesn't change self.on_action_fns_holder
        let n: usize = self.on_action_fns_holder.on_ability_hit.len();
        let mut sum: PartDmg = PartDmg(0., 0., 0.);
        for i in 0..n {
            sum += (self.on_action_fns_holder.on_ability_hit[i])(self, target_stats, n_targets);
        }
        sum
    }

    #[must_use]
    fn all_on_ultimate_hit(&mut self, target_stats: &UnitStats, n_targets: f32) -> PartDmg {
        //we iterate over an index because we can't mut borrow self twice (since we pass a mutable reference to on-action-functions)
        //this is hacky but fine as long as the on-action-function doesn't change self.on_action_fns_holder
        let n: usize = self.on_action_fns_holder.on_ultimate_hit.len();
        let mut sum: PartDmg = PartDmg(0., 0., 0.);
        for i in 0..n {
            sum += (self.on_action_fns_holder.on_ultimate_hit[i])(self, target_stats, n_targets);
        }
        sum
    }

    fn all_on_basic_attack_cast(&mut self) {
        //we iterate over an index because we can't mut borrow self twice (since we pass a mutable reference to on-action-functions)
        //this is hacky but fine as long as the on-action-function doesn't change self.on_action_fns_holder
        let n: usize = self.on_action_fns_holder.on_basic_attack_cast.len();
        for i in 0..n {
            (self.on_action_fns_holder.on_basic_attack_cast[i])(self);
        }
    }

    #[must_use]
    fn all_on_basic_attack_hit(
        &mut self,
        target_stats: &UnitStats,
        n_targets: f32,
        from_other_effect: bool,
    ) -> PartDmg {
        //we iterate over an index because we can't mut borrow self twice (since we pass a mutable reference to on-action-functions)
        //this is hacky but fine as long as the on-action-function doesn't change self.on_action_fns_holder
        let n: usize = self.on_action_fns_holder.on_basic_attack_hit.len();
        let mut sum: PartDmg = PartDmg(0., 0., 0.);
        for i in 0..n {
            sum += (self.on_action_fns_holder.on_basic_attack_hit[i])(
                self,
                target_stats,
                n_targets,
                from_other_effect,
            );
        }
        sum
    }

    fn all_on_phys_hit(&mut self) {
        //we iterate over an index because we can't mut borrow self twice (since we pass a mutable reference to on-action-functions)
        //this is hacky but fine as long as the on-action-function doesn't change self.on_action_fns_holder
        let n: usize = self.on_action_fns_holder.on_phys_hit.len();
        for i in 0..n {
            (self.on_action_fns_holder.on_phys_hit[i])(self);
        }
    }

    fn all_on_magic_hit(&mut self) {
        //we iterate over an index because we can't mut borrow self twice (since we pass a mutable reference to on-action-functions)
        //this is hacky but fine as long as the on-action-function doesn't change self.on_action_fns_holder
        let n: usize = self.on_action_fns_holder.on_magic_hit.len();
        for i in 0..n {
            (self.on_action_fns_holder.on_magic_hit[i])(self);
        }
    }

    fn all_on_true_dmg_hit(&mut self) {
        //we iterate over an index because we can't mut borrow self twice (since we pass a mutable reference to on-action-functions)
        //this is hacky but fine as long as the on-action-function doesn't change self.on_action_fns_holder
        let n: usize = self.on_action_fns_holder.on_true_dmg_hit.len();
        for i in 0..n {
            (self.on_action_fns_holder.on_true_dmg_hit[i])(self);
        }
    }

    #[must_use]
    fn all_on_any_hit(&mut self, target_stats: &UnitStats) -> PartDmg {
        //we iterate over an index because we can't mut borrow self twice (since we pass a mutable reference to on-action-functions)
        //this is hacky but fine as long as the on-action-function doesn't change self.on_action_fns_holder
        let n: usize = self.on_action_fns_holder.on_any_hit.len();
        let mut sum: PartDmg = PartDmg(0., 0., 0.);
        for i in 0..n {
            sum += (self.on_action_fns_holder.on_any_hit[i])(self, target_stats);
        }
        sum
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UnitAction {
    BasicAttack,
    Q,
    W,
    E,
    R,
    SpecialActives,
}

impl fmt::Display for UnitAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnitAction::BasicAttack => f.write_str("BasicAttack"),
            UnitAction::Q => f.write_str("Q"),
            UnitAction::W => f.write_str("W"),
            UnitAction::E => f.write_str("E"),
            UnitAction::R => f.write_str("R"),
            UnitAction::SpecialActives => f.write_str("SpecialActives"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Unit {
    //properties
    pub properties: &'static UnitProperties,
    runes_page: RunesPage,
    skill_order: SkillOrder,
    build: Build,

    //stats
    /// Stats that only comes from the Unit base stats (only change with lvl).
    lvl_stats: UnitStats,
    /// Stats that only comes from items (only items stats, stats from effects are updated dynamically in the `stats` field).
    items_stats: UnitStats,
    /// Stats that only comes from runes.
    runes_stats: UnitStats,
    /// Combat stats that gets updated dynamically during combat.
    stats: UnitStats,

    //lvl and abilities lvl
    lvl: NonZeroU8, //this is intentionally NonZeroU8 and not usize, so when used for indexing it reminds you to substract 1 to access array elements
    q_lvl: u8,
    w_lvl: u8,
    e_lvl: u8,
    r_lvl: u8,

    //simulation timings & variables
    time: f32,
    basic_attack_cd: f32,
    q_cd: f32,
    w_cd: f32,
    e_cd: f32,
    r_cd: f32,
    dmg_done: PartDmg,
    periodic_heals_shields: f32, //heals and shields obtained over a duration
    single_use_heals_shields: f32, //heals and shields obtained once
    units_travelled: f32,

    //on action functions
    on_action_fns_holder: OnActionFnsHolder,

    //temporary effects
    effects_stacks: EnumMap<EffectStackId, u8>, //holds various effects integers values on the unit
    effects_values: EnumMap<EffectValueId, f32>, //holds various effects floats values on the unit
    temporary_effects_durations: IndexMap<&'static TemporaryEffect, f32, FxBuildHasher>, //IndexMap of active temporary effects on the unit and their remaining duration
    temporary_effects_cooldowns: IndexMap<&'static TemporaryEffect, f32, FxBuildHasher>, //IndexMap of temporary effects on cooldown on the unit

    //simulation logs
    actions_log: Vec<(f32, UnitAction)>, //records each action performed and at what time in execution order, purely for debug purposes
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Unit '{}', lvl {}, Q-W-E-R lvl: {}-{}-{}-{}, build: {}",
            self.properties.name,
            self.lvl,
            self.q_lvl,
            self.w_lvl,
            self.e_lvl,
            self.r_lvl,
            self.build
        )?;
        writeln!(
            f,
            "time: {:.1}, dmg done: {:.0}, periodic heals & shields : {:.0}, single use heals & shields: {:.0}, units tavelled: {:.0}",
            self.time, self.dmg_done, self.periodic_heals_shields, self.single_use_heals_shields, self.units_travelled
        )?;
        writeln!(f, "hp: {:.0}", self.stats.hp)?;
        writeln!(f, "mana: {:.0}", self.stats.mana)?;
        writeln!(
            f,
            "ad: {:.0} ({:.0} base ad + {:.0} bonus ad)",
            self.stats.ad(),
            self.stats.base_ad,
            self.stats.bonus_ad
        )?;
        writeln!(f, "ap: {:.0}", self.stats.ap())?;
        writeln!(f, "armor: {:.0}", self.stats.armor)?;
        writeln!(f, "mr: {:.0}", self.stats.mr)?;
        writeln!(
            f,
            "attack speed: {:.3} ({:.3} base as + {:.0}% bonus as * {:.3} as ratio, capped at {:.2})",
            self.stats.attack_speed(self.properties.as_ratio),
            self.stats.base_as,
            100. * self.stats.bonus_as,
            self.properties.as_ratio,
            self.properties.as_limit
        )?;
        writeln!(f, "ability haste: {:.0}", self.stats.ability_haste)?;
        writeln!(f, "basic ability haste: {:.0}", self.stats.basic_haste)?;
        writeln!(f, "ultimate haste: {:.0}", self.stats.ultimate_haste)?;
        writeln!(f, "item haste: {:.0}", self.stats.item_haste)?;
        writeln!(f, "crit chance: {:.0}%", 100. * self.stats.crit_chance)?;
        writeln!(f, "crit damage: {:.0}%", 100. * self.stats.crit_dmg)?;
        writeln!(
            f,
            "ms: {:.0} ({:.0} flat ms + {:.0}% ms)",
            self.stats.ms(),
            self.stats.ms_flat,
            100. * self.stats.ms_percent,
        )?;
        writeln!(f, "lethality: {:.0}", self.stats.lethality)?;
        writeln!(
            f,
            "% armor pen: {:.0}%",
            100. * self.stats.armor_pen_percent
        )?;
        writeln!(f, "flat magic pen: {:.0}", self.stats.magic_pen_flat)?;
        writeln!(
            f,
            "% magic pen: {:.0}%",
            100. * self.stats.magic_pen_percent
        )?;
        writeln!(f, "flat armor red: {:.0}", self.stats.armor_red_flat)?;
        writeln!(
            f,
            "% armor red: {:.0}%",
            100. * self.stats.armor_red_percent
        )?;
        writeln!(f, "flat mr red: {:.0}", self.stats.mr_red_flat)?;
        writeln!(f, "% mr red: {:.0}%", 100. * self.stats.mr_red_percent)?;
        writeln!(f, "life steal: {:.0}%", 100. * self.stats.life_steal)?;
        writeln!(f, "omnivamp: {:.0}%", 100. * self.stats.omnivamp)?;
        writeln!(
            f,
            "ability dmg modifier: {:.0}%",
            100. * self.stats.ability_dmg_modifier
        )?;
        writeln!(
            f,
            "physical dmg modifier: {:.0}%",
            100. * self.stats.phys_dmg_modifier
        )?;
        writeln!(
            f,
            "magic dmg modifier: {:.0}%",
            100. * self.stats.magic_dmg_modifier
        )?;
        writeln!(
            f,
            "true dmg modifier: {:.0}%",
            100. * self.stats.true_dmg_modifier
        )?;
        writeln!(
            f,
            "total dmg modifier: {:.0}%",
            100. * self.stats.tot_dmg_modifier
        )
    }
}

/// Indicates the type of a damage instance.
#[derive(EnumSetType, Debug)]
enum DmgTag {
    BasicAttack,
    Ability,
    Ultimate,
}

impl Unit {
    /// base crit damage value for an Unit.
    pub(crate) const BASE_CRIT_DMG: f32 = 1.75;
    /// Default maximum attack speed limit for an Unit.
    pub(crate) const DEFAULT_AS_LIMIT: f32 = 2.5;

    #[allow(dead_code)]
    #[must_use]
    #[inline]
    pub fn get_runes(&self) -> &RunesPage {
        &self.runes_page
    }

    #[allow(dead_code)]
    #[must_use]
    #[inline]
    pub fn get_skill_order(&self) -> &SkillOrder {
        &self.skill_order
    }

    #[must_use]
    #[inline]
    pub fn get_build(&self) -> &Build {
        &self.build
    }

    #[must_use]
    #[inline]
    pub fn get_stats(&self) -> &UnitStats {
        &self.stats
    }

    #[allow(dead_code)]
    #[must_use]
    #[inline]
    pub fn get_lvl(&self) -> NonZeroU8 {
        self.lvl
    }

    #[must_use]
    #[inline]
    pub fn get_time(&self) -> f32 {
        self.time
    }

    #[allow(dead_code)]
    #[must_use]
    #[inline]
    pub fn get_basic_attack_cd(&self) -> f32 {
        self.basic_attack_cd
    }

    #[allow(dead_code)]
    #[must_use]
    #[inline]
    pub fn get_q_cd(&self) -> f32 {
        self.q_cd
    }

    #[allow(dead_code)]
    #[must_use]
    #[inline]
    pub fn get_w_cd(&self) -> f32 {
        self.w_cd
    }

    #[allow(dead_code)]
    #[must_use]
    #[inline]
    pub fn get_e_cd(&self) -> f32 {
        self.e_cd
    }

    #[allow(dead_code)]
    #[must_use]
    #[inline]
    pub fn get_r_cd(&self) -> f32 {
        self.r_cd
    }

    #[allow(dead_code)]
    #[must_use]
    #[inline]
    pub fn get_dmg_done(&self) -> PartDmg {
        self.dmg_done
    }

    #[allow(dead_code)]
    #[must_use]
    #[inline]
    pub fn get_periodic_heals_shields(&self) -> f32 {
        self.periodic_heals_shields
    }

    #[allow(dead_code)]
    #[must_use]
    #[inline]
    pub fn get_single_use_heals_shields(&self) -> f32 {
        self.single_use_heals_shields
    }

    #[allow(dead_code)]
    #[must_use]
    #[inline]
    pub fn get_units_travelled(&self) -> f32 {
        self.units_travelled
    }

    #[allow(dead_code)]
    #[must_use]
    #[inline]
    pub fn get_actions_log(&self) -> &[(f32, UnitAction)] {
        &self.actions_log
    }

    /// Returns true if adaptive force for the unit is physical, false if it is magic.
    /// Adaptive force doesn't count in champions passives, so it only depends on items stats in practise.
    #[must_use]
    #[inline]
    fn adaptive_is_phys(&self) -> bool {
        self.items_stats.bonus_ad >= self.items_stats.ap()
    }

    /// Returns true if adaptive force for the unit is physical, false if it is magic (alternative version, for electrocute, sumon aery, ...).
    /// Adaptive force doesn't count in champions passives, so it only depends on items stats in practise.
    #[allow(dead_code)]
    #[must_use]
    #[inline]
    fn adaptive_is_phys_alternate(&self) -> bool {
        0.10 * self.items_stats.bonus_ad >= 0.05 * self.items_stats.ap()
    }

    /// Sets the Unit level to the request value, returns Ok if success or Err if failure (depending on the validity of the given value).
    /// In case of a failure, the unit is not modified.
    pub fn set_lvl(&mut self, lvl: u8) -> Result<(), String> {
        //these checks are relatively inexpensive
        //return early if lvl is outside of range
        let maybe_lvl: Option<NonZeroU8> = NonZeroU8::new(lvl);
        if !(usize::from(MIN_UNIT_LVL)..=MAX_UNIT_LVL).contains(&usize::from(lvl))
            || maybe_lvl.is_none()
        {
            return Err(format!(
                "Unit lvl must be non zero and between {MIN_UNIT_LVL} and {MAX_UNIT_LVL} (got {lvl})"
            ));
        }

        self.lvl = maybe_lvl.unwrap(); //should never panic since we check for None value above
        self.update_abilities_lvls();

        //update unit lvl stats
        let base: &UnitStats = &self.properties.base_stats;
        let growth: &UnitStats = &self.properties.growth_stats;
        self.lvl_stats.hp = growth_stat_formula(self.lvl, base.hp, growth.hp);
        self.lvl_stats.mana = growth_stat_formula(self.lvl, base.mana, growth.mana);
        self.lvl_stats.base_ad = growth_stat_formula(self.lvl, base.base_ad, growth.base_ad);
        self.lvl_stats.bonus_ad = growth_stat_formula(self.lvl, base.bonus_ad, growth.bonus_ad);
        self.lvl_stats.ap_flat = growth_stat_formula(self.lvl, base.ap_flat, growth.ap_flat);
        self.lvl_stats.ap_percent =
            growth_stat_formula(self.lvl, base.ap_percent, growth.ap_percent);
        self.lvl_stats.armor = growth_stat_formula(self.lvl, base.armor, growth.armor);
        self.lvl_stats.mr = growth_stat_formula(self.lvl, base.mr, growth.mr);
        self.lvl_stats.base_as = growth_stat_formula(self.lvl, base.base_as, growth.base_as);
        self.lvl_stats.bonus_as = growth_stat_formula(self.lvl, base.bonus_as, growth.bonus_as);
        self.lvl_stats.ability_haste =
            growth_stat_formula(self.lvl, base.ability_haste, growth.ability_haste);
        self.lvl_stats.basic_haste =
            growth_stat_formula(self.lvl, base.basic_haste, growth.basic_haste);
        self.lvl_stats.ultimate_haste =
            growth_stat_formula(self.lvl, base.ultimate_haste, growth.ultimate_haste);
        self.lvl_stats.item_haste =
            growth_stat_formula(self.lvl, base.item_haste, growth.item_haste);
        self.lvl_stats.crit_chance = f32::min(
            1.,
            growth_stat_formula(self.lvl, base.crit_chance, growth.crit_chance),
        ); //cr capped at 100%
        self.lvl_stats.crit_dmg = growth_stat_formula(self.lvl, base.crit_dmg, growth.crit_dmg);
        self.lvl_stats.ms_flat = growth_stat_formula(self.lvl, base.ms_flat, growth.ms_flat);
        self.lvl_stats.ms_percent =
            growth_stat_formula(self.lvl, base.ms_percent, growth.ms_percent);
        self.lvl_stats.lethality = growth_stat_formula(self.lvl, base.lethality, growth.lethality);
        self.lvl_stats.armor_pen_percent =
            growth_stat_formula(self.lvl, base.armor_pen_percent, growth.armor_pen_percent);
        self.lvl_stats.magic_pen_flat =
            growth_stat_formula(self.lvl, base.magic_pen_flat, growth.magic_pen_flat);
        self.lvl_stats.magic_pen_percent =
            growth_stat_formula(self.lvl, base.magic_pen_percent, growth.magic_pen_percent);
        self.lvl_stats.armor_red_flat =
            growth_stat_formula(self.lvl, base.armor_red_flat, growth.armor_red_flat);
        self.lvl_stats.armor_red_percent =
            growth_stat_formula(self.lvl, base.armor_red_percent, growth.armor_red_percent);
        self.lvl_stats.mr_red_flat =
            growth_stat_formula(self.lvl, base.mr_red_flat, growth.mr_red_flat);
        self.lvl_stats.mr_red_percent =
            growth_stat_formula(self.lvl, base.mr_red_percent, growth.mr_red_percent);
        self.lvl_stats.life_steal =
            growth_stat_formula(self.lvl, base.life_steal, growth.life_steal);
        self.lvl_stats.omnivamp = growth_stat_formula(self.lvl, base.omnivamp, growth.omnivamp);
        self.lvl_stats.ability_dmg_modifier = growth_stat_formula(
            self.lvl,
            base.ability_dmg_modifier,
            growth.ability_dmg_modifier,
        );
        self.lvl_stats.phys_dmg_modifier =
            growth_stat_formula(self.lvl, base.phys_dmg_modifier, growth.phys_dmg_modifier);
        self.lvl_stats.magic_dmg_modifier =
            growth_stat_formula(self.lvl, base.magic_dmg_modifier, growth.magic_dmg_modifier);
        self.lvl_stats.true_dmg_modifier =
            growth_stat_formula(self.lvl, base.true_dmg_modifier, growth.true_dmg_modifier);
        self.lvl_stats.tot_dmg_modifier =
            growth_stat_formula(self.lvl, base.tot_dmg_modifier, growth.tot_dmg_modifier);

        //perform specific actions required when setting the Unit lvl (exemple: add veigar passive stacks ap to lvl_stats)
        self.all_on_lvl_set();

        Ok(())
    }

    /// Sets the Unit skill order, returns Ok if success or Err if failure (depending on the validity of the given skill order).
    /// In case of a failure, the unit is not modified.
    pub fn set_skill_order(&mut self, skill_order: SkillOrder) -> Result<(), String> {
        //return early if skill_order is not valid
        skill_order.check_skill_order_validity(*self.properties == Unit::APHELIOS_PROPERTIES)?;

        self.skill_order = skill_order;
        self.update_abilities_lvls();

        Ok(())
    }

    /// Updates unit abilities lvl.
    ///
    /// Because they depend on unit lvl, this function is called when setting lvl and skill order.
    /// This leads to redundant work when setting these in chain, but it's not a big deal.
    fn update_abilities_lvls(&mut self) {
        let lvl: usize = usize::from(self.lvl.get());
        self.q_lvl = self.skill_order.q[..lvl].iter().sum();
        self.w_lvl = self.skill_order.w[..lvl].iter().sum();
        self.e_lvl = self.skill_order.e[..lvl].iter().sum();
        self.r_lvl = self.skill_order.r[..lvl].iter().sum();
    }

    /// Clears the items on-action-fns from the unit, leaving only on-action-fns from the unit properties and runes.
    fn clear_items_on_action_fns(&mut self) {
        self.on_action_fns_holder.clear();

        //add base on-action-fns (from unit properties) and runes on-action-fns only
        self.on_action_fns_holder
            .extend(&self.properties.on_action_fns);
        self.on_action_fns_holder
            .extend(&self.runes_page.keystone.on_action_fns);
    }

    /// Clears every on-action-fns from the unit and re-add them.
    fn reload_on_action_fns(&mut self) {
        self.clear_items_on_action_fns();

        //add items on-action-fns
        for item in self.build.iter().filter(|&&item| *item != Item::NULL_ITEM) {
            self.on_action_fns_holder.extend(&item.on_action_fns);
        }
    }

    /// Updates the Unit build, returns Ok if success or Err if failure (depending on the validity of the given build).
    /// In case of a failure, the unit is not modified.
    pub fn set_build(&mut self, build: Build) -> Result<(), String> {
        //these checks are relatively expensive, if calling this function in hot code, consider using `Unit.set_build_unchecked()` instead
        build.check_validity()?;
        self.set_build_unchecked(build);
        Ok(())
    }

    /// Updates the Unit build regardless of its validity (saving some running time by discarding checks).
    /// You must ensure that the given build is valid. Otherwise, this will lead to wrong results when simulating fights with the unit.
    pub(crate) fn set_build_unchecked(&mut self, build: Build) {
        //no build validity check
        self.build = build;

        //clear items from unit
        self.items_stats.clear();
        self.clear_items_on_action_fns();

        //add items one by one to unit
        for item in build.iter().filter(|&&item| *item != Item::NULL_ITEM) {
            self.items_stats.add(&item.stats);
            self.on_action_fns_holder.extend(&item.on_action_fns);
        }
    }

    /// Creates a new Unit with the given properties, runes, skill order, lvl and build.
    /// Return an Err with a corresponding error message if the Unit could not be created because of an invalid argument.
    pub fn new(
        properties: &'static UnitProperties,
        runes_page: RunesPage,
        skill_order: SkillOrder,
        lvl: u8,
        build: Build,
    ) -> Result<Self, String> {
        //perform some checks before creating the Unit
        //we don't want two different abilities happening at the same time so cast time must be >= F32_TOL
        if properties.q.cast_time < F32_TOL
            || properties.w.cast_time < F32_TOL
            || properties.e.cast_time < F32_TOL
            || properties.r.cast_time < F32_TOL
        {
            return Err(format!(
                "{} abilities cast time should be >= F32_TOL",
                properties.name
            ));
        }

        //for similar reasons cooldowns must be >= F32_TOL
        if properties
            .q
            .base_cooldown_by_ability_lvl
            .iter()
            .any(|&cooldown| cooldown < F32_TOL)
        {
            return Err(format!(
                "{} abilities cast time should be >= F32_TOL",
                properties.name
            ));
        }
        if properties
            .w
            .base_cooldown_by_ability_lvl
            .iter()
            .any(|&cooldown| cooldown < F32_TOL)
        {
            return Err(format!(
                "{} W ability cooldown should be >= F32_TOL",
                properties.name
            ));
        }
        if properties
            .e
            .base_cooldown_by_ability_lvl
            .iter()
            .any(|&cooldown| cooldown < F32_TOL)
        {
            return Err(format!(
                "{} E ability cooldown should be >= F32_TOL",
                properties.name
            ));
        }
        if properties
            .r
            .base_cooldown_by_ability_lvl
            .iter()
            .any(|&cooldown| cooldown < F32_TOL)
        {
            return Err(format!(
                "{} R ability cooldown should be >= F32_TOL",
                properties.name
            ));
        }

        if properties.fight_scenarios.is_empty() {
            return Err(format!(
                "{} should have at least 1 fight scenario",
                properties.name
            ));
        }

        //create the unit
        let mut new_unit: Self = Self {
            //properties
            properties,
            runes_page: RunesPage::default(), //temporary value until initialized by setter function
            skill_order: SkillOrder::default(), //temporary value until initialized by setter function (must still be a valid skill order!)
            build: Build::default(),

            //stats
            lvl_stats: UnitStats::default(), //temporary value until initialized by setter function
            items_stats: UnitStats::default(), //temporary value until initialized by setter function
            runes_stats: UnitStats::default(), //temporary value until initialized by setter function
            stats: UnitStats::default(), //temporary value until initialized by setter function

            //lvl and abilities lvl
            lvl: NonZeroU8::new(MIN_UNIT_LVL).expect("Unit lvl cannot be 0"), //temporary value until initialized by setter function
            q_lvl: 0,
            w_lvl: 0,
            e_lvl: 0,
            r_lvl: 0,

            //simulation timings
            time: 0.,
            basic_attack_cd: 0.,
            q_cd: 0.,
            w_cd: 0.,
            e_cd: 0.,
            r_cd: 0.,
            dmg_done: PartDmg(0., 0., 0.),
            periodic_heals_shields: 0.,
            single_use_heals_shields: 0.,
            units_travelled: 0.,

            //on action functions
            on_action_fns_holder: OnActionFnsHolder {
                on_lvl_set: Vec::new(),
                on_fight_init: Vec::new(),
                special_active: Vec::new(),
                on_ability_cast: Vec::new(),
                on_ultimate_cast: Vec::new(),
                on_ability_hit: Vec::new(),
                on_ultimate_hit: Vec::new(),
                on_basic_attack_cast: Vec::new(),
                on_basic_attack_hit: Vec::new(),
                on_phys_hit: Vec::new(),
                on_magic_hit: Vec::new(),
                on_true_dmg_hit: Vec::new(),
                on_any_hit: Vec::new(),
            },

            //temporary effects
            effects_stacks: EnumMap::default(),
            effects_values: EnumMap::default(),
            temporary_effects_durations: IndexMap::with_hasher(FxBuildHasher),
            temporary_effects_cooldowns: IndexMap::with_hasher(FxBuildHasher),

            //simulation logs
            actions_log: Vec::new(),
        };

        //set on-action-fns (done implicitely in `new_unit.set_build` but we do it here as well just in case)
        new_unit.reload_on_action_fns();

        //check and set runes
        new_unit.set_runes(runes_page)?;

        //check and set lvl
        new_unit.set_lvl(lvl)?;

        //check and set skill order (after setting lvl)
        new_unit.set_skill_order(skill_order)?;

        //check and set build
        new_unit.set_build(build)?;

        //init fight so new_unit is ready for simulation
        new_unit.init_fight();
        Ok(new_unit)
    }

    /// Creates a new Unit with the given properties, lvl and build.
    /// The default runes and skill order from the given unit properties are used.
    /// Return an Err with a corresponding error message if the Unit could not be created because of an invalid argument.
    pub fn from_properties_defaults(
        properties: &'static UnitProperties,
        lvl: u8,
        build: Build,
    ) -> Result<Self, String> {
        Self::new(
            properties,
            properties.defaults.runes_pages,
            properties.defaults.skill_order.clone(),
            lvl,
            build,
        )
    }

    /// Creates a new Unit with the properties of a target dummy.
    pub fn new_target_dummy() -> Self {
        Self::from_properties_defaults(&TARGET_DUMMY_PROPERTIES, MIN_UNIT_LVL, Build::default())
            .expect("Failed to create target dummy")
    }

    /// Attempt to add the given effect to the Unit. If the effect is not on cooldown, the function
    /// adds it to the Unit and returns true (or refreshes its duration if already present).
    /// If the effect is on cooldown, it does nothing and returns false.
    ///
    /// The haste argument is to specify which haste value to use for the effect cooldown (ability haste, item haste, ...)
    fn add_temporary_effect(&mut self, effect_ref: &'static TemporaryEffect, haste: f32) -> bool {
        //return early if effect is on cooldown
        if self.temporary_effects_cooldowns.contains_key(effect_ref) {
            return false;
        }

        //store effect remaining duration
        self.temporary_effects_durations
            .insert(effect_ref, effect_ref.duration);

        //store effect cooldown only if cooldown is non-zero (cooldown starts on activation)
        let mut availability_coef: f32 = 1.;
        if effect_ref.cooldown != 0. {
            let real_cooldown: f32 = effect_ref.cooldown * haste_formula(haste);
            self.temporary_effects_cooldowns
                .insert(effect_ref, real_cooldown);
            availability_coef = effect_availability_formula(real_cooldown);
        }

        //add effect stack to the unit
        (effect_ref.add_stack)(self, availability_coef);
        true
    }

    /// Wait immobile for the given amount of time. Removes effects if they expire.
    pub fn wait(&mut self, dt: f32) {
        //sanity check, can be removed
        assert!(
            dt > 0.,
            "Cannot wait for a negative or null amount of time (got {dt})"
        );

        //update time
        self.time += dt;

        //update cooldowns, cannot go below 0
        self.basic_attack_cd = f32::max(0., self.basic_attack_cd - dt);
        self.q_cd = f32::max(0., self.q_cd - dt);
        self.w_cd = f32::max(0., self.w_cd - dt);
        self.e_cd = f32::max(0., self.e_cd - dt);
        self.r_cd = f32::max(0., self.r_cd - dt);

        //update effects cooldowns
        let mut idx: usize = self.temporary_effects_cooldowns.len();
        while idx > 0 {
            idx -= 1;

            //update effect cooldown
            let (_, cooldown_ref) = self.temporary_effects_cooldowns.get_index_mut(idx).unwrap();
            *cooldown_ref -= dt;

            //remove effect from storage if its cooldown ends
            if *cooldown_ref < F32_TOL {
                self.temporary_effects_cooldowns.swap_remove_index(idx);
            }
        }

        //update effects durations, must be done after effects cooldowns to not interfere when effects re-add themselves when removed
        //traverse hash map in reverse index order to allow for effects to re-add themselves when removed (e.g.: kindred wolf's frenzy)
        let mut idx: usize = self.temporary_effects_durations.len();
        while idx > 0 {
            idx -= 1;

            //update effect duration
            let (&effect_ref, duration_ref) =
                self.temporary_effects_durations.get_index_mut(idx).unwrap();
            *duration_ref -= dt;

            //remove effect from the unit if its duration ends
            if *duration_ref < F32_TOL {
                self.temporary_effects_durations.swap_remove_index(idx);
                (effect_ref.remove_every_stack)(self); //call after removing effect from hashmap so it can re-add itself
            }
        }
    }

    /// Move for the given amount of time. Removes effects if they expire and add distance to `self.simulation_results.units_travelled`.
    pub fn walk(&mut self, mut dt: f32) {
        while dt >= F32_TOL {
            //find minimum time until next expiring effect
            let min_duration: f32 = *self
                .temporary_effects_durations
                .values()
                .chain(core::iter::once(&dt))
                .min_by(|a, b| a.partial_cmp(b).expect("Failed to compare floats"))
                .unwrap(); //will never panic as we chain with once so there is at minimum 1 element

            //walk until next expiring effect
            self.units_travelled += self.stats.ms() * min_duration; //must be before self.wait() to still benefit from temporary effects
            self.wait(min_duration);
            dt -= min_duration;
        }
    }

    pub fn init_fight(&mut self) {
        //simulation timings & variables
        self.time = 0.;
        self.basic_attack_cd = 0.;
        self.q_cd = 0.;
        self.w_cd = 0.;
        self.e_cd = 0.;
        self.r_cd = 0.;
        self.dmg_done = PartDmg(0., 0., 0.);
        self.periodic_heals_shields = 0.;
        self.single_use_heals_shields = 0.;
        self.units_travelled = 0.;

        //init stats (runes are done later, need to do it after items passives init)
        self.stats.clear();
        self.stats.add(&self.lvl_stats);
        self.stats.add(&self.items_stats);

        //reset temporary effects
        self.effects_stacks.clear(); //this is not really needed since we init the variables later, but we do it to clear unused variables for debugging convenience
        self.effects_values.clear(); //same as above
        self.temporary_effects_durations.clear(); //this is needed to remove every temporary effects
        self.temporary_effects_cooldowns.clear(); //same as above

        //init effect variables and temporary effects on the unit (after effects reset)
        self.all_on_fight_init();

        //runes stats (after items passives init)
        self.update_runes_stats();
        self.stats.add(&self.runes_stats);

        //reset actions logs
        self.actions_log.clear();
    }

    /// From partial dmg (separated ad, ap & true dmg values without taking resistances into account),
    /// returns the post mitigation dmg received by the target. Also stacks passive effects.
    ///
    /// Since this is a relatively expensive function to run, try to call it as little as possible and use
    /// `n_targets`, `n_dmg_instances`, `n_stacking_instances` arguments to regroup multiple sources of dmg
    /// that happens at the same time or are from the same ability, ...
    ///
    /// ARGUMENTS :
    ///
    /// - self: attacking Unit.
    ///
    /// - `target_stats`: target stats used for the dmg calculations.
    ///
    /// - (`phys_dmg`, `magic_dmg`, `true_dmg)`: dmg used to calculate final dmg on the target.
    ///
    /// - (`n_dmg_instances`, `n_stacking_instances)`:
    ///     - `n_dmg_instances`: number of dmg instances on one of the targets considered for items on ad/magic dmg effects
    ///       (e.g. black cleaver, affects items on ad/magic dmg effects ONLY, doesn't concern items on-basic-attack/on-ability-hit effects).
    ///     - `n_stacking_instances`: number of stacking instances on a single target considered for items on-basic-attack/on-ability-hit effects stacking.
    ///       Must be less or equal to `n_dmg_instances`.
    ///       (affects items on-basic-attack/on-ability-hit effects ONLY, doesn't concern items on ad/magic dmg effects).
    ///       `n_dmg_instances` and `n_stacking_instances` are needed sperately, exemple:
    ///       Ashe q arrows stack black cleaver fully (`on_phys_hit`), but gives only 1 stack of kraken slayer (`on_basic_attack_hit`).
    ///       /!\ `n_dmg_instances` must always be greater than `n_stacking_instances` (else it is a logic error).
    ///
    /// - `dmg_tags`: types of the instance of dmg, applies effects if it contains specific tags (e.g. on-ability-hit).
    ///
    /// - `n_targets`: number of targets hit, affects items on-basic-attack/on-ability-hit effects ONLY
    ///   (dmg received by this function must already be the sum on all targets).
    fn dmg_on_target(
        &mut self,
        target_stats: &UnitStats,
        mut part_dmg: PartDmg,
        (n_dmg_instances, n_stacking_instances): (u8, u8),
        dmg_tags: EnumSet<DmgTag>,
        n_targets: f32,
    ) -> PartDmg {
        //calculation order: flat res reduction -> % res reduction -> % res penetration -> flat res penetration (i.e. lethality for armor)
        //calculate res before applying effects
        let mut virtual_armor: f32 = target_stats.armor - self.stats.armor_red_flat; //flat armor reduction, can reduce armor below 0
        let armor_coef: f32;
        if virtual_armor > 0. {
            //% armor reduction, % armor penetration and lethality cannot reduce armor below 0
            virtual_armor *= 1. - self.stats.armor_red_percent; //% armor reduction
            virtual_armor *= 1. - self.stats.armor_pen_percent; //% armor penetration
            virtual_armor = f32::max(0., virtual_armor - self.stats.lethality); //lethality, cannot reduce armor below 0

            armor_coef = resistance_formula_pos(virtual_armor);
        } else {
            armor_coef = resistance_formula_neg(virtual_armor);
        }

        let mut virtual_mr: f32 = target_stats.mr - self.stats.mr_red_flat; //flat mr reduction, can reduce mr below 0
        let mr_coef: f32;
        if virtual_mr > 0. {
            //% mr reduction, % magic penetration and flat magic penetration cannot reduce mr below 0
            virtual_mr *= 1. - self.stats.mr_red_percent; //% mr reduction
            virtual_mr *= 1. - self.stats.magic_pen_percent; //% magic penetration
            virtual_mr = f32::max(0., virtual_mr - self.stats.magic_pen_flat); //flat magic pen, cannot reduce mr below 0

            mr_coef = resistance_formula_pos(virtual_mr);
        } else {
            mr_coef = resistance_formula_neg(virtual_mr);
        }

        //use stats values before they get modified by effects
        let life_steal: f32 = self.stats.life_steal;
        let omnivamp: f32 = self.stats.omnivamp;
        let ability_dmg_modifier: f32 = self.stats.ability_dmg_modifier;
        let phys_dmg_modifier: f32 = self.stats.phys_dmg_modifier;
        let magic_dmg_modifier: f32 = self.stats.magic_dmg_modifier;
        let true_dmg_modifier: f32 = self.stats.true_dmg_modifier;
        let tot_dmg_modifier: f32 = self.stats.tot_dmg_modifier;

        //on ability hit and ability coef, must be done before on-basic-attack-hit (because of muramana shock that applies ability part first & conqueror...)
        if dmg_tags.contains(DmgTag::Ability) {
            //ability dmg modifier (as of patch 14.19, it doesn't affect on_ability_hit and on_ultimate_hit dmg anymore)
            part_dmg *= 1. + ability_dmg_modifier;

            //on ability hit
            for _ in 0..n_stacking_instances {
                part_dmg += self.all_on_ability_hit(target_stats, n_targets);
            }
        }

        //on ultimate hit
        for _ in 0..n_stacking_instances {
            part_dmg += self.all_on_ultimate_hit(target_stats, n_targets);
        }

        //on basic attack hit, must be done after on-basic-attack-hit (because of muramana shock that applies ability part first)
        if dmg_tags.contains(DmgTag::BasicAttack) {
            //runaans increases the number of targets hit by on-basic-attack-hit
            //exceptionally, use runaans variables here (shouldn't because outside of module, but I didn't find a better way)
            let basic_attack_n_targets: f32 = if self.build.contains(&&Item::RUNAANS_HURRICANE) {
                n_targets * (1. + RUNAANS_HURRICANE_WINDS_FURY_AVG_BOLTS)
            } else {
                n_targets
            };
            for _ in 0..n_stacking_instances {
                part_dmg +=
                    self.all_on_basic_attack_hit(target_stats, basic_attack_n_targets, false);
            }
        }

        //on phys dmg
        if part_dmg.0 > 0. {
            for _ in 0..n_dmg_instances {
                self.all_on_phys_hit();
            }
        }

        //on magic dmg
        if part_dmg.1 > 0. {
            for _ in 0..n_dmg_instances {
                self.all_on_magic_hit();
            }
        }

        //on true dmg
        if part_dmg.2 > 0. {
            for _ in 0..n_dmg_instances {
                self.all_on_true_dmg_hit();
            }
        }

        //on any hit (must be done after on-basic-attack-hit because of conqueror...)
        for _ in 0..n_stacking_instances {
            part_dmg += self.all_on_any_hit(target_stats);
        }

        self.time += F32_TOL; //to differentiate different dmg instances

        //dmg modifiers
        part_dmg.0 *= armor_coef * (1. + phys_dmg_modifier);
        part_dmg.1 *= mr_coef * (1. + magic_dmg_modifier);
        part_dmg.2 *= 1. + true_dmg_modifier;
        part_dmg *= 1. + tot_dmg_modifier;

        //update simulation logs
        let tot_dmg: f32 = part_dmg.as_sum();
        //omnivamp
        self.periodic_heals_shields += tot_dmg * omnivamp;
        //lifesteal
        if dmg_tags.contains(DmgTag::BasicAttack) {
            self.periodic_heals_shields += tot_dmg * life_steal;
        }

        //dmg done
        self.dmg_done += part_dmg;
        part_dmg
    }

    /// Triggers every item actives on the unit and returns dmg done.
    pub fn use_all_special_actives(&mut self, target_stats: &UnitStats) -> PartDmg {
        //save log
        self.actions_log
            .push((self.time, UnitAction::SpecialActives));

        self.all_special_active(target_stats)
    }

    /// Performs a basic attack and returns dmg done.
    pub fn basic_attack(&mut self, target_stats: &UnitStats) -> PartDmg {
        //save log
        self.actions_log.push((self.time, UnitAction::BasicAttack));

        //wait cast (windup) time
        let windup_time: f32 = real_windup_time(windup_formula(
            self.properties.windup_percent,
            self.properties.windup_modifier,
            self.stats.base_as,
            self.stats.attack_speed(self.properties.as_ratio),
        ));
        self.wait(windup_time);

        //on basic attack cast effects
        self.all_on_basic_attack_cast();

        //set cd
        self.basic_attack_cd = f32::max(
            0.,
            1. / f32::min(
                self.properties.as_limit,
                self.stats.attack_speed(self.properties.as_ratio),
            ) - windup_time,
        ); //limit as cd to the unit as limit

        //return dmg
        (self.properties.basic_attack)(self, target_stats)
    }

    /// cast q and returns dmg done.
    pub fn q(&mut self, target_stats: &UnitStats) -> PartDmg {
        //save log
        self.actions_log.push((self.time, UnitAction::Q));

        //wait cast time
        self.wait(self.properties.q.cast_time);

        //on ability cast effects
        self.all_on_ability_cast();

        //set cd
        self.q_cd = haste_formula(self.stats.ability_haste_basic())
            * self.properties.q.base_cooldown_by_ability_lvl[usize::from(self.q_lvl - 1)];

        //return dmg
        (self.properties.q.cast)(self, target_stats)
    }

    /// cast w and returns dmg done.
    pub fn w(&mut self, target_stats: &UnitStats) -> PartDmg {
        //save log
        self.actions_log.push((self.time, UnitAction::W));

        //wait cast time
        self.wait(self.properties.w.cast_time);

        //on ability cast effects
        self.all_on_ability_cast();

        //set cd
        self.w_cd = haste_formula(self.stats.ability_haste_basic())
            * self.properties.w.base_cooldown_by_ability_lvl[usize::from(self.w_lvl - 1)];

        //return dmg
        (self.properties.w.cast)(self, target_stats)
    }

    /// cast e and returns dmg done.
    pub fn e(&mut self, target_stats: &UnitStats) -> PartDmg {
        //save log
        self.actions_log.push((self.time, UnitAction::E));

        //wait cast time
        self.wait(self.properties.e.cast_time);

        //on ability cast effects
        self.all_on_ability_cast();

        //set cd
        self.e_cd = haste_formula(self.stats.ability_haste_basic())
            * self.properties.e.base_cooldown_by_ability_lvl[usize::from(self.e_lvl - 1)];

        //return dmg
        (self.properties.e.cast)(self, target_stats)
    }

    /// cast r and returns dmg done.
    pub fn r(&mut self, target_stats: &UnitStats) -> PartDmg {
        //save log
        self.actions_log.push((self.time, UnitAction::R));

        //wait cast time
        self.wait(self.properties.r.cast_time);

        //on ability and ultimate cast effects
        self.all_on_ability_cast();
        self.all_on_ultimate_cast();

        //set cd
        self.r_cd = haste_formula(self.stats.ability_haste_ultimate())
            * self.properties.r.base_cooldown_by_ability_lvl[usize::from(self.r_lvl - 1)];

        //return dmg
        (self.properties.r.cast)(self, target_stats)
    }

    /// Same as casting r except the dmg, units travelled, etc. during the r are all reduced
    /// according to the availability formula (to account for the r cooldown).
    /// Useless to use for ultimates that only adds a effect.
    pub fn weighted_r(&mut self, target_stats: &UnitStats) -> PartDmg {
        let phys_dmg_done_before_r: f32 = self.dmg_done.0;
        let magic_dmg_done_before_r: f32 = self.dmg_done.1;
        let true_dmg_done_before_r: f32 = self.dmg_done.2;

        let periodic_heals_shields_before_r: f32 = self.periodic_heals_shields;
        let single_use_heals_shields_before_r: f32 = self.single_use_heals_shields;
        let units_travelled_before_r: f32 = self.units_travelled;
        self.r(target_stats);
        let percent_to_remove: f32 = 1. - effect_availability_formula(self.r_cd);

        let phys_dmg: f32 = self.dmg_done.0 - phys_dmg_done_before_r;
        let magic_dmg: f32 = self.dmg_done.1 - magic_dmg_done_before_r;
        let true_dmg: f32 = self.dmg_done.2 - true_dmg_done_before_r;
        self.dmg_done.0 -= percent_to_remove * phys_dmg;
        self.dmg_done.1 -= percent_to_remove * magic_dmg;
        self.dmg_done.2 -= percent_to_remove * true_dmg;

        self.periodic_heals_shields -=
            percent_to_remove * (self.periodic_heals_shields - periodic_heals_shields_before_r);
        self.single_use_heals_shields -=
            percent_to_remove * (self.single_use_heals_shields - single_use_heals_shields_before_r);
        self.units_travelled -=
            percent_to_remove * (self.units_travelled - units_travelled_before_r);

        let percent_to_keep = 1. - percent_to_remove;
        PartDmg(
            percent_to_keep * phys_dmg,
            percent_to_keep * magic_dmg,
            percent_to_keep * true_dmg,
        ) //return weighted dmg
    }

    /// Simulate a fight for the unit hitting the specified target according to what is defined in the unit properties
    /// and returns (average dps, effective hp, average move speed) obtained from the simulation.
    /// This function will always start by initializing the unit with `self.init_fight` and use all items actives before simulating.
    pub fn simulate_fight(&mut self, target_stats: &UnitStats, index: usize, fight_duration: f32) {
        //sanity check
        assert!(
            index < self.properties.fight_scenarios.len(),
            "Fight scenario index is out of bounds"
        );

        self.init_fight();
        self.use_all_special_actives(target_stats);
        (self.properties.fight_scenarios[index].0)(self, target_stats, fight_duration);
    }
}

/// Default `basic_attack` for an unit.
fn default_basic_attack(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    let phys_dmg: f32 = champ.stats.ad() * champ.stats.crit_coef();
    champ.dmg_on_target(
        target_stats,
        PartDmg(phys_dmg, 0., 0.),
        (1, 1),
        enum_set!(DmgTag::BasicAttack),
        1.,
    )
}

/// For performance reasons, we use a `null_basic_attack` function (that should never be called and will panic if so) instead of an Option, for units that do not have one.
///
/// This is to avoid checking an Option everytime a `basic_attack` is called, since the majority of basic attacks aren't null
/// and the user should know in advance if said unit `basic_attack` is null or not.
pub(crate) fn null_basic_attack(_champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    unreachable!("Null_basic_attack was called");
}

/// For performance reasons, we use a `NULL_BASIC_ABILITY` constant (that should never be used) instead of an Option, for units that do not have one.
///
/// This is to avoid checking an Option everytime an ability is called, since the majority of abilities aren't null
/// and the user should know in advance if said unit ability is null or not.
pub(crate) const NULL_BASIC_ABILITY: BasicAbility = BasicAbility {
    cast: null_ability_cast,
    cast_time: F32_TOL,
    base_cooldown_by_ability_lvl: [F32_TOL, F32_TOL, F32_TOL, F32_TOL, F32_TOL, F32_TOL],
};

/// For performance reasons, we use a `NULL_ULTIMATE_ABILITY` constant (that should never be used) instead of an Option, for units that do not have one.
///
/// This is to avoid checking an Option everytime an ability is called, since the majority of abilities aren't null
/// and the user should know in advance if said unit ability is null or not.
pub(crate) const NULL_ULTIMATE_ABILITY: UltimateAbility = UltimateAbility {
    cast: null_ability_cast,
    cast_time: F32_TOL,
    base_cooldown_by_ability_lvl: [F32_TOL, F32_TOL, F32_TOL],
};

fn null_ability_cast(_champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    unreachable!("null_ability_cast was called");
}

/// For performance reasons, we use a `null_simulate_fight` function (that should never be called and will panic if so) instead of an Option, for units that do not have one.
///
/// This is to avoid checking an Option everytime a `simulate_fight` is called, since the majority of `simulate_fight` aren't null
/// and the user should know in advance if said unit `simulate_fight` is null or not.
pub(crate) fn null_simulate_fight(_champ: &mut Unit, _target_stats: &UnitStats, _time_limit: f32) {
    unreachable!("null_simulate_fight was called");
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    use super::*;

    /// Test that the target dummy properties are valid.
    #[test]
    pub fn test_target_dummy_properties() {
        Unit::new_target_dummy(); //can panic inside if `TARGET_DUMMY_PROPERTIES` is invalid
    }
}
