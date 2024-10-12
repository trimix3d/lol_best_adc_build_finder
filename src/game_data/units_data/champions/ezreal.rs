use crate::game_data::*;

use items_data::Item;
use runes_data::*;
use units_data::*;

use enumset::enum_set;

//champion parameters (constants):
const EZREAL_Q_HIT_PERCENT: f32 = 0.9;
const EZREAL_W_HIT_PERCENT: f32 = 0.8;
/// Number of targets hit by ezreal R.
const EZREAL_R_N_TARGETS: f32 = 1.;
const EZREAL_R_HIT_PERCENT: f32 = 1.; // !!! 0.85;

fn ezreal_init_abilities(champ: &mut Unit) {
    champ.effects_stacks[EffectStackId::EzrealEssenceFluxMark] = 0;
    champ.effects_values[EffectValueId::EzrealEssenceFluxHitTime] =
        -(EZREAL_W_MARK_DURATION + F32_TOL); //to allow for effect at time == 0
    champ.effects_stacks[EffectStackId::EzrealRisingSpellForceStacks] = 0;
    champ.effects_values[EffectValueId::EzrealRisingSpellForceBonusAS] = 0.;
}

fn ezreal_basic_attack(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    //w mark + basic attack
    ezreal_detonate_w_mark_if_any(champ, target_stats)
        + units_data::default_basic_attack(champ, target_stats)
}

fn ezreal_rising_spell_force_add_stack(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_stacks[EffectStackId::EzrealRisingSpellForceStacks] < 5 {
        champ.effects_stacks[EffectStackId::EzrealRisingSpellForceStacks] += 1;
        let bonus_ad_buff: f32 = 0.10;
        champ.stats.bonus_as += bonus_ad_buff;
        champ.effects_values[EffectValueId::EzrealRisingSpellForceBonusAS] += bonus_ad_buff;
    }
}

fn ezreal_rising_spell_force_disable(champ: &mut Unit) {
    champ.stats.bonus_as -= champ.effects_values[EffectValueId::EzrealRisingSpellForceBonusAS];
    champ.effects_values[EffectValueId::EzrealRisingSpellForceBonusAS] = 0.;
    champ.effects_stacks[EffectStackId::EzrealRisingSpellForceStacks] = 0;
}

const EZREAL_RISING_SPELL_FORCE: TemporaryEffect = TemporaryEffect {
    id: EffectId::EzrealRisingSpellForce,
    add_stack: ezreal_rising_spell_force_add_stack,
    remove_every_stack: ezreal_rising_spell_force_disable,
    duration: 6.,
    cooldown: 0.,
};

const EZREAL_Q_PHYS_DMG_BY_Q_LVL: [f32; 5] = [20., 45., 70., 95., 120.];
const EZREAL_Q_CD_REFUND: f32 = 1.5;

fn ezreal_q(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    let w_mark_dmg: PartDmg = ezreal_detonate_w_mark_if_any(champ, target_stats);

    let q_lvl_idx: usize = usize::from(champ.q_lvl - 1); //to index ability ratios by lvl

    let phys_dmg: f32 =
        EZREAL_Q_PHYS_DMG_BY_Q_LVL[q_lvl_idx] + 1.30 * champ.stats.ad() + 0.15 * champ.stats.ap();

    //q hit reduces abilities cooldown
    champ.q_cd = f32::max(0., champ.q_cd - EZREAL_Q_HIT_PERCENT * EZREAL_Q_CD_REFUND);
    champ.w_cd = f32::max(0., champ.w_cd - EZREAL_Q_HIT_PERCENT * EZREAL_Q_CD_REFUND);
    champ.e_cd = f32::max(0., champ.e_cd - EZREAL_Q_HIT_PERCENT * EZREAL_Q_CD_REFUND);
    champ.r_cd = f32::max(0., champ.r_cd - EZREAL_Q_HIT_PERCENT * EZREAL_Q_CD_REFUND);

    //add passive stack
    champ.add_temporary_effect(&EZREAL_RISING_SPELL_FORCE, 0.);

    w_mark_dmg
        + champ.dmg_on_target(
            target_stats,
            PartDmg(EZREAL_Q_HIT_PERCENT * phys_dmg, 0., 0.),
            (1, 1),
            enum_set!(DmgTag::Ability | DmgTag::BasicAttack),
            1.,
        )
}

