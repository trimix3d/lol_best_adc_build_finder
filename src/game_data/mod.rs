pub mod buffs_data;
pub mod items_data;
pub mod units_data;

//patch number
pub const PATCH_NUMBER_MAJOR: &str = env!("CARGO_PKG_VERSION_MAJOR");
pub const PATCH_NUMBER_MINOR: &str = env!("CARGO_PKG_VERSION_MINOR");

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

//game parameters (constants):
const EPS: f32 = 1e-4;
/// Tolerance for float values equality (most of the time related to timings).
/// Must be some orders of magnitude larger than the machine epsilon (for proper functionning of the logic of the program),
/// but also small enough to be an acceptable time interval error in seconds between two actions (for accurate simulation).
const F32_TOL: f32 = if EPS > f32::EPSILON {
    EPS
} else {
    f32::EPSILON
};
/// Minimum time in seconds between two mouse click. Used to calculate windup time (time spent casting a basic attack).
const TIME_BETWEEN_CLICKS: f32 = 0.15;
/// Amount of gold that one point of hp is worth.
pub const HP_GOLD_VALUE: f32 = 2.67;
/// Amount of gold that one point of armor is worth.
pub const ARMOR_GOLD_VALUE: f32 = 20.;
/// Amount of gold that one point of mr is worth.
pub const MR_GOLD_VALUE: f32 = 18.;
/// Starting golds on summoners rift.
pub const STARTING_GOLDS: f32 = 500.;
/// Passive gold generation per minute on summoners rift.
pub const PASSIVE_GOLDS_GEN_PER_MIN: f32 = 122.4;
/// CS/min of the player of the champion we want to optimize,
/// a bit inflated to take other sources of golds into account (kills, towers, ...).
pub const CS_PER_MIN: f32 = 8.;
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
/// Travel distance required to fully charge energized attacks (rapid firecanon, statikk shiv, ...).
const ENERGIZED_ATTACKS_TRAVEL_REQUIRED: f32 = 100. * 24.;

//other parameters:
/// Average time in seconds we consider between fights (used to weight items actives with different cooldowns).
const TIME_BETWEEN_FIGHTS: f32 = 180.;
/// Returns the availability coef of a passive/active effect according to its cooldown.
/// It should be used on effects that have cooldowns way longer than the fight simulation.
/// The function receives the real cooldown of the effect, already reduced by haste.
fn effect_availability_formula(real_cooldown: f32) -> f32 {
    TIME_BETWEEN_FIGHTS / (TIME_BETWEEN_FIGHTS + real_cooldown)
}

//game related functions:
/// Returns coefficient multiplying base cooldown to give the actual cooldown reduced by haste.
/// <https://leagueoflegends.fandom.com/wiki/Haste>
fn haste_formula(haste: f32) -> f32 {
    100. / (100. + haste)
}
