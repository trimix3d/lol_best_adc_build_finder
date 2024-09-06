mod champions;
pub mod runes_data;

use super::{
    buffs_data::*,
    effect_availability_formula, haste_formula,
    items_data::{items::NULL_ITEM, Build, Item},
    F32_TOL, TIME_BETWEEN_CLICKS,
};

use enum_map::EnumMap;
use indexmap::IndexMap;
use runes_data::{RuneShard, RunesPage};
use rustc_hash::FxBuildHasher;
use std::num::NonZeroU8;

//units constants
/// Maximum lvl value of a Unit.
pub const MAX_UNIT_LVL: usize = 18;
/// Minimum lvl value of a Unit in the program (can't be below lvl 6 because we want all spells to be available).
pub const MIN_UNIT_LVL: u8 = 6;
/// Maximum number of items an Unit can hold.
pub const MAX_UNIT_ITEMS: usize = 6;

/// From base and growth stat, returns final stat value at the given lvl.
/// <https://leagueoflegends.fandom.com/wiki/Champion_statistic#Growth_statistic_per_level>
fn growth_stat_formula(lvl: NonZeroU8, base: f32, growth: f32) -> f32 {
    base + growth * (f32::from(lvl.get() - 1)) * (0.7025 + 0.0175 * (f32::from(lvl.get() - 1)))
}