const EZREAL_W_MARK_MAGIC_DMG_BY_W_LVL: [f32; 5] = [80., 135., 190., 245., 300.];
const EZREAL_W_MARK_AP_RATIO_BY_W_LVL: [f32; 5] = [0.70, 0.75, 0.80, 0.85, 0.90];
const EZREAL_W_MARK_DURATION: f32 = 4.;

fn ezreal_detonate_w_mark_if_any(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    if champ.effects_stacks[EffectStackId::EzrealEssenceFluxMark] == 0 {
        //if no mark, do nothing
        PartDmg(0., 0., 0.)
    } else if champ.time - champ.effects_values[EffectValueId::EzrealEssenceFluxHitTime]
        >= EZREAL_W_MARK_DURATION
    {
        //if mark from too long ago, reset
        champ.effects_stacks[EffectStackId::EzrealEssenceFluxMark] = 0; //reset mark
        PartDmg(0., 0., 0.)
    } else {
        //detonate mark
        champ.effects_stacks[EffectStackId::EzrealEssenceFluxMark] = 0;

        let w_lvl_idx: usize = usize::from(champ.w_lvl - 1); //to index ability ratios by lvl

        let magic_dmg: f32 = EZREAL_W_MARK_MAGIC_DMG_BY_W_LVL[w_lvl_idx]
            + champ.stats.bonus_ad
            + EZREAL_W_MARK_AP_RATIO_BY_W_LVL[w_lvl_idx] * champ.stats.ap();

        champ.dmg_on_target(
            target_stats,
            PartDmg(0., EZREAL_W_HIT_PERCENT * magic_dmg, 0.),
            (1, 1),
            enum_set!(DmgTag::Ability),
            1.,
        )
    }
}

fn ezreal_w(champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    champ.effects_stacks[EffectStackId::EzrealEssenceFluxMark] = 1;
    champ.effects_values[EffectValueId::EzrealEssenceFluxHitTime] = champ.time;

    //add passive stack
    champ.add_temporary_effect(&EZREAL_RISING_SPELL_FORCE, 0.);

    PartDmg(0., 0., 0.)
}

const EZREAL_E_MAGIC_DMG_BY_E_LVL: [f32; 5] = [80., 130., 180., 230., 280.];

fn ezreal_e(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    champ.units_travelled += 475.; //blink range

    let w_mark_dmg: PartDmg = ezreal_detonate_w_mark_if_any(champ, target_stats);

    //add passive stack
    champ.add_temporary_effect(&EZREAL_RISING_SPELL_FORCE, 0.);

    let e_lvl_idx: usize = usize::from(champ.e_lvl - 1); //to index ability ratios by lvl

    let magic_dmg: f32 = EZREAL_E_MAGIC_DMG_BY_E_LVL[e_lvl_idx]
        + 0.5 * champ.stats.bonus_ad
        + 0.75 * champ.stats.ap();

    w_mark_dmg
        + champ.dmg_on_target(
            target_stats,
            PartDmg(0., magic_dmg, 0.),
            (1, 1),
            enum_set!(DmgTag::Ability),
            1.,
        )
}

const EZREAL_R_MAGIC_DMG_BY_R_LVL: [f32; 3] = [350., 550., 750.];

fn ezreal_r(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    let w_mark_dmg: PartDmg = ezreal_detonate_w_mark_if_any(champ, target_stats);

    //add passive stack
    champ.add_temporary_effect(&EZREAL_RISING_SPELL_FORCE, 0.);

    let r_lvl_idx: usize = usize::from(champ.r_lvl - 1); //to index ability ratios by lvl

    let magic_dmg: f32 = EZREAL_R_N_TARGETS
        * (EZREAL_R_MAGIC_DMG_BY_R_LVL[r_lvl_idx] + champ.stats.bonus_ad + 0.9 * champ.stats.ap());

    w_mark_dmg
        + champ.dmg_on_target(
            target_stats,
            PartDmg(0., EZREAL_R_HIT_PERCENT * magic_dmg, 0.),
            (1, 1),
            enum_set!(DmgTag::Ability | DmgTag::Ultimate),
            EZREAL_R_N_TARGETS,
        )
}

