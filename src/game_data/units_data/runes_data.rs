use super::*;

#[derive(Debug)]
pub struct RuneKeystone {
    pub name: &'static str,
    pub on_action_fns: OnActionFns,
}

#[allow(dead_code)] //each shard is not always used
#[derive(Debug, Clone, Copy)]
pub enum RuneShard {
    Left,
    Middle,
    Right,
}

/// Represents the runes page of a champion.
/// Doesn't implement
#[derive(Debug, Clone)]
pub struct RunesPage {
    pub keystone: &'static RuneKeystone,
    pub shard1: RuneShard,
    pub shard2: RuneShard,
    pub shard3: RuneShard,
}

impl Default for RunesPage {
    /// Returns runes pages with an empty `RuneKeystone` and only Left `RuneShards`.
    fn default() -> Self {
        Self::const_default()
    }
}

/// Amount of AP given by adaptive runes shards.
const RUNES_SHARDS_ADAPTIVE_AP: f32 = 9.;

impl RunesPage {
    /// Returns runes pages with an empty `RuneKeystone` and only Left `RuneShards`.
    /// Provides a default valid value for `SkillOrder` usable in compile time constants (unlike `Default::default()` which is not const).
    #[must_use]
    pub const fn const_default() -> Self {
        Self {
            keystone: &RuneKeystone::EMPTY_RUNE_KEYSTONE,
            shard1: RuneShard::Left,
            shard2: RuneShard::Left,
            shard3: RuneShard::Left,
        }
    }
}

impl Unit {
    /// Sets the Unit runes, returns Ok if success or Err if failure (depending on the validity of the given runes page).
    /// In case of a failure, the unit is not modified.
    /// In the current state, this function will always succeed because all possible runes pages are valid (but it may change in the future).
    pub fn set_runes(&mut self, runes_page: RunesPage) -> Result<(), String> {
        self.runes_page = runes_page;
        Ok(())
    }

    /// Updates unit runes stats (stats only coming from runes).
    ///
    /// Because of runes hp by lvl and adaptive force, runes stats actually depend on lvl and items as well.
    /// For this reason, this function must be ran after being sure that `Unit.lvl_stats` and `Unit.items_stats` are up to date.
    /// This also means that runes stats might become out of date after changing lvl/items.
    pub(crate) fn update_runes_stats(&mut self) {
        self.runes_stats.clear();

        let runes_adaptive_bonus_ad: f32;
        let runes_adaptive_ap_flat: f32;
        if self.adaptive_is_phys() {
            runes_adaptive_bonus_ad = ADAPTIVE_AP_TO_AD_RATIO * RUNES_SHARDS_ADAPTIVE_AP;
            runes_adaptive_ap_flat = 0.;
        } else {
            runes_adaptive_bonus_ad = 0.;
            runes_adaptive_ap_flat = RUNES_SHARDS_ADAPTIVE_AP;
        }

        match self.runes_page.shard1 {
            RuneShard::Left => {
                self.runes_stats.bonus_ad += runes_adaptive_bonus_ad;
                self.runes_stats.ap_flat += runes_adaptive_ap_flat;
            }
            RuneShard::Middle => self.runes_stats.bonus_as += 0.10,
            RuneShard::Right => self.runes_stats.ability_haste += 8.,
        }

        match self.runes_page.shard2 {
            RuneShard::Left => {
                self.runes_stats.bonus_ad += runes_adaptive_bonus_ad;
                self.runes_stats.ap_flat += runes_adaptive_ap_flat;
            }
            RuneShard::Middle => self.runes_stats.ms_percent += 0.02,
            RuneShard::Right => self.runes_stats.hp += runes_hp_by_lvl(self.lvl),
        }

        match self.runes_page.shard3 {
            RuneShard::Left => self.runes_stats.hp += 65.,
            RuneShard::Middle => (), //tenacity & slow resist: do nothing (not implemented)
            RuneShard::Right => self.runes_stats.hp += runes_hp_by_lvl(self.lvl),
        }
    }
}

