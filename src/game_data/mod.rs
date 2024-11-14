pub mod units_data;

use units_data::items_data::{AVG_BOOTS_COST, AVG_LEGENDARY_ITEM_COST, AVG_SUPPORT_ITEM_COST};

use core::{fmt, ops};

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
/// a bit inflated (~+25%) to take other sources of golds into account (kills, towers, ...).
pub const CS_PER_MIN: f32 = 10.;
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

//fights simulation parameters:
/// Average time in seconds we consider between fights (used to weight items actives with different cooldowns).
const TIME_BETWEEN_FIGHTS: f32 = 180.;
/// Returns the availability coef of a passive/active effect according to its cooldown.
/// It should be used on effects that have cooldowns way longer than the fight simulation.
/// The function receives the real cooldown of the effect, already reduced by haste.
#[inline]
fn effect_availability_formula(real_cooldown: f32) -> f32 {
    TIME_BETWEEN_FIGHTS / (TIME_BETWEEN_FIGHTS + real_cooldown)
}

/// Contains a damage value divided in (`phys_dmg`, `magic_dmg`, `true_dmg`).
#[derive(Debug, Clone, Copy)]
pub struct PartDmg(pub f32, pub f32, pub f32);

impl fmt::Display for PartDmg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} phys, {} magic, {} true)", self.0, self.1, self.2)
    }
}

impl ops::Add for PartDmg {
    type Output = Self;

    #[must_use]
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl ops::Sub for PartDmg {
    type Output = Self;

    #[must_use]
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl ops::Mul<f32> for PartDmg {
    type Output = Self;

    #[must_use]
    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}

impl ops::Mul<PartDmg> for f32 {
    type Output = PartDmg;

    #[must_use]
    #[inline]
    fn mul(self, rhs: PartDmg) -> Self::Output {
        PartDmg(self * rhs.0, self * rhs.1, self * rhs.2)
    }
}

impl ops::Div<f32> for PartDmg {
    type Output = Self;

    #[must_use]
    #[inline]
    fn div(self, rhs: f32) -> Self::Output {
        Self(self.0 / rhs, self.1 / rhs, self.2 / rhs)
    }
}

impl ops::AddAssign for PartDmg {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
        self.2 += rhs.2;
    }
}

impl ops::SubAssign for PartDmg {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
        self.1 -= rhs.1;
        self.2 -= rhs.2;
    }
}

impl ops::MulAssign<f32> for PartDmg {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        self.0 *= rhs;
        self.1 *= rhs;
        self.2 *= rhs;
    }
}

impl ops::DivAssign<f32> for PartDmg {
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        self.0 /= rhs;
        self.1 /= rhs;
        self.2 /= rhs;
    }
}

impl PartDmg {
    /// Returns the sum of each dmg type (`phys_dmg`, `magic_dmg`, `true_dmg`) contained in the dmg value.
    #[must_use]
    #[inline]
    pub fn as_sum(&self) -> f32 {
        self.0 + self.1 + self.2
    }
}