fn ezreal_fight_scenario_basic_attack_in_between_abilities(
    champ: &mut Unit,
    target_stats: &UnitStats,
    fight_duration: f32,
) {
    //w and e at the beggining
    champ.w(target_stats);
    champ.e(target_stats);

    while champ.time < fight_duration {
        //priority order: w, q, basic attack
        if champ.w_cd == 0. {
            champ.w(target_stats);
        } else if champ.q_cd == 0. {
            champ.q(target_stats);
        } else if champ.basic_attack_cd == 0. {
            champ.basic_attack(target_stats);
        } else {
            champ.walk(
                F32_TOL
                    + [
                        champ.w_cd,
                        champ.q_cd,
                        champ.basic_attack_cd,
                        f32::max(0., fight_duration - champ.time),
                    ]
                    .into_iter()
                    .min_by(|a, b| a.partial_cmp(b).expect("Failed to compare floats"))
                    .unwrap(),
            );
        }
    }
    //add weighted r dmg at the end
    champ.weighted_r(target_stats);
}

fn ezreal_fight_scenario_only_abilities(
    champ: &mut Unit,
    target_stats: &UnitStats,
    fight_duration: f32,
) {
    //w and e at the beggining
    champ.w(target_stats);
    champ.e(target_stats);

    while champ.time < fight_duration {
        //priority order: w, q (no basic attack)
        if champ.w_cd == 0. {
            champ.w(target_stats);
        } else if champ.q_cd == 0. {
            champ.q(target_stats);
        } else {
            champ.walk(
                F32_TOL
                    + [
                        champ.w_cd,
                        champ.q_cd,
                        f32::max(0., fight_duration - champ.time),
                    ]
                    .into_iter()
                    .min_by(|a, b| a.partial_cmp(b).expect("Failed to compare floats"))
                    .unwrap(),
            );
        }
    }
    //add weighted r dmg at the end
    champ.weighted_r(target_stats);
}