impl RuneKeystone {
    pub const EMPTY_RUNE_KEYSTONE: RuneKeystone = RuneKeystone {
        name: "Empty keystone",
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
    };
}

//press the attack
const PRESS_THE_ATTACK_DELAY: f32 = 4.;
fn press_the_attack_init(champ: &mut Unit) {
    champ.effects_stacks[EffectStackId::PressTheAttackStacks] = 0;
    champ.effects_values[EffectValueId::PressTheAttackLastStackTime] =
        -(PRESS_THE_ATTACK_DELAY + F32_TOL); //to allow for effect at time == 0
}

const PRESS_THE_ATTACK_ADAPTIVE_DMG_BY_LVL: [f32; MAX_UNIT_LVL] = [
    40.,    //lvl 1
    47.06,  //lvl 2
    54.12,  //lvl 3
    61.18,  //lvl 4
    68.24,  //lvl 5
    75.29,  //lvl 6
    82.35,  //lvl 7
    89.41,  //lvl 8
    96.47,  //lvl 9
    103.53, //lvl 10
    110.59, //lvl 11
    117.64, //lvl 12
    124.71, //lvl 13
    131.76, //lvl 14
    138.82, //lvl 15
    145.88, //lvl 16
    152.94, //lvl 17
    160.,   //lvl 18
];

const PRESS_THE_ATTACK_MAX_STACKS: u8 = 3;
const PRESS_THE_ATTACK_DMG_MODIFIER: f32 = 0.08;
fn press_the_attack_on_basic_attack_hit(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    _n_targets: f32,
    _from_other_effect: bool,
) -> PartDmg {
    //assumes champ doesn't leave combat when fully stacked
    if champ.effects_stacks[EffectStackId::PressTheAttackStacks] == PRESS_THE_ATTACK_MAX_STACKS {
        return PartDmg(0., 0., 0.);
    }
    //if last hit from too long ago, reset stacks and add 1
    if champ.time - champ.effects_values[EffectValueId::PressTheAttackLastStackTime]
        >= PRESS_THE_ATTACK_DELAY
    {
        champ.effects_values[EffectValueId::PressTheAttackLastStackTime] = champ.time;
        champ.effects_stacks[EffectStackId::PressTheAttackStacks] = 1;
        return PartDmg(0., 0., 0.);
    }
    //if last hit is recent enough (previous condition) but not fully stacked, add 1 stack
    if champ.effects_stacks[EffectStackId::PressTheAttackStacks] < PRESS_THE_ATTACK_MAX_STACKS - 1 {
        champ.effects_stacks[EffectStackId::PressTheAttackStacks] += 1;
        champ.effects_values[EffectValueId::PressTheAttackLastStackTime] = champ.time;
        return PartDmg(0., 0., 0.);
    }
    //if fully stacked (previous conditions), put stack value to max, update dmg modifiers and return dmg
    champ.effects_stacks[EffectStackId::PressTheAttackStacks] = PRESS_THE_ATTACK_MAX_STACKS;
    increase_exponentially_scaling_stat(
        &mut champ.stats.phys_dmg_modifier,
        PRESS_THE_ATTACK_DMG_MODIFIER,
    );
    increase_exponentially_scaling_stat(
        &mut champ.stats.magic_dmg_modifier,
        PRESS_THE_ATTACK_DMG_MODIFIER,
    );
    let adaptive_dmg: f32 = PRESS_THE_ATTACK_ADAPTIVE_DMG_BY_LVL[usize::from(champ.lvl.get() - 1)];
    if champ.adaptive_is_phys() {
        PartDmg(adaptive_dmg, 0., 0.)
    } else {
        PartDmg(0., adaptive_dmg, 0.)
    }
}