/// Returns final MS of an unit after soft cap.
/// <https://leagueoflegends.fandom.com/wiki/Movement_speed>
#[inline]
#[must_use]
pub fn capped_ms(raw_ms: f32) -> f32 {
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

/// From flat ap and ap coef, returns total ap amount.
fn ap_formula(ap_flat: f32, ap_coef: f32) -> f32 {
    ap_flat * (1. + ap_coef)
}

/// Returns coefficient multiplying dmg dealt, in the case when resistance stat is positive.
/// <https://leagueoflegends.fandom.com/wiki/Armor> <https://leagueoflegends.fandom.com/wiki/Magic_resistance>
#[inline]
#[must_use]
pub fn resistance_formula_pos(res: f32) -> f32 {
    100. / (100. + res)
}

/// Returns coefficient multiplying dmg dealt, in the case when resistance stat is negative.
/// <https://leagueoflegends.fandom.com/wiki/Armor> <https://leagueoflegends.fandom.com/wiki/Magic_resistance>
#[inline]
#[must_use]
pub fn resistance_formula_neg(res: f32) -> f32 {
    2. - 100. / (100. - res)
}

/// Returns the ideal windup time (= basic attack cast time) for the given unit.
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
fn real_windup_time(windup_time: f32) -> f32 {
    windup_time + TIME_BETWEEN_CLICKS * TIME_BETWEEN_CLICKS / (TIME_BETWEEN_CLICKS + windup_time)
}

/// Returns HP given by runes at the given lvl.
/// <https://leagueoflegends.fandom.com/wiki/Rune_(League_of_Legends)#Rune_paths>
fn runes_hp_by_lvl(lvl: NonZeroU8) -> f32 {
    10. * f32::from(lvl.get())
}

#[derive(Debug, Clone, Copy)]
pub struct UnitStats {
    pub hp: f32,                //health points
    pub mana: f32,              //mana
    pub base_ad: f32,           //base attack damage
    pub bonus_ad: f32,          //bonus attack damage
    pub ap_flat: f32,           //ability power
    pub ap_coef: f32,           //ap coef
    pub armor: f32,             //armor
    pub mr: f32,                //magic resistance
    pub base_as: f32,           //base attack speed
    pub bonus_as: f32,          //bonus attack speed
    pub ability_haste: f32,     //ability haste
    pub basic_haste: f32,       //basic ability haste (only affects basic spells)
    pub ultimate_haste: f32,    //ultimate haste (only affects ultimate)
    pub item_haste: f32,        //item haste
    pub crit_chance: f32,       //crit chance
    pub crit_dmg: f32,          //crit damage
    pub ms_flat: f32,           //flat movement speed
    pub ms_percent: f32,        //% movement speed
    pub lethality: f32,         //lethality (kinda "flat armor penetration")
    pub armor_pen_percent: f32, //% armor penetration
    pub magic_pen_flat: f32,    //flat magic penetration
    pub magic_pen_percent: f32, //% magic penetration
    pub armor_red_flat: f32,    //flat armor reduction
    pub armor_red_percent: f32, //% armor reduction
    pub mr_red_flat: f32,       //flat magic reduction
    pub mr_red_percent: f32,    //% magic reduction
    pub life_steal: f32,        //life steal
    pub omnivamp: f32,          //omnivamp
}

impl Default for UnitStats {
    fn default() -> Self {
        UnitStats::const_default()
    }
}

impl UnitStats {
    /// Provides a default value for `UnitStats` usable in compile time constants (unlike `Default::default()` which is not const).
    #[must_use]
    pub const fn const_default() -> UnitStats {
        UnitStats {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_coef: 0.,
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
        }
    }

    /// Returns the total attack damage of the unit. It's a getter function instead of being stored like other stats
    /// because it depends on already existing stats and it could cause bugs if we update one but forget the other.
    #[inline]
    #[must_use]
    pub fn ad(&self) -> f32 {
        self.base_ad + self.bonus_ad
    }

    /// Returns the ap coefficient of the unit (like rabadons magical opus coef).
    /// It's a getter function instead of being stored like other stats because it depends on
    /// already existing stats and it could cause bugs if we update one but forget the other.
    #[inline]
    #[must_use]
    pub fn ap(&self) -> f32 {
        ap_formula(self.ap_flat, self.ap_coef)
    }

    /// Returns the attack speed of the unit. It's a getter function instead of being stored like other stats
    /// because it depends on already existing stats and it could cause bugs if we update one but forget the other.
    #[inline]
    #[must_use]
    pub fn attack_speed(&self, as_ratio: f32) -> f32 {
        f32::min(
            Unit::DEFAULT_AS_LIMIT,
            self.base_as + as_ratio * self.bonus_as,
        )
    }

    /// Returns the basic haste (ability haste for basic spells only) of the unit. It's a getter function instead of being stored like other stats
    /// because it depends on already existing stats and it could cause bugs if we update one but forget the other.
    #[inline]
    #[must_use]
    pub fn ability_haste_basic(&self) -> f32 {
        self.ability_haste + self.basic_haste
    }

    /// Returns the ultimate haste (ability haste for ultimate only) of the unit. It's a getter function instead of being stored like other stats
    /// because it depends on already existing stats and it could cause bugs if we update one but forget the other.
    #[inline]
    #[must_use]
    pub fn ability_haste_ultimate(&self) -> f32 {
        self.ability_haste + self.ultimate_haste
    }

    /// Returns the true movement speed of the unit. It's a getter function instead of being stored like other stats
    /// because it depends on already existing stats and it could cause bugs if we update one but forget the other.
    /// Doesn't check and will give wrong results if it receives incoherent stats values (e.g. negative slow percent, ...)
    #[inline]
    #[must_use]
    pub fn ms(&self) -> f32 {
        capped_ms(self.ms_flat * (1. + self.ms_percent))
    }

    /// Returns the average damage amplification for crit hits. i.e. if an attack that can crit does 100 dmg without crit,
    /// it will do on average 100 * `self.crit_formula()` when taking crits into account.
    #[inline]
    #[must_use]
    pub fn crit_coef(&self) -> f32 {
        1. + self.crit_chance * (self.crit_dmg - 1.)
    }

    fn add(&mut self, other_ref: &UnitStats) {
        self.hp += other_ref.hp;
        self.mana += other_ref.mana;
        self.base_ad += other_ref.base_ad;
        self.bonus_ad += other_ref.bonus_ad;
        self.ap_flat += other_ref.ap_flat;
        self.ap_coef += other_ref.ap_coef;
        self.armor += other_ref.armor;
        self.mr += other_ref.mr;
        self.base_as += other_ref.base_as;
        self.bonus_as += other_ref.bonus_as;
        self.ability_haste += other_ref.ability_haste;
        self.basic_haste += other_ref.basic_haste;
        self.ultimate_haste += other_ref.ultimate_haste;
        self.item_haste += other_ref.item_haste;
        self.crit_chance += other_ref.crit_chance;
        self.crit_dmg += other_ref.crit_dmg;
        self.ms_flat += other_ref.ms_flat;
        self.ms_percent += other_ref.ms_percent;
        self.lethality += other_ref.lethality;
        self.armor_pen_percent += other_ref.armor_pen_percent * (1. - self.armor_pen_percent); //stacks multiplicatively
        self.magic_pen_flat += other_ref.magic_pen_flat;
        self.magic_pen_percent += other_ref.magic_pen_percent * (1. - self.magic_pen_percent); //stacks multiplicatively
        self.armor_red_flat += other_ref.armor_red_flat;
        self.armor_red_percent += other_ref.armor_red_percent * (1. - self.armor_red_percent); //stacks multiplicatively
        self.mr_red_flat += other_ref.mr_red_flat;
        self.mr_red_percent += other_ref.mr_red_percent * (1. - self.mr_red_percent); //stacks multiplicatively
        self.life_steal += other_ref.life_steal;
        self.omnivamp += other_ref.omnivamp;

        self.crit_chance = f32::min(1., self.crit_chance); //cr capped at 100%
    }

    fn store_add(&mut self, ref_a: &UnitStats, ref_b: &UnitStats) {
        self.hp = ref_a.hp + ref_b.hp;
        self.mana = ref_a.mana + ref_b.mana;
        self.base_ad = ref_a.base_ad + ref_b.base_ad;
        self.bonus_ad = ref_a.bonus_ad + ref_b.bonus_ad;
        self.ap_flat = ref_a.ap_flat + ref_b.ap_flat;
        self.ap_coef = ref_a.ap_coef + ref_b.ap_coef;
        self.armor = ref_a.armor + ref_b.armor;
        self.mr = ref_a.mr + ref_b.mr;
        self.base_as = ref_a.base_as + ref_b.base_as;
        self.bonus_as = ref_a.bonus_as + ref_b.bonus_as;
        self.ability_haste = ref_a.ability_haste + ref_b.ability_haste;
        self.basic_haste = ref_a.basic_haste + ref_b.basic_haste;
        self.ultimate_haste = ref_a.ultimate_haste + ref_b.ultimate_haste;
        self.item_haste = ref_a.item_haste + ref_b.item_haste;
        self.crit_chance = f32::min(1., ref_a.crit_chance + ref_b.crit_chance); //cr capped at 100%
        self.crit_dmg = ref_a.crit_dmg + ref_b.crit_dmg;
        self.ms_flat = ref_a.ms_flat + ref_b.ms_flat;
        self.ms_percent = ref_a.ms_percent + ref_b.ms_percent;
        self.lethality = ref_a.lethality + ref_b.lethality;
        self.armor_pen_percent = ref_a.armor_pen_percent + ref_b.armor_pen_percent
            - ref_a.armor_pen_percent * ref_b.armor_pen_percent; //stacks multiplicatively
        self.magic_pen_flat = ref_a.magic_pen_flat + ref_b.magic_pen_flat;
        self.magic_pen_percent = ref_a.magic_pen_percent + ref_b.magic_pen_percent
            - ref_a.magic_pen_percent * ref_b.magic_pen_percent; //stacks multiplicatively
        self.armor_red_flat = ref_a.armor_red_flat + ref_b.armor_red_flat;
        self.armor_red_percent = ref_a.armor_red_percent + ref_b.armor_red_percent
            - ref_a.armor_red_percent * ref_b.armor_red_percent; //stacks multiplicatively
        self.mr_red_flat = ref_a.mr_red_flat + ref_b.mr_red_flat;
        self.mr_red_percent = ref_a.mr_red_percent + ref_b.mr_red_percent
            - ref_a.mr_red_percent * ref_b.mr_red_percent; //stacks multiplicatively
        self.life_steal = ref_a.life_steal + ref_b.life_steal;
        self.omnivamp = ref_a.omnivamp + ref_b.omnivamp;
    }

    fn put_to_zero(&mut self) {
        self.hp = 0.;
        self.mana = 0.;
        self.base_ad = 0.;
        self.bonus_ad = 0.;
        self.ap_flat = 0.;
        self.ap_coef = 0.;
        self.armor = 0.;
        self.mr = 0.;
        self.base_as = 0.;
        self.bonus_as = 0.;
        self.ability_haste = 0.;
        self.basic_haste = 0.;
        self.ultimate_haste = 0.;
        self.item_haste = 0.;
        self.crit_chance = 0.;
        self.crit_dmg = 0.;
        self.ms_flat = 0.;
        self.ms_percent = 0.;
        self.lethality = 0.;
        self.armor_pen_percent = 0.;
        self.magic_pen_flat = 0.;
        self.magic_pen_percent = 0.;
        self.armor_red_flat = 0.;
        self.armor_red_percent = 0.;
        self.mr_red_flat = 0.;
        self.mr_red_percent = 0.;
        self.life_steal = 0.;
        self.omnivamp = 0.;
    }
}

#[derive(Debug)]
pub struct BasicSpell {
    /// Returns spell dmg and triggers effects.
    /// Should call `Unit.dmg_on_target()` only for the return value at the end of the function !
    cast: fn(&mut Unit, &UnitStats) -> f32,
    cast_time: f32,
    base_cooldown_by_spell_lvl: [f32; 6], //length 6 to account aphelios case, normal spells only use the first 5 values
}

#[derive(Debug)]
pub struct UltimateSpell {
    /// Returns spell dmg and triggers effects.
    /// Should call `Unit.dmg_on_target()` only for the return value at the end of the function !
    cast: fn(&mut Unit, &UnitStats) -> f32,
    cast_time: f32,
    base_cooldown_by_spell_lvl: [f32; 3], //ultimate has 3 lvls
}

#[derive(Debug)]
pub struct UnitDefaults {
    pub runes_pages: &'static RunesPage,
    pub skill_order: &'static SkillOrder,
    pub legendary_items_pool: &'static [&'static Item],
    pub boots_pool: &'static [&'static Item],
    pub support_items_pool: &'static [&'static Item],
}