const EZREAL_BASE_AS: f32 = 0.625;
impl Unit {
    pub const EZREAL_PROPERTIES: UnitProperties = UnitProperties {
        name: "Ezreal",
        as_limit: Unit::DEFAULT_AS_LIMIT,
        as_ratio: EZREAL_BASE_AS,
        windup_percent: 0.18839,
        windup_modifier: 1., //get it from https://leagueoflegends.fandom.com/wiki/List_of_champions/Basic_attacks, 1 by default
        base_stats: UnitStats {
            hp: 600.,
            mana: 375.,
            base_ad: 60.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 24.,
            mr: 30.,
            base_as: EZREAL_BASE_AS,
            bonus_as: 0.,
            ability_haste: 0.,
            basic_haste: 0.,
            ultimate_haste: 0.,
            item_haste: 0.,
            crit_chance: 0.,
            crit_dmg: Unit::BASE_CRIT_DMG,
            ms_flat: 325.,
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
            hp: 102.,
            mana: 70.,
            base_ad: 2.75,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 4.7,
            mr: 1.3,
            base_as: 0.,
            bonus_as: 0.025,
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
        basic_attack: ezreal_basic_attack,
        q: BasicAbility {
            cast: ezreal_q,
            cast_time: 0.25,
            base_cooldown_by_ability_lvl: [5.5, 5.25, 5., 4.75, 4.5, F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        w: BasicAbility {
            cast: ezreal_w,
            cast_time: 0.25,
            base_cooldown_by_ability_lvl: [8., 8., 8., 8., 8., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        e: BasicAbility {
            cast: ezreal_e,
            cast_time: 0.25,
            base_cooldown_by_ability_lvl: [26., 23., 20., 17., 14., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        r: UltimateAbility {
            cast: ezreal_r,
            cast_time: 1.,
            base_cooldown_by_ability_lvl: [120., 105., 90.],
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(ezreal_init_abilities),
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
        fight_scenarios: &[
            (
                ezreal_fight_scenario_basic_attack_in_between_abilities,
                "basic attack in between abilities",
            ),
            (
                ezreal_fight_scenario_only_abilities,
                "only launch abilities",
            ),
        ],
        defaults: UnitDefaults {
            runes_pages: RunesPage {
                keystone: &RuneKeystone::PRESS_THE_ATTACK, //PTA for short fights, conq better for long fights //todo: prone to change
                shard1: RuneShard::Left,
                shard2: RuneShard::Left,
                shard3: RuneShard::Left,
            },
            skill_order: SkillOrder {
                //lvls:
                //  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18
                q: [1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                e: [0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0],
                w: [0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1],
                r: [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0],
            },
            legendary_items_pool: &[
                //&Item::ABYSSAL_MASK,
                //&Item::AXIOM_ARC,
                &Item::BANSHEES_VEIL,
                &Item::BLACK_CLEAVER,
                &Item::BLACKFIRE_TORCH,
                &Item::BLADE_OF_THE_RUINED_KING,
                &Item::BLOODTHIRSTER,
                &Item::CHEMPUNK_CHAINSWORD,
                &Item::COSMIC_DRIVE,
                &Item::CRYPTBLOOM,
                //&Item::DEAD_MANS_PLATE, //passive handles dashes incorrectly
                &Item::DEATHS_DANCE,
                &Item::ECLIPSE,
                &Item::EDGE_OF_NIGHT,
                &Item::ESSENCE_REAVER,
                //&Item::EXPERIMENTAL_HEXPLATE,
                &Item::FROZEN_HEART,
                &Item::GUARDIAN_ANGEL,
                &Item::GUINSOOS_RAGEBLADE,
                //&Item::HEXTECH_ROCKETBELT,
                &Item::HORIZON_FOCUS,
                &Item::HUBRIS,
                &Item::HULLBREAKER,
                &Item::ICEBORN_GAUNTLET,
                &Item::IMMORTAL_SHIELDBOW,
                &Item::INFINITY_EDGE,
                //&Item::JAKSHO,
                //&Item::KAENIC_ROOKERN,
                &Item::KRAKEN_SLAYER,
                &Item::LIANDRYS_TORMENT,
                &Item::LICH_BANE,
                &Item::LORD_DOMINIKS_REGARDS,
                &Item::LUDENS_COMPANION,
                //&Item::MALIGNANCE,
                &Item::MAW_OF_MALMORTIUS,
                &Item::MERCURIAL_SCIMITAR,
                &Item::MORELLONOMICON,
                &Item::MORTAL_REMINDER,
                &Item::MURAMANA,
                &Item::NASHORS_TOOTH,
                &Item::NAVORI_FLICKERBLADE,
                &Item::OPPORTUNITY,
                &Item::OVERLORDS_BLOODMAIL,
                &Item::PHANTOM_DANCER,
                //&Item::PROFANE_HYDRA,
                &Item::RABADONS_DEATHCAP,
                //&Item::RANDUINS_OMEN,
                &Item::RAPID_FIRECANNON,
                //&Item::RAVENOUS_HYDRA,
                &Item::RIFTMAKER,
                &Item::ROD_OF_AGES,
                //&Item::RUNAANS_HURRICANE, //passive works with q but only when in basic attack range, so doesn't synergise well
                &Item::RYLAIS_CRYSTAL_SCEPTER,
                &Item::SERAPHS_EMBRACE,
                &Item::SERPENTS_FANG,
                &Item::SERYLDAS_GRUDGE,
                &Item::SHADOWFLAME,
                &Item::SPEAR_OF_SHOJIN,
                &Item::STATIKK_SHIV,
                &Item::STERAKS_GAGE,
                &Item::STORMSURGE,
                //&Item::STRIDEBREAKER,
                &Item::SUNDERED_SKY,
                &Item::TERMINUS,
                &Item::THE_COLLECTOR,
                &Item::TITANIC_HYDRA,
                &Item::TRINITY_FORCE,
                &Item::UMBRAL_GLAIVE,
                &Item::VOID_STAFF,
                &Item::VOLTAIC_CYCLOSWORD,
                &Item::WITS_END,
                &Item::YOUMUUS_GHOSTBLADE,
                //&Item::YUN_TAL_WILDARROWS, //q doesn't proc passive because cannot crit
                &Item::ZHONYAS_HOURGLASS,
            ],
            boots_pool: &[
                &Item::BERSERKERS_GREAVES,
                &Item::BOOTS_OF_SWIFTNESS,
                &Item::IONIAN_BOOTS_OF_LUCIDITY,
                //&Item::MERCURYS_TREADS,
                //&Item::PLATED_STEELCAPS,
                &Item::SORCERERS_SHOES,
            ],
            support_items_pool: &[],
        },
    };
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    use super::*;

    #[test]
    pub fn test_unit_defaults() {
        Unit::from_properties_defaults(&Unit::EZREAL_PROPERTIES, MIN_UNIT_LVL, Build::default())
            .expect("Failed to create unit");
    }
}