impl RuneKeystone {
    pub const PRESS_THE_ATTACK: RuneKeystone = RuneKeystone {
        name: "Press the attack",
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(press_the_attack_init),
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: Some(press_the_attack_on_basic_attack_hit),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//lethal tempo
fn lethal_tempo_init(champ: &mut Unit) {
    champ.effects_stacks[EffectStackId::LethalTempoStacks] = 0;
    champ.effects_values[EffectValueId::LethalTempoBonusAS] = 0.;
}

const LETHAL_TEMPO_MAX_STACKS: u8 = 6;
fn lethal_tempo_add_stack(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_stacks[EffectStackId::LethalTempoStacks] < LETHAL_TEMPO_MAX_STACKS {
        champ.effects_stacks[EffectStackId::LethalTempoStacks] += 1;
        let bonus_as_buff: f32 = 0.04; //ranged value
        champ.stats.bonus_as += bonus_as_buff;
        champ.effects_values[EffectValueId::LethalTempoBonusAS] += bonus_as_buff;
    }
}

fn lethal_tempo_remove_every_stack(champ: &mut Unit) {
    champ.stats.bonus_as -= champ.effects_values[EffectValueId::LethalTempoBonusAS];
    champ.effects_values[EffectValueId::LethalTempoBonusAS] = 0.;
    champ.effects_stacks[EffectStackId::LethalTempoStacks] = 0;
}

const LETHAL_TEMPO_AS: TemporaryEffect = TemporaryEffect {
    id: EffectId::LethalTempoAS,
    add_stack: lethal_tempo_add_stack,
    remove_every_stack: lethal_tempo_remove_every_stack,
    duration: 6.,
    cooldown: 0.,
};

fn lethal_tempo_on_basic_attack_cast(champ: &mut Unit) {
    champ.add_temporary_effect(&LETHAL_TEMPO_AS, 0.);
}

const LETHAL_TEMPO_ADAPTIVE_DMG_BY_LVL: [f32; MAX_UNIT_LVL] = [
    6.,    //lvl 1
    7.06,  //lvl 2
    8.12,  //lvl 3
    9.18,  //lvl 4
    10.24, //lvl 5
    11.29, //lvl 6
    12.35, //lvl 7
    13.41, //lvl 8
    14.47, //lvl 9
    15.53, //lvl 10
    16.59, //lvl 11
    17.65, //lvl 12
    18.71, //lvl 13
    19.76, //lvl 14
    20.82, //lvl 15
    21.88, //lvl 16
    22.94, //lvl 17
    24.,   //lvl 18
]; //ranged value

fn lethal_tempo_on_basic_attack_hit(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    _n_targets: f32,
    from_other_effect: bool,
) -> PartDmg {
    if from_other_effect
        || champ.effects_stacks[EffectStackId::LethalTempoStacks] < LETHAL_TEMPO_MAX_STACKS
    {
        return PartDmg(0., 0., 0.);
    }
    let adaptive_dmg: f32 = LETHAL_TEMPO_ADAPTIVE_DMG_BY_LVL[usize::from(champ.lvl.get() - 1)]
        * (1. + 0.66 * champ.stats.bonus_as); //ranged value
    if champ.adaptive_is_phys() {
        PartDmg(adaptive_dmg, 0., 0.)
    } else {
        PartDmg(0., adaptive_dmg, 0.)
    }
}

impl RuneKeystone {
    pub const LETHAL_TEMPO: RuneKeystone = RuneKeystone {
        name: "Lethal tempo",
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(lethal_tempo_init),
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: Some(lethal_tempo_on_basic_attack_cast),
            on_basic_attack_hit: Some(lethal_tempo_on_basic_attack_hit),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//fleet footwork
fn fleet_footwork_init(champ: &mut Unit) {
    champ.effects_values[EffectValueId::FleetFootworkLastTriggerDistance] =
        -(ENERGIZED_ATTACKS_TRAVEL_REQUIRED + F32_TOL); // to allow for effect at time == 0
    champ.effects_values[EffectValueId::FleetFootworkMSPercent] = 0.;
}

fn fleet_footwork_ms_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_values[EffectValueId::FleetFootworkMSPercent] == 0. {
        let ms_percent_buff: f32 = 0.15; //ranged value
        champ.stats.ms_percent += ms_percent_buff;
        champ.effects_values[EffectValueId::FleetFootworkMSPercent] = ms_percent_buff;
    }
}

fn fleet_footwork_ms_disable(champ: &mut Unit) {
    champ.stats.ms_percent -= champ.effects_values[EffectValueId::FleetFootworkMSPercent];
    champ.effects_values[EffectValueId::FleetFootworkMSPercent] = 0.;
}

const FLEET_FOOTWORK_MS: TemporaryEffect = TemporaryEffect {
    id: EffectId::FleetFootworkMS,
    add_stack: fleet_footwork_ms_enable,
    remove_every_stack: fleet_footwork_ms_disable,
    duration: 1.,
    cooldown: 0.,
};

fn fleet_footwork_on_basic_attack_hit(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    _n_targets: f32,
    from_other_effect: bool,
) -> PartDmg {
    if from_other_effect {
        return PartDmg(0., 0., 0.);
    }

    //if not enough energy, add basic attack energy stacks
    if champ.units_travelled - champ.effects_values[EffectValueId::FleetFootworkLastTriggerDistance]
        < ENERGIZED_ATTACKS_TRAVEL_REQUIRED
    {
        champ.effects_values[EffectValueId::FleetFootworkLastTriggerDistance] -=
            ENERGIZED_ATTACKS_TRAVEL_REQUIRED * (ENERGIZED_STACKS_PER_BASIC_ATTACK / 100.);
        return PartDmg(0., 0., 0.);
    }
    //if enough energy (previous condition), trigger energized attack
    champ.effects_values[EffectValueId::FleetFootworkLastTriggerDistance] = champ.units_travelled;
    champ.periodic_heals_shields += FLEET_FOOTWORK_HEAL_BY_LVL[usize::from(champ.lvl.get() - 1)]
        + 0.06 * champ.stats.bonus_ad
        + 0.03 * champ.stats.ap(); //ranged value
    champ.add_temporary_effect(&FLEET_FOOTWORK_MS, 0.);
    PartDmg(0., 0., 0.)
}

const FLEET_FOOTWORK_HEAL_BY_LVL: [f32; MAX_UNIT_LVL] = [
    6.,    //lvl 1
    9.05,  //lvl 2
    12.25, //lvl 3
    15.59, //lvl 4
    19.09, //lvl 5
    22.73, //lvl 6
    26.52, //lvl 7
    30.46, //lvl 8
    34.55, //lvl 9
    38.78, //lvl 10
    43.16, //lvl 11
    47.7,  //lvl 12
    52.38, //lvl 13
    57.2,  //lvl 14
    62.18, //lvl 15
    67.31, //lvl 16
    72.58, //lvl 17
    78.,   //lvl 18
]; //ranged value

impl RuneKeystone {
    pub const FLEET_FOOTWORK: RuneKeystone = RuneKeystone {
        name: "Fleet footwork",
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(fleet_footwork_init),
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: Some(fleet_footwork_on_basic_attack_hit),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//todo: conqueror
fn conqueror_init(champ: &mut Unit) {
    champ.effects_stacks[EffectStackId::ConquerorAdaptiveIsPhys] =
        u8::from(champ.adaptive_is_phys());
    champ.effects_stacks[EffectStackId::ConquerorStacks] = 0;
    champ.effects_values[EffectValueId::ConquerorAdaptiveAP] = 0.;
    champ.effects_values[EffectValueId::ConquerorLastAbilityHitTime] = -F32_TOL; //to allow for effect at time == 0
    champ.effects_values[EffectValueId::ConquerorLastBasicAttackHitTime] = -F32_TOL;
    //to allow for effect at time == 0
}

const CONQUEROR_ADAPTIVE_AP_PER_STACK_BY_LVL: [f32; MAX_UNIT_LVL] = [
    1.8,  //lvl 1
    1.93, //lvl 2
    2.06, //lvl 3
    2.19, //lvl 4
    2.32, //lvl 5
    2.45, //lvl 6
    2.58, //lvl 7
    2.71, //lvl 8
    2.84, //lvl 9
    2.96, //lvl 10
    3.09, //lvl 11
    3.22, //lvl 12
    3.35, //lvl 13
    3.48, //lvl 14
    3.61, //lvl 15
    3.74, //lvl 16
    3.87, //lvl 17
    4.,   //lvl 18
];

fn conqueror_add_stack(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_stacks[EffectStackId::ConquerorStacks] < 12 {
        champ.effects_stacks[EffectStackId::ConquerorStacks] += 1;

        let adaptive_buff: f32 =
            CONQUEROR_ADAPTIVE_AP_PER_STACK_BY_LVL[usize::from(champ.lvl.get() - 1)];
        champ.effects_values[EffectValueId::ConquerorAdaptiveAP] += adaptive_buff;

        if champ.effects_stacks[EffectStackId::ConquerorAdaptiveIsPhys] == 1 {
            champ.stats.bonus_ad += ADAPTIVE_AP_TO_AD_RATIO * adaptive_buff;
        } else {
            champ.stats.ap_flat += adaptive_buff;
        }
    }
}

fn conqueror_remove_every_stack(champ: &mut Unit) {
    if champ.effects_stacks[EffectStackId::ConquerorAdaptiveIsPhys] == 1 {
        champ.stats.bonus_ad -=
            ADAPTIVE_AP_TO_AD_RATIO * champ.effects_values[EffectValueId::ConquerorAdaptiveAP];
    } else {
        champ.stats.ap_flat -= champ.effects_values[EffectValueId::ConquerorAdaptiveAP];
    }
    champ.effects_stacks[EffectStackId::ConquerorStacks] = 0;
    champ.effects_values[EffectValueId::ConquerorAdaptiveAP] = 0.;
}

const CONQUEROR: TemporaryEffect = TemporaryEffect {
    id: EffectId::Conqueror,
    add_stack: conqueror_add_stack,
    remove_every_stack: conqueror_remove_every_stack,
    duration: 5.,
    cooldown: 0.,
};

fn conqueror_on_basic_attack_hit(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    _n_targets: f32,
    from_other_effect: bool,
) -> PartDmg {
    if from_other_effect {
        return PartDmg(0., 0., 0.);
    }

    //set last basic attack hit time only if real basic attack (not ability that applies basic attack effects)
    if champ.effects_values[EffectValueId::ConquerorLastAbilityHitTime] != champ.time {
        champ.effects_values[EffectValueId::ConquerorLastBasicAttackHitTime] = champ.time;
    }
    PartDmg(0., 0., 0.)
}

fn conqueror_on_ability_hit(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    _n_targets: f32,
) -> PartDmg {
    champ.effects_values[EffectValueId::ConquerorLastAbilityHitTime] = champ.time;
    PartDmg(0., 0., 0.)
}

fn conqueror_on_any_hit(champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    //basic attacks give 1 stacks, any other attacks give 2
    champ.add_temporary_effect(&CONQUEROR, 0.);

    //if not same instance of dmg as last basic attack hit -> apply second stack because not a basic attack
    if champ.time != champ.effects_values[EffectValueId::ConquerorLastBasicAttackHitTime] {
        champ.add_temporary_effect(&CONQUEROR, 0.);
    }
    PartDmg(0., 0., 0.)
}

impl RuneKeystone {
    pub const CONQUEROR: RuneKeystone = RuneKeystone {
        name: "Conqueror",
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(conqueror_init),
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: Some(conqueror_on_ability_hit),
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: Some(conqueror_on_basic_attack_hit),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: Some(conqueror_on_any_hit),
        },
    };
}

//todo: electrocute

//todo: dark harvest

//todo: hail of blades

//todo: summon aery

//todo: arcane comet

//todo: phase rush

//todo: grasp of the undying

//todo: aftershock

//todo: first strike

pub const ALL_RUNES_KEYSTONES: [RuneKeystone; 4] = [
    RuneKeystone::PRESS_THE_ATTACK,
    RuneKeystone::LETHAL_TEMPO,
    RuneKeystone::FLEET_FOOTWORK,
    RuneKeystone::CONQUEROR,
];
