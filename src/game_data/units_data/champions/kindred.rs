use crate::{game_data::*, AVG_ITEM_COST_WITH_BOOTS};

use items_data::Item;
use runes_data::*;
use units_data::*;

use enumset::enum_set;

//champion parameters (constants):
const MARKS_PER_MIN: f32 = 4. / 15.; //assumes 4 marks at 15min
const MARKS_PER_ITEM: f32 = MARKS_PER_MIN * AVG_ITEM_COST_WITH_BOOTS / TOT_GOLDS_PER_MIN; //use marks per items instead of marks per min to not bias towards expensive items
const Q_N_TARGETS: f32 = 1.5;

fn kindred_init_abilities(champ: &mut Unit) {
    #[allow(clippy::cast_precision_loss)] //`build.item_count()` is well within f32 precision range
    let marks: f32 = (champ.build.item_count() as f32) * MARKS_PER_ITEM;
    champ.effects_values[EffectValueId::KindredMarks] = marks;

    champ.effects_values[EffectValueId::KindredDanceOfArrowsBonusAS] = 0.;

    champ.effects_values[EffectValueId::KindredHuntersVigorLastTriggerDistance] =
        -(HUNTERS_VIGOR_TRAVEL_REQUIRED + F32_TOL); // to allow for effect at time == 0

    champ.effects_values[EffectValueId::KindredWolfsFrenzyLastHitTime] = 0.;
    champ.effects_values[EffectValueId::KindredWolfsFrenzyLastStartTime] =
        -(WOLFS_FRENZY_DURATION + F32_TOL);
    champ.effects_values[EffectValueId::KindredWolfsFrenzyMagicDmgPerSec] = 0.;

    champ.effects_stacks[EffectStackId::KindredMountingDreadStacks] = 0;
    champ.effects_values[EffectValueId::KindredMountingDreadLastStackTime] =
        -(MOUNTING_DREAD_DELAY + F32_TOL); //to allow for effect at time == 0
}

const HUNTERS_VIGOR_TRAVEL_REQUIRED: f32 = 2500.; //in game, looks like it's a bit higher than the default `ENERGIZED_ATTACKS_TRAVEL_REQUIRED`

fn kindred_on_basic_attack_hit(
    champ: &mut Unit,
    target_stats: &UnitStats,
    _n_targets: f32,
    from_other_effects: bool,
) -> PartDmg {
    //hunter's vigor
    if !from_other_effects {
        if champ.units_travelled
            - champ.effects_values[EffectValueId::KindredHuntersVigorLastTriggerDistance]
            < HUNTERS_VIGOR_TRAVEL_REQUIRED
        {
            champ.effects_values[EffectValueId::KindredHuntersVigorLastTriggerDistance] -=
                HUNTERS_VIGOR_TRAVEL_REQUIRED * (5. / 100.); //5 stacks per basic attack (this is different than the default)
        }
        champ.effects_values[EffectValueId::KindredHuntersVigorLastTriggerDistance] =
            champ.units_travelled;
        champ.periodic_heals_shields += f32::min(1., 1.25 * MEAN_MISSING_HP_PERCENT)
            * HUNTERS_VIGOR_HEAL_BY_LVL[usize::from(champ.lvl.get() - 1)];
    }

    //mounting dread stacks
    if champ.effects_stacks[EffectStackId::KindredMountingDreadStacks] == 0 {
        return PartDmg(0., 0., 0.);
    }
    if champ.time - champ.effects_values[EffectValueId::KindredMountingDreadLastStackTime]
        >= MOUNTING_DREAD_DELAY
    {
        champ.effects_stacks[EffectStackId::KindredMountingDreadStacks] = 0;
        return PartDmg(0., 0., 0.);
    }
    if champ.effects_stacks[EffectStackId::KindredMountingDreadStacks] < 4 - 1 {
        champ.effects_stacks[EffectStackId::KindredMountingDreadStacks] += 1;
        champ.effects_values[EffectValueId::KindredMountingDreadLastStackTime] = champ.time;
        return PartDmg(0., 0., 0.);
    }
    champ.effects_stacks[EffectStackId::KindredMountingDreadStacks] = 0;

    //base dmg (doesn't crit)
    let base_phys_dmg: f32 =
        MOUNTING_DREAD_BASE_PHYS_DMG_BY_E_LVL[usize::from(champ.e_lvl - 1)] + champ.stats.bonus_ad;

    //missing hp part crits with special formula
    let hp_crit_treshold: f32 = 0.25 + 0.5 * champ.stats.crit_chance;
    let missing_hp_crit_chance: f32 = hp_crit_treshold * hp_crit_treshold;
    let missing_hp_dmg: f32 = (0.05 + 0.005 * champ.effects_values[EffectValueId::KindredMarks])
        * (MEAN_MISSING_HP_PERCENT * target_stats.hp)
        * (1. + missing_hp_crit_chance * (0.5 + champ.stats.crit_dmg - Unit::BASE_CRIT_DMG));

    PartDmg(base_phys_dmg + missing_hp_dmg, 0., 0.)
}