pub type FightScenario = (fn(&mut Unit, &UnitStats, f32), &'static str);

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
    /// Perform specific actions required when setting the Unit lvl (exemple: add veigar passive stacks ap to `lvl_stats`).
    pub on_lvl_set: Option<fn(&mut Unit)>,
    /// Init spells variables and starting buffs on the unit. This function should ensure that
    /// all Unit variables and all Unit buffs variables are properly initialized (in `Unit.buffs_values` or `Unit.buffs_stacks`).
    pub init_spells: Option<fn(&mut Unit)>,
    pub basic_attack: fn(&mut Unit, &UnitStats) -> f32, //returns basic attack dmg and triggers effects
    //no field for passive (incorporated in Unit spells instead)
    pub q: BasicSpell,
    pub w: BasicSpell,
    pub e: BasicSpell,
    pub r: UltimateSpell,
    pub fight_scenarios: &'static [FightScenario],
    pub unit_defaults: UnitDefaults,
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
        SkillOrder::const_default()
    }
}

impl SkillOrder {
    /// Returns classic skill order with q->w->e lvl-up priority.
    /// Provides a default valid value for `SkillOrder` usable in compile time constants (unlike `Default::default()` which is not const).
    #[must_use]
    pub const fn const_default() -> SkillOrder {
        SkillOrder {
            //lvls:
            //  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18
            q: [1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            w: [0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0],
            e: [0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1],
            r: [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0],
        }
    }
}

impl SkillOrder {
    /// Returns Ok if the given `skill_order` is valid, Err with an error message otherwise.
    /// A valid `skill_order` is one with 1 lvl-up per Unit lvl and in total 5 lvl-ups per spell (3 for ultimate).
    /// Aphelios special case is also treated when the 'aphelios' arg is set to true.
    pub fn check_skill_order_validity(&self, is_aphelios: bool) -> Result<(), String> {
        //u8 will never overflow since we enforce values in skill order to be only 0s or 1s (=> max sum we can encounter is `MAX_UNIT_LVL`)
        let mut q_sum: u8 = 0;
        let mut w_sum: u8 = 0;
        let mut e_sum: u8 = 0;
        let mut r_sum: u8 = 0;
        for i in 0..self.q.len() {
            //aphelios can lvl up a spell at lvl 6, 11, 16 in addition to his ult (beware i == lvl-1)
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
        let max_spell_lvl: u8 = if is_aphelios { 6 } else { 5 };
        if q_sum != max_spell_lvl || w_sum != max_spell_lvl || e_sum != max_spell_lvl || r_sum != 3
        {
            return Err("Wrong number of skill points distributed across spells".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct UnitSimulationResult {
    pub dmg_done: f32,
    pub life_vamped: f32,   //heals obtained by basic attacks over a duration
    pub heals_shields: f32, //"abrupt" heals and shields obtained once
    pub units_travelled: f32,
}

impl Default for UnitSimulationResult {
    fn default() -> Self {
        UnitSimulationResult {
            dmg_done: 0.,
            life_vamped: 0.,
            heals_shields: 0.,
            units_travelled: 0.,
        }
    }
}

impl UnitSimulationResult {
    fn reset(&mut self) {
        self.dmg_done = 0.;
        self.life_vamped = 0.;
        self.heals_shields = 0.;
        self.units_travelled = 0.;
    }
}

#[derive(Debug, Clone)]
pub struct Unit {
    //constant unit properties
    pub properties: &'static UnitProperties,
    //the rest below can change at runtime
    //stats
    pub stats: UnitStats,
    //lvl related
    skill_order: SkillOrder,
    pub lvl: NonZeroU8, //this is intentionally NonZeroU8 and not usize, so when used for indexing it reminds you to substract 1 to access array elements
    q_lvl: u8,
    w_lvl: u8,
    e_lvl: u8,
    r_lvl: u8,
    /// Stats that only comes from the Unit base stats (only change with lvl)
    pub lvl_stats: UnitStats,
    //build related
    pub build: Build,
    pub build_cost: f32,
    /// Stats that only comes from items
    pub items_stats: UnitStats,
    //runes related
    runes_page: RunesPage,
    runes_stats: UnitStats, //not pub on purpose because must not be used in items calculation
    //simulation timings
    pub time: f32,
    pub basic_attack_cd: f32,
    pub q_cd: f32,
    pub w_cd: f32,
    pub e_cd: f32,
    pub r_cd: f32,
    //permanent and temporary buffs
    pub buffs_stacks: EnumMap<BuffStackId, u8>, //holds various buffs integers values on the unit
    pub buffs_values: EnumMap<BuffValueId, f32>, //holds various buffs floats values on the unit
    temporary_buffs_durations: IndexMap<&'static TemporaryBuff, f32, FxBuildHasher>, //IndexMap of active temporary buffs on the unit and their remaining duration
    temporary_buffs_cooldowns: IndexMap<&'static TemporaryBuff, f32, FxBuildHasher>, //IndexMap of temporary buffs on cooldown on the unit
    //active fight scenario
    pub fight_scenario: FightScenario,
    //simulation results
    pub sim_results: UnitSimulationResult,
    pub actions_log: Vec<(f32, &'static str)>,
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
            "time: {:.1}, dmg done: {:.0}, life vamped : {:.0}, heals & shields: {:.0}, units tavelled: {:.0}",
            self.time, self.sim_results.dmg_done, self.sim_results.life_vamped, self.sim_results.heals_shields, self.sim_results.units_travelled
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
        writeln!(f, "omnivamp: {:.0}%", 100. * self.stats.omnivamp)
    }
}

/// Indicates if a dmg instance is from a basic or an ultimate spell.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DmgSource {
    BasicSpell,
    UltimateSpell,
    Other,
}

/// Used to return (`ad_dmg`, `ap_dmg`, `true_dmg`) of a damage instance.
pub type RawDmg = (f32, f32, f32);

impl Unit {
    /// base crit damage value for an Unit.
    pub const BASE_CRIT_DMG: f32 = 1.75;
    /// Default maximum attack speed limit for an Unit.
    pub const DEFAULT_AS_LIMIT: f32 = 2.5;

    /// Creates a new Unit with the given properties, runes, skill order, lvl and build.
    /// Return an Err with a corresponding error message if the Unit could not be created because of an invalid argument.
    pub fn new(
        properties_ref: &'static UnitProperties,
        runes_page: RunesPage,
        skill_order: SkillOrder,
        lvl: u8,
        build: Build,
    ) -> Result<Unit, String> {
        //perform some checks before creating the Unit
        //we don't want two different spells happening at the same time so cast time must be >= F32_TOL
        if properties_ref.q.cast_time < F32_TOL
            || properties_ref.w.cast_time < F32_TOL
            || properties_ref.e.cast_time < F32_TOL
            || properties_ref.r.cast_time < F32_TOL
        {
            return Err("Spells cast time should be >= F32_TOL".to_string());
        }

        //for similar reasons cooldowns must be >= F32_TOL
        if properties_ref
            .q
            .base_cooldown_by_spell_lvl
            .iter()
            .any(|cooldown| *cooldown < F32_TOL)
        {
            return Err("Q spell cooldown should be >= F32_TOL".to_string());
        }
        if properties_ref
            .w
            .base_cooldown_by_spell_lvl
            .iter()
            .any(|cooldown| *cooldown < F32_TOL)
        {
            return Err("W spell cooldown should be >= F32_TOL".to_string());
        }
        if properties_ref
            .e
            .base_cooldown_by_spell_lvl
            .iter()
            .any(|cooldown| *cooldown < F32_TOL)
        {
            return Err("E spell cooldown should be >= F32_TOL".to_string());
        }
        if properties_ref
            .r
            .base_cooldown_by_spell_lvl
            .iter()
            .any(|cooldown| *cooldown < F32_TOL)
        {
            return Err("R spell cooldown should be >= F32_TOL".to_string());
        }

        //create the unit
        let mut new_unit: Unit = Unit {
            properties: properties_ref,
            stats: UnitStats::default(),
            runes_page: RunesPage {
                shard1: RuneShard::Left,
                shard2: RuneShard::Left,
                shard3: RuneShard::Left,
            }, //trash temporary value until initialized by setter function
            runes_stats: UnitStats::default(),
            skill_order: SkillOrder::default(), //temporary value until initialized by setter function (must still be a valid skill order!)
            lvl: NonZeroU8::new(MIN_UNIT_LVL).expect("Unit lvl cannot be 0"), //trash temporary value until initialized by setter function
            q_lvl: 0, //trash temporary value until initialized by setter function
            w_lvl: 0, //trash temporary value until initialized by setter function
            e_lvl: 0, //trash temporary value until initialized by setter function
            r_lvl: 0, //trash temporary value until initialized by setter function
            lvl_stats: UnitStats::default(),
            build: Build::default(), //trash temporary value until initialized by setter function
            build_cost: 0.,
            items_stats: UnitStats::default(),
            time: 0.,
            basic_attack_cd: 0.,
            q_cd: 0.,
            w_cd: 0.,
            e_cd: 0.,
            r_cd: 0.,
            buffs_stacks: EnumMap::default(),
            buffs_values: EnumMap::default(),
            temporary_buffs_durations: IndexMap::with_hasher(FxBuildHasher),
            temporary_buffs_cooldowns: IndexMap::with_hasher(FxBuildHasher),
            fight_scenario: properties_ref.fight_scenarios[0],
            sim_results: UnitSimulationResult::default(),
            actions_log: Vec::new(),
        };

        //check and set runes
        new_unit.set_runes(runes_page)?;

        //check and set lvl
        new_unit.set_lvl(lvl)?;

        //check and set skill order
        new_unit.set_skill_order(skill_order)?;

        //check and set build
        new_unit.set_build(build)?;

        //init fight so new_unit is ready for simulation
        new_unit.init_fight();
        Ok(new_unit)
    }

    /// Creates a new Unit with the given properties, lvl and build.
    /// The default runes and skill order from the given properties are used.
    pub fn from_defaults(
        properties_ref: &'static UnitProperties,
        lvl: u8,
        build: Build,
    ) -> Result<Unit, String> {
        Unit::new(
            properties_ref,
            properties_ref.unit_defaults.runes_pages.clone(),
            properties_ref.unit_defaults.skill_order.clone(),
            lvl,
            build,
        )
    }

    /// Sets the Unit skill order, returns Ok if success or Err if failure (depending on the validity of the given skill order).
    /// In case of a failure, the unit is not modified.
    pub fn set_skill_order(&mut self, skill_order: SkillOrder) -> Result<(), String> {
        //return early if skill_order is not valid
        skill_order.check_skill_order_validity(*self.properties == Unit::APHELIOS_PROPERTIES)?;

        self.skill_order = skill_order;
        self.update_spells_lvls();

        Ok(())
    }

    /// Updates unit spells lvl.
    ///
    /// Because they depend on unit lvl, this function is called when setting lvl and skill order.
    /// This leads to redundant work when setting these in chain, but it's not a big deal.
    fn update_spells_lvls(&mut self) {
        let lvl: usize = usize::from(self.lvl.get());
        self.q_lvl = self.skill_order.q[..lvl].iter().sum();
        self.w_lvl = self.skill_order.w[..lvl].iter().sum();
        self.e_lvl = self.skill_order.e[..lvl].iter().sum();
        self.r_lvl = self.skill_order.r[..lvl].iter().sum();
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
        self.update_spells_lvls();

        //update unit lvl stats
        let base: &UnitStats = &self.properties.base_stats;
        let growth: &UnitStats = &self.properties.growth_stats;
        self.lvl_stats.hp = growth_stat_formula(self.lvl, base.hp, growth.hp);
        self.lvl_stats.mana = growth_stat_formula(self.lvl, base.mana, growth.mana);
        self.lvl_stats.base_ad = growth_stat_formula(self.lvl, base.base_ad, growth.base_ad);
        self.lvl_stats.bonus_ad = growth_stat_formula(self.lvl, base.bonus_ad, growth.bonus_ad);
        self.lvl_stats.ap_flat = growth_stat_formula(self.lvl, base.ap_flat, growth.ap_flat);
        self.lvl_stats.ap_coef = growth_stat_formula(self.lvl, base.ap_coef, growth.ap_coef);
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

        //perform specific actions required when setting the Unit lvl (exemple: add veigar passive stacks ap to lvl_stats)
        if let Some(on_lvl_set) = self.properties.on_lvl_set {
            on_lvl_set(self);
        }

        Ok(())
    }

    /// Updates the Unit build, returns Ok if success or Err if failure (depending on the validity of the given build).
    /// In case of a failure, the unit is not modified.
    pub fn set_build(&mut self, build: Build) -> Result<(), String> {
        //these checks are relatively expensive, if calling this function in hot code, consider using `Unit.set_build_unchecked()` instead
        build.check_validity()?;
        self.build = build;

        //update unit items stats
        self.items_stats.put_to_zero();
        self.build_cost = 0.;
        for &item_ref in build.iter().filter(|&&item_ref| *item_ref != NULL_ITEM) {
            self.items_stats.add(&item_ref.stats);
            self.build_cost += item_ref.cost;
        }

        Ok(())
    }

    /// Updates the Unit build regardless of its validity (saving some running time by discarding checks), always returns Ok.
    /// You must ensure that the given build is valid. Otherwise, this will lead to wrong results when simulating fights with the unit.
    pub fn set_build_unchecked(&mut self, build: Build) -> Result<(), String> {
        //no build validity check
        self.build = build;

        //update unit items stats
        self.items_stats.put_to_zero();
        self.build_cost = 0.;
        for &item_ref in build.iter().filter(|&&item_ref| *item_ref != NULL_ITEM) {
            self.items_stats.add(&item_ref.stats);
            self.build_cost += item_ref.cost;
        }

        Ok(())
    }

    pub fn init_fight(&mut self) {
        //simulation timings
        self.time = 0.;
        self.basic_attack_cd = 0.;
        self.q_cd = 0.;
        self.w_cd = 0.;
        self.e_cd = 0.;
        self.r_cd = 0.;

        //init stats
        self.stats.store_add(&self.lvl_stats, &self.items_stats);
        //runes are done later (need to do it after items passives init)

        //reset temporary buffs
        self.buffs_stacks.clear(); //this is not really needed since we init the variables later, but we do it to clear unused variables for debugging convenience
        self.buffs_values.clear(); //same as above
        self.temporary_buffs_durations.clear(); //this is needed to remove every temporary buffs
        self.temporary_buffs_cooldowns.clear(); //same as above

        //self.temporary_buffs_durations.shrink_to_fit(); //hits performance a bit, i think the reduced memory usage is not worth it
        //self.temporary_buffs_cooldowns.shrink_to_fit(); //hits performance a bit, i think the reduced memory usage is not worth it

        //init items variables and starting buffs on the unit (after buffs reset)
        //we iterate over the index because we can't borrow mut self twice (since we pass a mutable reference to the item function)
        //this is hacky but the init function should never change self.build
        for i in 0..MAX_UNIT_ITEMS {
            if let Some(init_item) = self.build[i].init_item {
                init_item(self);
            }
        }

        //runes stats (after items passives init)
        self.update_runes_stats();
        self.stats.add(&self.runes_stats);

        //init spells variables and starting buffs on the unit
        if let Some(init_spells) = self.properties.init_spells {
            init_spells(self);
        }

        //reset simulation results
        self.sim_results.reset();

        //reset actions log
        self.actions_log.clear();
        //self.actions_log.shrink_to_fit(); //hits performance a bit, i think the reduced memory usage is not worth it
    }

    /// Attempt to add the given buff to the Unit. If the buff is not on cooldown, the function
    /// adds it to the Unit and returns true (or refreshes its duration if already present).
    /// If the buff is on cooldown, it does nothing and returns false.
    ///
    /// The haste argument is to specify which haste value to use for the buff cooldown (ability haste, item haste, ...)
    pub fn add_temporary_buff(&mut self, buff_ref: &'static TemporaryBuff, haste: f32) -> bool {
        //return early if buff is on cooldown
        if self.temporary_buffs_cooldowns.contains_key(buff_ref) {
            return false;
        }

        //store buff remaining duration
        self.temporary_buffs_durations
            .insert(buff_ref, buff_ref.duration);

        //store buff cooldown only if cooldown is non-zero (cooldown starts on activation)
        let mut availability_coef: f32 = 1.;
        if buff_ref.cooldown != 0. {
            let real_cooldown: f32 = buff_ref.cooldown * haste_formula(haste);
            availability_coef = effect_availability_formula(real_cooldown);
            self.temporary_buffs_cooldowns
                .insert(buff_ref, real_cooldown);
        }

        //add buff stack to the unit
        (buff_ref.add_stack)(self, availability_coef);
        true
    }

    /// Wait immobile for the given amount of time. Removes buffs if they expire.
    pub fn wait(&mut self, dt: f32) {
        //sanity check
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

        //update buffs durations
        let mut i: usize = 0;
        while i < self.temporary_buffs_durations.len() {
            //update buff duration
            let (&buff_ref, duration_ref) =
                self.temporary_buffs_durations.get_index_mut(i).unwrap();
            *duration_ref -= dt;

            //remove buff from the unit if its duration ends
            if *duration_ref < F32_TOL {
                (buff_ref.remove_every_stack)(self);
                self.temporary_buffs_durations.swap_remove_index(i);
            } else {
                i += 1;
            }
        }

        //update buffs cooldowns
        let mut i: usize = 0;
        while i < self.temporary_buffs_cooldowns.len() {
            //update buff cooldown
            let (_, cooldown_ref) = self.temporary_buffs_cooldowns.get_index_mut(i).unwrap();
            *cooldown_ref -= dt;

            //remove buff from storage if its cooldown ends
            if *cooldown_ref < F32_TOL {
                self.temporary_buffs_cooldowns.swap_remove_index(i);
            } else {
                i += 1;
            }
        }
    }

    /// Move for the given amount of time. Removes buffs if they expire and add distance to `self.simulation_results.units_travelled`.
    pub fn walk(&mut self, mut dt: f32) {
        while dt >= F32_TOL {
            //find minimum time until next expiring buff
            let min_duration: f32 = *self
                .temporary_buffs_durations
                .values()
                .chain(std::iter::once(&dt))
                .min_by(|a, b| a.partial_cmp(b).expect("Failed to compare floats"))
                .unwrap(); //will never panic as we chain with once so there is at minimum 1 element

            //walk until next expiring buff
            self.sim_results.units_travelled += self.stats.ms() * min_duration; //must be before self.wait() to still benefit from temporary buffs
            self.wait(min_duration);
            dt -= min_duration;
        }
    }

    /// Returns items on basic attack hit static dmg.
    pub fn get_items_on_basic_attack_hit_static(&mut self, target_stats: &UnitStats) -> RawDmg {
        let mut ad_dmg: f32 = 0.;
        let mut ap_dmg: f32 = 0.;
        let mut true_dmg: f32 = 0.;
        //we iterate over the index because we can't borrow mut self twice (since we pass a mutable reference to the item function)
        //this is hacky but the init function should never change self.build
        for i in 0..MAX_UNIT_ITEMS {
            if let Some(on_basic_attack_hit_static) = self.build[i].on_basic_attack_hit_static {
                let (
                    on_basic_attack_hit_static_ad_dmg,
                    on_basic_attack_hit_static_ap_dmg,
                    on_basic_attack_hit_static_true_dmg,
                ) = (on_basic_attack_hit_static)(self, target_stats);
                ad_dmg += on_basic_attack_hit_static_ad_dmg;
                ap_dmg += on_basic_attack_hit_static_ap_dmg;
                true_dmg += on_basic_attack_hit_static_true_dmg;
            }
        }
        (ad_dmg, ap_dmg, true_dmg)
    }

    /// Returns items spell coef.
    fn get_items_spell_coef(&mut self) -> f32 {
        let mut coef: f32 = 1.;
        //we iterate over the index because we can't borrow mut self twice (since we pass a mutable reference to the item function)
        //this is hacky but the init function should never change self.build
        for i in 0..MAX_UNIT_ITEMS {
            if let Some(spell_coef) = self.build[i].spell_coef {
                coef += (spell_coef)(self); //stacks additively
            }
        }
        coef
    }

    /// From raw dmg (separated ad, ap & true dmg values without taking resistances into account),
    /// returns the actual dmg taken by the target. Also stacks Unit/items passive effects.
    ///
    /// Since this is a relatively expensive function to run, try to call it as little as possible and use
    /// `n_targets`, `n_dmg_instances`, `n_stacking_instances` arguments to regroup multiple sources of dmg
    /// that happens at the same time or are from the same spell, ...
    ///
    /// ARGUMENTS :
    ///
    /// - self: attacking Unit.
    ///
    /// - `target_stats`: target stats used for the dmg calculations.
    ///
    /// - (`ad_dmg`, `ap_dmg`, `true_dmg)`: raw dmg used to calculate final dmg on the target.
    ///
    /// - (`n_dmg_instances`, `n_stacking_instances)`:
    ///     - `n_dmg_instances`: number of dmg instances on one of the targets considered for items on ad/ap dmg effects
    ///       (e.g. black cleaver, affects items on ad/ap dmg effects ONLY, doesn't concern items on basic attack/spell hit effects).
    ///     - `n_stacking_instances`: number of stacking instances on a single target considered for items on basic attack/spell hit effects stacking.
    ///       Must be less or equal to `n_dmg_instances`.
    ///       (affects items on basic attack/spell hit effects ONLY, doesn't concern items on ad/ap dmg effects).
    ///       `n_dmg_instances` and `n_stacking_instances` are needed sperately, exemple:
    ///       Ashe q arrows stack black cleaver fully (`on_ad_hit`), but gives only 1 stack of kraken slayer (`on_basic_attack_hit`).
    ///       /!\ `n_dmg_instances` must always be greater than `n_stacking_instances` (else it is a logic error).
    ///
    /// - `dmg_source`: source of the instance of dmg
    ///   (if `DmgSource::BasicSpell`, triggers items on basic spell hit and spell coef,
    ///   if `DmgSource::UtlimateSpell`, triggers items on ultimate spell hit and spell coef).
    ///
    /// - `triggers_on_basic_attack_effects`: if the attack triggers items on basic attack hit effects.
    ///
    /// - `n_targets`: number of targets hit, affects items on basic attack/spell hit effects ONLY
    ///   (raw dmg received by this function must already be the sum on all targets).
    pub fn dmg_on_target(
        &mut self,
        target_stats: &UnitStats,
        (mut ad_dmg, mut ap_dmg, mut true_dmg): RawDmg,
        (n_dmg_instances, n_stacking_instances): (u8, u8),
        dmg_source: DmgSource,
        triggers_on_basic_attack_effects: bool,
        n_targets: f32,
    ) -> f32 {
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

        //on spell hit and spell coef, must be done before on basic attack hit because of muramana shock that applies spell part first
        match dmg_source {
            DmgSource::BasicSpell => {
                //on basic spell hit
                //we iterate over the index because we can't borrow mut self twice (since we pass a mutable reference to the item function)
                //this is hacky but the init function should never change self.build
                for i in 0..MAX_UNIT_ITEMS {
                    if let Some(on_basic_spell_hit) = self.build[i].on_basic_spell_hit {
                        for _ in 0..n_stacking_instances {
                            let (
                                on_basic_spell_hit_ad_dmg,
                                on_basic_spell_hit_ap_dmg,
                                on_basic_spell_hit_true_dmg,
                            ) = (on_basic_spell_hit)(self, target_stats, n_targets);
                            ad_dmg += on_basic_spell_hit_ad_dmg;
                            ap_dmg += on_basic_spell_hit_ap_dmg;
                            true_dmg += on_basic_spell_hit_true_dmg;
                        }
                    }
                }
                //spell coef (also affects on_spell_hit dmg)
                let coef: f32 = self.get_items_spell_coef();
                ad_dmg *= coef;
                ap_dmg *= coef;
                true_dmg *= coef;
            }
            DmgSource::UltimateSpell => {
                //on ultimate spell hit
                for i in 0..MAX_UNIT_ITEMS {
                    if let Some(on_ultimate_spell_hit) = self.build[i].on_ultimate_spell_hit {
                        for _ in 0..n_stacking_instances {
                            let (
                                on_ultimate_spell_hit_ad_dmg,
                                on_ultimate_spell_hit_ap_dmg,
                                on_ultimate_spell_hit_true_dmg,
                            ) = (on_ultimate_spell_hit)(self, target_stats, n_targets);
                            ad_dmg += on_ultimate_spell_hit_ad_dmg;
                            ap_dmg += on_ultimate_spell_hit_ap_dmg;
                            true_dmg += on_ultimate_spell_hit_true_dmg;
                        }
                    }
                }
                //spell coef (also affects on_spell_hit dmg)
                let coef: f32 = self.get_items_spell_coef();
                ad_dmg *= coef;
                ap_dmg *= coef;
                true_dmg *= coef;
            }
            DmgSource::Other => (),
        }

        //on basic attack hit, divided in two parts:
        // - static: dmg that applies on all targets unconditionally
        // - dynamic: dmg that applies only on the first target hit conditionnally (like energized passives, ...)
        if triggers_on_basic_attack_effects {
            for i in 0..MAX_UNIT_ITEMS {
                //on basic attack hit static
                if let Some(on_basic_attack_hit_static) = self.build[i].on_basic_attack_hit_static {
                    for _ in 0..n_stacking_instances {
                        let (
                            on_basic_attack_hit_static_ad_dmg,
                            on_basic_attack_hit_static_ap_dmg,
                            on_basic_attack_hit_static_true_dmg,
                        ) = (on_basic_attack_hit_static)(self, target_stats);
                        ad_dmg += n_targets * on_basic_attack_hit_static_ad_dmg;
                        ap_dmg += n_targets * on_basic_attack_hit_static_ap_dmg;
                        true_dmg += n_targets * on_basic_attack_hit_static_true_dmg;
                    }
                }
                //on basic attack hit dynamic
                if let Some(on_basic_attack_hit_dynamic) = self.build[i].on_basic_attack_hit_dynamic
                {
                    for _ in 0..n_stacking_instances {
                        let (
                            on_basic_attack_hit_dynamic_ad_dmg,
                            on_basic_attack_hit_dynamic_ap_dmg,
                            on_basic_attack_hit_dynamic_true_dmg,
                        ) = (on_basic_attack_hit_dynamic)(self, target_stats);
                        ad_dmg += on_basic_attack_hit_dynamic_ad_dmg;
                        ap_dmg += on_basic_attack_hit_dynamic_ap_dmg;
                        true_dmg += on_basic_attack_hit_dynamic_true_dmg;
                    }
                }
            }
        }

        //on any hit
        for i in 0..MAX_UNIT_ITEMS {
            if let Some(on_any_hit) = self.build[i].on_any_hit {
                for _ in 0..n_stacking_instances {
                    let (on_any_hit_ad_dmg, on_any_hit_ap_dmg, on_any_hit_true_dmg) =
                        (on_any_hit)(self, target_stats);
                    ad_dmg += on_any_hit_ad_dmg;
                    ap_dmg += on_any_hit_ap_dmg;
                    true_dmg += on_any_hit_true_dmg;
                }
            }
        }

        //on ad dmg
        if ad_dmg > 0. {
            for i in 0..MAX_UNIT_ITEMS {
                if let Some(on_ad_hit) = self.build[i].on_ad_hit {
                    for _ in 0..n_dmg_instances {
                        (on_ad_hit)(self);
                    }
                }
            }
        }

        //ap and true dmg coef
        if ap_dmg > 0. || true_dmg > 0. {
            let mut coef: f32 = 1.;
            for i in 0..MAX_UNIT_ITEMS {
                if let Some(ap_true_dmg_coef) = self.build[i].ap_true_dmg_coef {
                    coef += (ap_true_dmg_coef)(self);
                }
            }
            ap_dmg *= coef;
            true_dmg *= coef;
        }

        //total dmg coef
        let mut tot_coef: f32 = 1.;
        for i in 0..MAX_UNIT_ITEMS {
            if let Some(tot_dmg_coef) = self.build[i].tot_dmg_coef {
                tot_coef += (tot_dmg_coef)(self, target_stats); //stacks additively
            }
        }
        let tot_dmg: f32 = (ad_dmg * armor_coef + ap_dmg * mr_coef + true_dmg) * tot_coef;

        //lifesteal and omnivamp
        self.sim_results.life_vamped += tot_dmg * self.stats.omnivamp; //omnivamp
        if triggers_on_basic_attack_effects {
            self.sim_results.life_vamped += tot_dmg * self.stats.life_steal;
            //lifesteal
        }

        self.time += F32_TOL; //to differentiate different dmg instances
        self.sim_results.dmg_done += tot_dmg;
        tot_dmg
    }

    /// Triggers every item actives on the unit and returns dmg done.
    pub fn use_all_items_actives(&mut self, target_stats: &UnitStats) -> f32 {
        //save log
        self.actions_log.push((self.time, "all items actives"));

        //we iterate over the index because we can't borrow mut self twice (since we pass a mutable reference to the item function)
        //this is hacky but the init function should never change self.build
        let mut dmg: f32 = 0.;
        for i in 0..MAX_UNIT_ITEMS {
            if let Some(active) = self.build[i].active {
                dmg += active(self, target_stats);
                self.wait(F32_TOL); //so effects that relies on time to differentiate instances of dmg work properly
            }
        }
        dmg
    }

    /// Performs a basic attack and returns dmg done.
    pub fn basic_attack(&mut self, target_stats: &UnitStats) -> f32 {
        //save log
        self.actions_log.push((self.time, "basic attack"));

        //wait cast (windup) time
        let windup_time: f32 = real_windup_time(windup_formula(
            self.properties.windup_percent,
            self.properties.windup_modifier,
            self.stats.base_as,
            self.stats.attack_speed(self.properties.as_ratio),
        ));
        self.wait(windup_time);

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
    pub fn q(&mut self, target_stats: &UnitStats) -> f32 {
        //save log
        self.actions_log.push((self.time, "Q"));

        //wait cast time
        self.wait(self.properties.q.cast_time);

        //on spell cast effects
        //we iterate over the index because we can't borrow mut self twice (since we pass a mutable reference to the item function)
        //this is hacky but the init function should never change self.build
        for i in 0..MAX_UNIT_ITEMS {
            if let Some(on_basic_spell_cast) = self.build[i].on_basic_spell_cast {
                (on_basic_spell_cast)(self);
            }
        }
        //set cd
        self.q_cd = haste_formula(self.stats.ability_haste_basic())
            * self.properties.q.base_cooldown_by_spell_lvl[usize::from(self.q_lvl - 1)];

        //return dmg
        (self.properties.q.cast)(self, target_stats)
    }

    /// cast w and returns dmg done.
    pub fn w(&mut self, target_stats: &UnitStats) -> f32 {
        //save log
        self.actions_log.push((self.time, "W"));

        //wait cast time
        self.wait(self.properties.w.cast_time);

        //on spell cast effects
        //we iterate over the index because we can't borrow mut self twice (since we pass a mutable reference to the item function)
        //this is hacky but the init function should never change self.build
        for i in 0..MAX_UNIT_ITEMS {
            if let Some(on_basic_spell_cast) = self.build[i].on_basic_spell_cast {
                (on_basic_spell_cast)(self);
            }
        }
        //set cd
        self.w_cd = haste_formula(self.stats.ability_haste_basic())
            * self.properties.w.base_cooldown_by_spell_lvl[usize::from(self.w_lvl - 1)];

        //return dmg
        (self.properties.w.cast)(self, target_stats)
    }

    /// cast e and returns dmg done.
    pub fn e(&mut self, target_stats: &UnitStats) -> f32 {
        //save log
        self.actions_log.push((self.time, "E"));

        //wait cast time
        self.wait(self.properties.e.cast_time);

        //on spell cast effects
        //we iterate over the index because we can't borrow mut self twice (since we pass a mutable reference to the item function)
        //this is hacky but the init function should never change self.build
        for i in 0..MAX_UNIT_ITEMS {
            if let Some(on_basic_spell_cast) = self.build[i].on_basic_spell_cast {
                (on_basic_spell_cast)(self);
            }
        }
        //set cd
        self.e_cd = haste_formula(self.stats.ability_haste_basic())
            * self.properties.e.base_cooldown_by_spell_lvl[usize::from(self.e_lvl - 1)];

        //return dmg
        (self.properties.e.cast)(self, target_stats)
    }

    /// cast r and returns dmg done.
    pub fn r(&mut self, target_stats: &UnitStats) -> f32 {
        //save log
        self.actions_log.push((self.time, "R"));

        //wait cast time
        self.wait(self.properties.r.cast_time);

        //on spell cast effects
        //we iterate over the index because we can't borrow mut self twice (since we pass a mutable reference to the item function)
        //this is hacky but the init function should never change self.build
        for i in 0..MAX_UNIT_ITEMS {
            if let Some(on_ultimate_cast) = self.build[i].on_ultimate_cast {
                (on_ultimate_cast)(self);
            }
        }
        //set cd
        self.r_cd = haste_formula(self.stats.ability_haste_ultimate())
            * self.properties.r.base_cooldown_by_spell_lvl[usize::from(self.r_lvl - 1)];

        //return dmg
        (self.properties.r.cast)(self, target_stats)
    }

    /// Same as casting r except the dmg, units travelled, etc. during the r are all reduced
    /// according to the availability formula (to account for the r cooldown).
    /// Useless to use for ultimates that only adds a buff.
    pub fn weighted_r(&mut self, target_stats: &UnitStats) -> f32 {
        let dmg_done_before_r: f32 = self.sim_results.dmg_done;
        let life_vamped_before_r: f32 = self.sim_results.life_vamped;
        let heals_shields_before_r: f32 = self.sim_results.heals_shields;
        let units_travelled_before_r: f32 = self.sim_results.units_travelled;
        self.r(target_stats);
        let percent_to_remove: f32 = 1. - effect_availability_formula(self.r_cd);

        let tot_dmg: f32 = self.sim_results.dmg_done - dmg_done_before_r;
        self.sim_results.dmg_done -= percent_to_remove * tot_dmg;
        self.sim_results.life_vamped -=
            percent_to_remove * (self.sim_results.life_vamped - life_vamped_before_r);
        self.sim_results.heals_shields -=
            percent_to_remove * (self.sim_results.heals_shields - heals_shields_before_r);
        self.sim_results.units_travelled -=
            percent_to_remove * (self.sim_results.units_travelled - units_travelled_before_r);

        (1. - percent_to_remove) * tot_dmg //return weighted dmg
    }

    /// Simulate a fight for the unit hitting the specified target according to what is defined in the unit properties
    /// and returns (average dps, effective hp, average move speed) obtained from the simulation.
    /// This function will always start by initializing the unit with `self.init_fight` and use all items actives before simulating.
    pub fn simulate_fight(
        &mut self,
        target_stats: &UnitStats,
        fight_duration: f32,
        ad_taken_percent: f32,
    ) -> (f32, f32, f32) {
        self.init_fight();
        self.use_all_items_actives(target_stats);
        (self.fight_scenario.0)(self, target_stats, fight_duration);

        //retrieve scores
        let actual_time: f32 = self.time; //take self.time instead of fight_duration in scores calculations, since simulation can be slighlty extended
        let dps: f32 = self.sim_results.dmg_done / actual_time; //average dps of the unit over the fight simulation
        let effective_hp: f32 = (self.stats.hp
            + self.sim_results.heals_shields
            + 6. * self.sim_results.life_vamped / actual_time)
            / (ad_taken_percent * resistance_formula_pos(self.stats.armor)
                + (1. - ad_taken_percent) * resistance_formula_pos(self.stats.mr)); //effective (hp + heals and shields gained during the simulation + life vamped over a standardized duration), ap dmg percent is deducted by the formula: 1. - ad_taken_percent (neglecting true damage)
        let move_speed: f32 = self.sim_results.units_travelled / actual_time; //average move speed of the unit over the fight simulation
        (dps, effective_hp, move_speed)
    }

    /// Default `basic_attack` for an unit.
    #[inline]
    pub fn default_basic_attack(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
        let ad_dmg: f32 = champ.stats.ad() * champ.stats.crit_coef();
        champ.dmg_on_target(
            target_stats,
            (ad_dmg, 0., 0.),
            (1, 1),
            DmgSource::Other,
            true,
            1.,
        )
    }
}

/// For performance reasons, we use a `null_basic_attack` function (that should never be called and will panic if so) instead of an Option, for units that do not have one.
///
/// This is to avoid checking an Option everytime a `basic_attack` is called, since the majority of basic attacks aren't null
/// and the user should know in advance if said unit `basic_attack` is null or not.
pub fn null_basic_attack(_champ: &mut Unit, _target_stats: &UnitStats) -> f32 {
    unreachable!("Null_basic_attack was called");
}

/// For performance reasons, we use a `NULL_BASIC_SPELL` constant (that should never be used) instead of an Option, for units that do not have one.
///
/// This is to avoid checking an Option everytime a spell is called, since the majority of spells aren't null
/// and the user should know in advance if said unit spell is null or not.
pub const NULL_BASIC_SPELL: BasicSpell = BasicSpell {
    cast: null_spell_cast,
    cast_time: F32_TOL,
    base_cooldown_by_spell_lvl: [F32_TOL, F32_TOL, F32_TOL, F32_TOL, F32_TOL, F32_TOL],
};

/// For performance reasons, we use a `NULL_ULTIMATE_SPELL` constant (that should never be used) instead of an Option, for units that do not have one.
///
/// This is to avoid checking an Option everytime a spell is called, since the majority of spells aren't null
/// and the user should know in advance if said unit spell is null or not.
pub const NULL_ULTIMATE_SPELL: UltimateSpell = UltimateSpell {
    cast: null_spell_cast,
    cast_time: F32_TOL,
    base_cooldown_by_spell_lvl: [F32_TOL, F32_TOL, F32_TOL],
};

fn null_spell_cast(_champ: &mut Unit, _target_stats: &UnitStats) -> f32 {
    unreachable!("Null_spell_cast was called");
}

/// For performance reasons, we use a `null_simulate_fight` function (that should never be called and will panic if so) instead of an Option, for units that do not have one.
///
/// This is to avoid checking an Option everytime a `simulate_fight` is called, since the majority of `simulate_fight` aren't null
/// and the user should know in advance if said unit `simulate_fight` is null or not.
pub fn null_simulate_fight(_champ: &mut Unit, _target_stats: &UnitStats, _time_limit: f32) {
    unreachable!("Null_simulate_fight was called");
}
