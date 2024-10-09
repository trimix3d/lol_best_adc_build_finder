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
/// For now, only implements runes shards.
#[derive(Debug, Clone)]
pub struct RunesPage {
    pub keystone: &'static RuneKeystone,
    pub shard1: RuneShard,
    pub shard2: RuneShard,
    pub shard3: RuneShard,
}

impl Default for RunesPage {
    /// Returns runes pages with only Left `RuneShards`.
    fn default() -> Self {
        Self::const_default()
    }
}

impl RunesPage {
    /// Returns runes pages with only Left `RuneShards`.
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

    #[allow(dead_code)]
    #[must_use]
    #[inline]
    pub fn get_runes(&self) -> &RunesPage {
        &self.runes_page
    }

    /// Updates unit runes stats (stats only coming from runes).
    ///
    /// Because of runes hp by lvl and adaptive force, runes stats actually depend on lvl and items as well.
    /// For this reason, this function must be ran after being sure that `Unit.lvl_stats` and `Unit.items_stats` are up to date.
    /// This also means that runes stats might become out of date after changing lvl/items.
    pub(crate) fn update_runes_stats(&mut self) {
        self.runes_stats.clear();

        //adaptive force doesn't count in champions passives, so it only depends on items stats in practise
        let runes_adaptive_bonus_ad: f32;
        let runes_adaptive_ap_flat: f32;
        if self.adaptive_is_phys() {
            runes_adaptive_bonus_ad = 5.4;
            runes_adaptive_ap_flat = 0.;
        } else {
            runes_adaptive_bonus_ad = 0.;
            runes_adaptive_ap_flat = 9.;
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

const PRESS_THE_ATTACK_DMG_BY_LVL: [f32; MAX_UNIT_LVL] = [
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
    from_other_effect: bool,
) -> PartDmg {
    if from_other_effect
        || champ.effects_stacks[EffectStackId::PressTheAttackStacks] == PRESS_THE_ATTACK_MAX_STACKS
    {
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
    let adaptive_dmg: f32 = PRESS_THE_ATTACK_DMG_BY_LVL[usize::from(champ.lvl.get() - 1)];
    if champ.adaptive_is_phys() {
        //todo: adaptive condition
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

//todo: lethal tempo

//todo: fleet footwork

//todo: conqueror

//todo: electrocute

//todo: dark harvest

//todo: hail of blades

//todo: summon aery

//todo: arcane comet

//todo: phase rush

//todo: grasp of the undying

//todo: aftershock

//todo: first strike