fn kindred_dance_of_arrows_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_values[EffectValueId::KindredDanceOfArrowsBonusAS] == 0. {
        let bonus_as_buff: f32 = 0.35 + 0.05 * champ.effects_values[EffectValueId::KindredMarks];
        champ.stats.bonus_as += bonus_as_buff;
        champ.effects_values[EffectValueId::KindredDanceOfArrowsBonusAS] = bonus_as_buff;
    }
}

fn kindred_dance_of_arrows_disable(champ: &mut Unit) {
    champ.stats.bonus_as -= champ.effects_values[EffectValueId::KindredDanceOfArrowsBonusAS];
    champ.effects_values[EffectValueId::KindredDanceOfArrowsBonusAS] = 0.;
}

const KINDRED_DANCE_OF_ARROWS_AS: TemporaryEffect = TemporaryEffect {
    id: EffectId::KindredDanceOfArrowsAS,
    add_stack: kindred_dance_of_arrows_enable,
    remove_every_stack: kindred_dance_of_arrows_disable,
    duration: 4.,
    cooldown: 0.,
};

const Q_PHYS_DMG_BY_Q_LVL: [f32; 5] = [40., 65., 90., 115., 140.];
const Q_STATIC_CD_BY_Q_LVL: [f32; 5] = [4., 3.5, 3., 2.5, 2.];
const Q_DASH_DISTANCE: f32 = 300.;

fn kindred_q(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    let q_lvl_idx: usize = usize::from(champ.q_lvl - 1); //to index ability ratios by lvl

    champ.add_temporary_effect(&KINDRED_DANCE_OF_ARROWS_AS, 0.);

    //dash
    champ.wait(Q_DASH_DISTANCE / (500. + champ.stats.ms())); //dash time
    champ.units_travelled += Q_DASH_DISTANCE;

    //reset basic attack cd
    champ.basic_attack_cd = 0.;

    //reduce q cd if w field is active
    if (champ.time - champ.effects_values[EffectValueId::KindredWolfsFrenzyLastStartTime]
        < WOLFS_FRENZY_DURATION)
        && (champ.q_cd > Q_STATIC_CD_BY_Q_LVL[q_lvl_idx])
    {
        champ.q_cd = Q_STATIC_CD_BY_Q_LVL[q_lvl_idx];
    }

    let phys_dmg: f32 =
        Q_N_TARGETS * (Q_PHYS_DMG_BY_Q_LVL[q_lvl_idx] + 0.75 * champ.stats.bonus_ad);

    champ.dmg_on_target(
        target_stats,
        PartDmg(phys_dmg, 0., 0.),
        (1, 1),
        enum_set!(DmgTag::Ability),
        Q_N_TARGETS,
    )
}

