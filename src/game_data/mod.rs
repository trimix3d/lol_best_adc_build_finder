pub mod effects_data;
pub mod items_data;
pub mod units_data;

use items_data::{AVG_BOOTS_COST, AVG_LEGENDARY_ITEM_COST, AVG_SUPPORT_ITEM_COST};
use units_data::{MAX_UNIT_LVL, MIN_UNIT_LVL};

//patch number
pub const PATCH_NUMBER_MAJOR: &str = env!("CARGO_PKG_VERSION_MAJOR");
pub const PATCH_NUMBER_MINOR: &str = env!("CARGO_PKG_VERSION_MINOR");

/// Tolerance for float values equality (most of the time related to timings).
/// Must be some orders of magnitude larger than the machine epsilon (for proper functionning of the logic of the program),
/// but also small enough to be an acceptable time interval error in seconds between two actions (for accurate simulation).
const F32_TOL: f32 = 1e-4;

//game parameters (constants):
/// Minimum time in seconds between two mouse click. Used to calculate windup time (time spent casting a basic attack).
const TIME_BETWEEN_CLICKS: f32 = 0.15;
/// Amount of gold that one point of hp is worth.
pub const HP_GOLD_VALUE: f32 = 2.67;
/// Amount of gold that one point of armor is worth.
pub const ARMOR_GOLD_VALUE: f32 = 20.;
/// Amount of gold that one point of mr is worth.
pub const MR_GOLD_VALUE: f32 = 20.;
/// Starting golds on summoners rift.
pub const STARTING_GOLDS: f32 = 500.;
/// Passive gold generation per minute on summoners rift.
pub const PASSIVE_GOLDS_GEN_PER_MIN: f32 = 122.4;
/// CS/min of the player of the champion we want to optimize,
/// a bit inflated (~+20%) to take other sources of golds into account (kills, towers, ...).
pub const CS_PER_MIN: f32 = 8.0;
const GOLDS_PER_MELEE_CS: f32 = 21.;
const GOLDS_PER_CASTER_CS: f32 = 14.;
/// Average gold per siege minion over a 30min game.
const AVG_GOLDS_PER_SIEGE_CS: f32 = 17.25 / 30. * 75. + 12.75 / 30. * 90.;
/// Average gold per minion over a 30min game.
/// - 1 siege minion per 3 wave before 15min
/// - 1 siege minion per 2 wave between 15 and 25min.
/// - 1 siege minion per wave after 25min.
pub const AVG_GOLDS_PER_CS: f32 = 15. / 30.
    * (3. * GOLDS_PER_MELEE_CS + 3. * GOLDS_PER_CASTER_CS + 1. / 3. * AVG_GOLDS_PER_SIEGE_CS)
    / (6. + 1. / 3.)
    + 10. / 30.
        * (3. * GOLDS_PER_MELEE_CS + 3. * GOLDS_PER_CASTER_CS + 1. / 2. * AVG_GOLDS_PER_SIEGE_CS)
        / (6. + 1. / 2.)
    + 5. / 30. * (3. * GOLDS_PER_MELEE_CS + 3. * GOLDS_PER_CASTER_CS + 1. * AVG_GOLDS_PER_SIEGE_CS)
        / 7.;
/// Total amount of golds income per minute considering farm + passive generation
const TOT_GOLDS_PER_MIN: f32 = AVG_GOLDS_PER_CS * CS_PER_MIN + PASSIVE_GOLDS_GEN_PER_MIN;
const XP_PER_MELEE_CS: f32 = 61.75;
const XP_PER_CASTER_CS: f32 = 30.4;
const XP_PER_SIEGE_CS: f32 = 95.;
/// Average xp per minion over a 30min game.
/// - 1 siege minion per 3 wave before 15min
/// - 1 siege minion per 2 wave between 15 and 25min.
/// - 1 siege minion per wave after 25min.
pub const AVG_XP_PER_CS: f32 = 15. / 30.
    * (3. * XP_PER_MELEE_CS + 3. * XP_PER_CASTER_CS + 1. / 3. * XP_PER_SIEGE_CS)
    / (6. + 1. / 3.)
    + 10. / 30. * (3. * XP_PER_MELEE_CS + 3. * XP_PER_CASTER_CS + 1. / 2. * XP_PER_SIEGE_CS)
        / (6. + 1. / 2.)
    + 5. / 30. * (3. * XP_PER_MELEE_CS + 3. * XP_PER_CASTER_CS + 1. * XP_PER_SIEGE_CS) / 7.;
/// Amount of experience gained farming for the average legendary item.
/// We approximate that the gold income is only from cs golds and passive golds generation.
const XP_PER_LEGENDARY_ITEM: f32 =
    AVG_XP_PER_CS * CS_PER_MIN * AVG_LEGENDARY_ITEM_COST / TOT_GOLDS_PER_MIN;
/// Amount of experience gained farming for the average boots item.
/// We approximate that the gold income is only from cs golds and passive golds generation.
const XP_PER_BOOTS_ITEM: f32 = AVG_XP_PER_CS * CS_PER_MIN * AVG_BOOTS_COST / TOT_GOLDS_PER_MIN;
/// Amount of experience gained farming for the average support item.
/// We approximate that the gold income is only from cs golds and passive golds generation.
const XP_PER_SUPPORT_ITEM: f32 =
    AVG_XP_PER_CS * CS_PER_MIN * AVG_SUPPORT_ITEM_COST / TOT_GOLDS_PER_MIN;

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
/// Travel distance required to fully charge energized attacks (rapid firecanon, statikk shiv, ...).
const ENERGIZED_ATTACKS_TRAVEL_REQUIRED: f32 = 100. * 24.;

//fights simulation parameters:
/// Average time in seconds we consider between fights (used to weight items actives with different cooldowns).
const TIME_BETWEEN_FIGHTS: f32 = 180.;
/// Returns the availability coef of a passive/active effect according to its cooldown.
/// It should be used on effects that have cooldowns way longer than the fight simulation.
/// The function receives the real cooldown of the effect, already reduced by haste.
fn effect_availability_formula(real_cooldown: f32) -> f32 {
    TIME_BETWEEN_FIGHTS / (TIME_BETWEEN_FIGHTS + real_cooldown)
}
/// Reference area used to compute the average number of targets hit by basic attacks aoe effects.
/// Should have a value so that an aoe basic attack effect with this range hits on average the same number of targets than runaans bolts.
const AOE_BASIC_ATTACK_REFERENCE_RADIUS: f32 = 450.;

/// From the radius of the aoe basic attack effect, gives the number of targets hit
/// (additionnal to the target that was originally hit by the basic attack).
macro_rules! basic_attack_aoe_effect_avg_additionnal_targets {
    ($radius:expr) => {
        crate::game_data::items_data::items::RUNAANS_HURRICANE_WINDS_FURY_AVG_BOLTS
            * $radius
            * $radius
            / (crate::game_data::AOE_BASIC_ATTACK_REFERENCE_RADIUS
                * crate::game_data::AOE_BASIC_ATTACK_REFERENCE_RADIUS)
    };
}
use basic_attack_aoe_effect_avg_additionnal_targets; //to make it accessible in submodules

//game mechanics related functions:
/// Returns coefficient multiplying base cooldown to give the actual cooldown reduced by haste.
/// <https://leagueoflegends.fandom.com/wiki/Haste>
fn haste_formula(haste: f32) -> f32 {
    100. / (100. + haste)
}

/// From number of items, returns the associated unit lvl.
#[must_use]
pub fn lvl_from_number_of_items(
    item_slot: usize,
    boots_slot: usize,
    support_item_slot: usize,
) -> u8 {
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