const WOLFS_FRENZY_MAGIC_DMG_BY_W_LVL: [f32; 5] = [25., 30., 35., 40., 45.];
const WOLFS_FRENZY_DURATION: f32 = 8.5; //secs
const WOLF_BASE_AS: f32 = KINDRED_BASE_AS * 0.893; //estimated to match timings measured (manually) in game
const WOLF_AS_RATIO: f32 = WOLF_BASE_AS;
const HUNTERS_VIGOR_HEAL_BY_LVL: [f32; MAX_UNIT_LVL] = [
    47., //lvl 1
    49., //lvl 2
    51., //lvl 3
    53., //lvl 4
    55., //lvl 5
    57., //lvl 6
    59., //lvl 7
    61., //lvl 8
    63., //lvl 9
    65., //lvl 10
    67., //lvl 11
    69., //lvl 12
    71., //lvl 13
    73., //lvl 14
    75., //lvl 15
    77., //lvl 16
    79., //lvl 17
    81., //lvl 18
];

fn kindred_w(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    //correct w cd (haste only reduces cooldown after the field)
    let calculated_cooldown: f32 = haste_formula(champ.stats.ability_haste_basic())
        * champ.properties.w.base_cooldown_by_ability_lvl[usize::from(champ.w_lvl - 1)];
    let time_spent: f32 = calculated_cooldown - champ.w_cd; //no time should have been spent since cooldown was set, but we still compensate just in case
    champ.w_cd = WOLFS_FRENZY_DURATION
        + haste_formula(champ.stats.ability_haste_basic())
            * (champ.properties.w.base_cooldown_by_ability_lvl[usize::from(champ.w_lvl - 1)]
                - WOLFS_FRENZY_DURATION)
        - time_spent;

    //reduce q cd
    let static_q_cd: f32 = Q_STATIC_CD_BY_Q_LVL[usize::from(champ.q_lvl - 1)];
    if champ.q_cd > static_q_cd {
        champ.q_cd = static_q_cd;
    }

    let w_lvl_idx: usize = usize::from(champ.w_lvl - 1); //to index ability ratios by lvl
    let wolf_as: f32 = WOLF_BASE_AS + WOLF_AS_RATIO * 0.25 * champ.stats.bonus_as; //unlike champions, wolf attack speed is not limited at 2.5
    let wolf_magic_dmg: f32 = WOLFS_FRENZY_MAGIC_DMG_BY_W_LVL[w_lvl_idx]
        + 0.2 * champ.stats.bonus_ad
        + 0.2 * champ.stats.ap()
        + (0.015 + 0.01 * champ.effects_values[EffectValueId::KindredMarks])
            * ((1. - MEAN_MISSING_HP_PERCENT) * target_stats.hp); //dmg per wolf's hit

    //assumes kindred stays in the field
    champ.effects_values[EffectValueId::KindredWolfsFrenzyMagicDmgPerSec] =
        (1. * wolf_as) * wolf_magic_dmg;
    champ.effects_values[EffectValueId::KindredWolfsFrenzyLastStartTime] = champ.time;
    PartDmg(0., 0., 0.)
}

fn kindred_on_any_hit(champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    //wolf's frenzy attacks over time (haven't found a better way to implement this)
    if champ.time - champ.effects_values[EffectValueId::KindredWolfsFrenzyLastStartTime]
        < WOLFS_FRENZY_DURATION
    {
        let time_elapsed: f32 =
            champ.time - champ.effects_values[EffectValueId::KindredWolfsFrenzyLastHitTime];
        champ.effects_values[EffectValueId::KindredWolfsFrenzyLastHitTime] = champ.time;
        return PartDmg(
            0.,
            time_elapsed * champ.effects_values[EffectValueId::KindredWolfsFrenzyMagicDmgPerSec],
            0.,
        );
    }
    PartDmg(0., 0., 0.)
}

const MOUNTING_DREAD_BASE_PHYS_DMG_BY_E_LVL: [f32; 5] = [80., 110., 140., 170., 200.];
const MOUNTING_DREAD_DELAY: f32 = 4.;

fn kindred_e(champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    //slow not implemented
    champ.effects_stacks[EffectStackId::KindredMountingDreadStacks] = 1;
    champ.effects_values[EffectValueId::KindredMountingDreadLastStackTime] = champ.time;
    PartDmg(0., 0., 0.)
}

fn kindred_r(_champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    //does nothing
    PartDmg(0., 0., 0.)
}

fn kindred_fight_scenario(champ: &mut Unit, target_stats: &UnitStats, fight_duration: f32) {
    //start with a basic attack
    champ.basic_attack(target_stats);

    while champ.time < fight_duration {
        //priority order: q, w, basic attack, e
        if champ.q_cd == 0. {
            champ.q(target_stats);
        } else if champ.w_cd == 0. {
            champ.w(target_stats);
        } else if champ.basic_attack_cd == 0. {
            champ.basic_attack(target_stats);
        } else if champ.e_cd == 0. {
            champ.e(target_stats);
        } else {
            champ.walk(
                F32_TOL
                    + [
                        champ.q_cd,
                        champ.w_cd,
                        champ.basic_attack_cd,
                        champ.e_cd,
                        f32::max(0., fight_duration - champ.time),
                    ]
                    .into_iter()
                    .min_by(|a, b| a.partial_cmp(b).expect("Failed to compare floats"))
                    .unwrap(),
            );
        }
    }
}

const KINDRED_BASE_AS: f32 = 0.625;
impl Unit {
    pub const KINDRED_PROPERTIES: UnitProperties = UnitProperties {
        name: "Kindred",
        as_limit: Unit::DEFAULT_AS_LIMIT,
        as_ratio: KINDRED_BASE_AS, //if not specified, same as base AS
        windup_percent: 0.17544,
        windup_modifier: 1., //"mod" next to attack windup, 1 by default
        base_stats: UnitStats {
            hp: 595.,
            mana: 300.,
            base_ad: 65.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 29.,
            mr: 30.,
            base_as: KINDRED_BASE_AS,
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
            hp: 104.,
            mana: 35.,
            base_ad: 3.25,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 4.7,
            mr: 1.3,
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
        basic_attack: units_data::default_basic_attack,
        q: BasicAbility {
            cast: kindred_q,
            cast_time: F32_TOL,
            base_cooldown_by_ability_lvl: [9., 9., 9., 9., 9., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        w: BasicAbility {
            cast: kindred_w,
            cast_time: F32_TOL,
            base_cooldown_by_ability_lvl: [18., 17., 16., 15., 14., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        e: BasicAbility {
            cast: kindred_e,
            cast_time: 0.25,
            base_cooldown_by_ability_lvl: [14., 12.5, 11., 9.5, 8., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        r: UltimateAbility {
            cast: kindred_r,
            cast_time: F32_TOL,
            base_cooldown_by_ability_lvl: [180., 150., 120.],
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(kindred_init_abilities),
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: Some(kindred_on_basic_attack_hit),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: Some(kindred_on_any_hit),
        },
        fight_scenarios: &[(kindred_fight_scenario, "all out")],
        defaults: UnitDefaults {
            runes_pages: RunesPage {
                keystone: &RuneKeystone::PRESS_THE_ATTACK, //todo: prone to change
                shard1: RuneShard::Middle,
                shard2: RuneShard::Left,
                shard3: RuneShard::Right,
            },
            skill_order: SkillOrder {
                //lvls:
                //  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18
                q: [1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                w: [0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0],
                e: [0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1],
                r: [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0],
            },
            legendary_items_pool: &[
                //&Item::ABYSSAL_MASK,
                //&Item::AXIOM_ARC,
                //&Item::BANSHEES_VEIL,
                &Item::BLACK_CLEAVER,
                //&Item::BLACKFIRE_TORCH,
                &Item::BLADE_OF_THE_RUINED_KING,
                &Item::BLOODTHIRSTER,
                &Item::CHEMPUNK_CHAINSWORD,
                //&Item::COSMIC_DRIVE,
                //&Item::CRYPTBLOOM,
                &Item::DEAD_MANS_PLATE,
                &Item::DEATHS_DANCE,
                &Item::ECLIPSE,
                &Item::EDGE_OF_NIGHT,
                &Item::ESSENCE_REAVER,
                //&Item::EXPERIMENTAL_HEXPLATE,
                &Item::FROZEN_HEART,
                &Item::GUARDIAN_ANGEL,
                &Item::GUINSOOS_RAGEBLADE,
                //&Item::HEXTECH_ROCKETBELT,
                //&Item::HORIZON_FOCUS,
                &Item::HUBRIS,
                &Item::HULLBREAKER,
                &Item::ICEBORN_GAUNTLET,
                &Item::IMMORTAL_SHIELDBOW,
                &Item::INFINITY_EDGE,
                &Item::JAKSHO,
                &Item::KAENIC_ROOKERN,
                &Item::KRAKEN_SLAYER,
                //&Item::LIANDRYS_TORMENT,
                //&Item::LICH_BANE,
                &Item::LORD_DOMINIKS_REGARDS,
                // &Item::LUDENS_COMPANION,
                &Item::MALIGNANCE,
                &Item::MAW_OF_MALMORTIUS,
                &Item::MERCURIAL_SCIMITAR,
                //&Item::MORELLONOMICON,
                &Item::MORTAL_REMINDER,
                &Item::MURAMANA,
                //&Item::NASHORS_TOOTH,
                &Item::NAVORI_FLICKERBLADE,
                &Item::OPPORTUNITY,
                &Item::OVERLORDS_BLOODMAIL,
                &Item::PHANTOM_DANCER,
                &Item::PROFANE_HYDRA,
                //&Item::RABADONS_DEATHCAP,
                &Item::RANDUINS_OMEN,
                &Item::RAPID_FIRECANNON,
                &Item::RAVENOUS_HYDRA,
                //&Item::RIFTMAKER,
                //&Item::ROD_OF_AGES,
                &Item::RUNAANS_HURRICANE,
                //&Item::RYLAIS_CRYSTAL_SCEPTER,
                //&Item::SERAPHS_EMBRACE,
                &Item::SERPENTS_FANG,
                &Item::SERYLDAS_GRUDGE,
                //&Item::SHADOWFLAME,
                &Item::SPEAR_OF_SHOJIN,
                &Item::STATIKK_SHIV,
                &Item::STERAKS_GAGE,
                //&Item::STORMSURGE,
                &Item::STRIDEBREAKER,
                &Item::SUNDERED_SKY,
                &Item::TERMINUS,
                &Item::THE_COLLECTOR,
                &Item::TITANIC_HYDRA,
                &Item::TRINITY_FORCE,
                &Item::UMBRAL_GLAIVE,
                //&Item::VOID_STAFF,
                &Item::VOLTAIC_CYCLOSWORD,
                &Item::WITS_END,
                &Item::YOUMUUS_GHOSTBLADE,
                &Item::YUN_TAL_WILDARROWS,
                //&Item::ZHONYAS_HOURGLASS,
            ],
            boots_pool: &[
                &Item::BERSERKERS_GREAVES,
                &Item::BOOTS_OF_SWIFTNESS,
                &Item::IONIAN_BOOTS_OF_LUCIDITY,
                //&Item::MERCURYS_TREADS,
                //&Item::PLATED_STEELCAPS,
                //&Item::SORCERERS_SHOES,
            ],
            supp_items_pool: &[],
        },
    };
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    use super::*;

    #[test]
    pub fn test_constant_parameters() {
        assert!(
            Unit::KINDRED_PROPERTIES.w.base_cooldown_by_ability_lvl[..5]
                .iter()
                .all(|&cd| cd >= WOLFS_FRENZY_DURATION),
            "Cooldown of Kindred W must be greater than the duration of its field ({}), this is due to how the cooldown reduced by haste is calculated",
            WOLFS_FRENZY_DURATION
        )
    }
}
