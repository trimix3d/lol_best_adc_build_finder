use crate::game_data::units_data::*;

use items_data::items::*;

use enumset::enum_set;

//champion parameters (constants):
/// Number of feathers that must be on the ground before pressing e in fight simulation.
/// Must be less or equal to 8 (max number of feathers on the ground on 1 combo, more is unrealistic except with r).
const XAYAH_N_FEATHERS_BEFORE_RECALL: u8 = 6;
/// Average number of targets hit by feathers recall (e).
const XAYAH_FEATHERS_N_TARGETS: f32 = 1.10;
const XAYAH_Q_HIT_PERCENT: f32 = 0.9;

fn xayah_init_abilities(champ: &mut Unit) {
    champ.effects_stacks[EffectStackId::XayahNFeathersOnGround] = 0;
    champ.effects_stacks[EffectStackId::XayahCleanCutsStacks] = 0;
    champ.effects_values[EffectValueId::XayahWBasicAttackCoef] = 1.;
    champ.effects_values[EffectValueId::XayahDeadlyPlumageBonusAS] = 0.;
    champ.effects_values[EffectValueId::XayahDeadlyPlumageMsPercent] = 0.;
}

fn xayah_basic_attack(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    //if empowered by w, basic attack gives ms
    if champ.effects_values[EffectValueId::XayahWBasicAttackCoef] != 1. {
        champ.add_temporary_effect(&XAYAH_DEADLY_PLUMAGE_MS, 0.);
    }

    //launch feathers
    if champ.effects_stacks[EffectStackId::XayahCleanCutsStacks] != 0 {
        champ.effects_stacks[EffectStackId::XayahCleanCutsStacks] -= 1;
        champ.effects_stacks[EffectStackId::XayahNFeathersOnGround] += 1;
    }

    let phys_dmg: f32 = champ.effects_values[EffectValueId::XayahWBasicAttackCoef]
        * champ.stats.ad()
        * champ.stats.crit_coef();
    champ.dmg_on_target(
        target_stats,
        PartDmg(phys_dmg, 0., 0.),
        (1, 1),
        enum_set!(DmgTag::BasicAttack),
        1.,
    )
}

const XAYAH_CLEAN_CUTS_MAX_STACKS: u8 = 5;
const XAYAH_CLEAN_CUTS_STACKS_PER_ABILITY: u8 = 3;

const XAYAH_Q_PHYS_DMG_BY_Q_LVL: [f32; 5] = [45., 60., 75., 90., 105.];

fn xayah_q(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    champ.effects_stacks[EffectStackId::XayahCleanCutsStacks] = u8::min(
        XAYAH_CLEAN_CUTS_MAX_STACKS,
        champ.effects_stacks[EffectStackId::XayahCleanCutsStacks]
            + XAYAH_CLEAN_CUTS_STACKS_PER_ABILITY,
    );

    //put two feathers on ground
    champ.effects_stacks[EffectStackId::XayahNFeathersOnGround] += 2;

    let q_lvl_idx: usize = usize::from(champ.q_lvl - 1); //to index ability ratios by lvl
    let phys_dmg: f32 = 2. * (XAYAH_Q_PHYS_DMG_BY_Q_LVL[q_lvl_idx] + 0.5 * champ.stats.bonus_ad);

    champ.dmg_on_target(
        target_stats,
        PartDmg(XAYAH_Q_HIT_PERCENT * phys_dmg, 0., 0.),
        (2, 1),
        enum_set!(DmgTag::Ability),
        XAYAH_Q_HIT_PERCENT,
    )
}

const XAYAH_DEADLY_PLUMAGE_PERCENT_MS_BUFF: f32 = 0.30;

fn xayah_deadly_plumage_ms_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_values[EffectValueId::XayahDeadlyPlumageMsPercent] == 0. {
        champ.stats.ms_percent += XAYAH_DEADLY_PLUMAGE_PERCENT_MS_BUFF;
        champ.effects_values[EffectValueId::XayahDeadlyPlumageMsPercent] =
            XAYAH_DEADLY_PLUMAGE_PERCENT_MS_BUFF;
    }
}

fn xayah_deadly_plumage_ms_disable(champ: &mut Unit) {
    champ.stats.ms_percent -= champ.effects_values[EffectValueId::XayahDeadlyPlumageMsPercent];
    champ.effects_values[EffectValueId::XayahDeadlyPlumageMsPercent] = 0.;
}

const XAYAH_DEADLY_PLUMAGE_MS: TemporaryEffect = TemporaryEffect {
    id: EffectId::XayahDeadlyPlumageMS,
    add_stack: xayah_deadly_plumage_ms_enable,
    remove_every_stack: xayah_deadly_plumage_ms_disable,
    duration: 1.5,
    cooldown: 0.,
};

const XAYAH_W_BONUS_AS_BY_W_LVL: [f32; 5] = [0.35, 0.40, 0.45, 0.50, 0.55];

fn xayah_deadly_plumage_as_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_values[EffectValueId::XayahDeadlyPlumageBonusAS] == 0. {
        champ.effects_values[EffectValueId::XayahWBasicAttackCoef] = 1.2; //empower basic attacks
        let bonus_as_buff: f32 = XAYAH_W_BONUS_AS_BY_W_LVL[usize::from(champ.w_lvl - 1)];
        champ.stats.bonus_as += bonus_as_buff;
        champ.effects_values[EffectValueId::XayahDeadlyPlumageBonusAS] = bonus_as_buff;
    }
}

fn xayah_deadly_plumage_as_disable(champ: &mut Unit) {
    champ.effects_values[EffectValueId::XayahWBasicAttackCoef] = 1.;
    champ.stats.bonus_as -= champ.effects_values[EffectValueId::XayahDeadlyPlumageBonusAS];
    champ.effects_values[EffectValueId::XayahDeadlyPlumageBonusAS] = 0.;
}

const XAYAH_DEADLY_PLUMAGE_AS: TemporaryEffect = TemporaryEffect {
    id: EffectId::XayahDeadlyPlumageAS,
    add_stack: xayah_deadly_plumage_as_enable,
    remove_every_stack: xayah_deadly_plumage_as_disable,
    duration: 4.,
    cooldown: 0.,
};

fn xayah_w(champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    champ.effects_stacks[EffectStackId::XayahCleanCutsStacks] = u8::min(
        XAYAH_CLEAN_CUTS_MAX_STACKS,
        champ.effects_stacks[EffectStackId::XayahCleanCutsStacks]
            + XAYAH_CLEAN_CUTS_STACKS_PER_ABILITY,
    );
    champ.add_temporary_effect(&XAYAH_DEADLY_PLUMAGE_AS, 0.);
    PartDmg(0., 0., 0.)
}

const XAYAH_E_PHYS_DMG_PER_FEATHER_BY_E_LVL: [f32; 5] = [55., 65., 75., 85., 95.];

fn xayah_e(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    champ.effects_stacks[EffectStackId::XayahCleanCutsStacks] = u8::min(
        XAYAH_CLEAN_CUTS_MAX_STACKS,
        champ.effects_stacks[EffectStackId::XayahCleanCutsStacks]
            + XAYAH_CLEAN_CUTS_STACKS_PER_ABILITY,
    );
    let e_lvl_idx: usize = usize::from(champ.e_lvl - 1); //to index ability ratios by lvl

    //recall feathers
    let n: f32 = f32::from(champ.effects_stacks[EffectStackId::XayahNFeathersOnGround]); //number of feathers
    champ.effects_stacks[EffectStackId::XayahNFeathersOnGround] = 0;
    let mut phys_dmg: f32 = (XAYAH_E_PHYS_DMG_PER_FEATHER_BY_E_LVL[e_lvl_idx]
        + 0.6 * champ.stats.bonus_ad)
        * (1. + 0.75 * champ.stats.crit_chance); //dmg for 1 feather
    phys_dmg *= n - 0.05 * (0.5 * n * (n - 1.)); //dmg formula for n feathers (diminishing returns)

    champ.dmg_on_target(
        target_stats,
        PartDmg(XAYAH_FEATHERS_N_TARGETS * phys_dmg, 0., 0.),
        (1, 1),
        enum_set!(DmgTag::Ability),
        XAYAH_FEATHERS_N_TARGETS,
    )
}

const XAYAH_R_PHYS_DMG_BY_R_LVL: [f32; 3] = [200., 300., 400.];

fn xayah_r(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    champ.effects_stacks[EffectStackId::XayahCleanCutsStacks] = u8::min(
        XAYAH_CLEAN_CUTS_MAX_STACKS,
        champ.effects_stacks[EffectStackId::XayahCleanCutsStacks]
            + XAYAH_CLEAN_CUTS_STACKS_PER_ABILITY,
    );
    champ.walk(1.5);
    champ.effects_stacks[EffectStackId::XayahNFeathersOnGround] += 5;

    let r_lvl_idx: usize = usize::from(champ.r_lvl - 1); //to index ability ratios by lvl

    let phys_dmg: f32 = XAYAH_R_PHYS_DMG_BY_R_LVL[r_lvl_idx] + champ.stats.bonus_ad;

    champ.dmg_on_target(
        target_stats,
        PartDmg(phys_dmg, 0., 0.),
        (1, 1),
        enum_set!(DmgTag::Ability | DmgTag::Ultimate),
        1.,
    )
}

fn xayah_fight_scenario(champ: &mut Unit, target_stats: &UnitStats, fight_duration: f32) {
    while champ.time < fight_duration {
        //priority order: basic attack when too much clean cuts stacks, e when enough feathers on ground, q, w, basic attack
        if champ.effects_stacks[EffectStackId::XayahCleanCutsStacks]
            > XAYAH_CLEAN_CUTS_MAX_STACKS - XAYAH_CLEAN_CUTS_STACKS_PER_ABILITY
        {
            //wait for the basic attack cooldown if there is one
            if champ.basic_attack_cd != 0. {
                champ.walk(champ.basic_attack_cd + F32_TOL);
            }
            champ.basic_attack(target_stats);
        } else if champ.e_cd == 0.
            && champ.effects_stacks[EffectStackId::XayahNFeathersOnGround]
                >= XAYAH_N_FEATHERS_BEFORE_RECALL
        {
            champ.e(target_stats);
        } else if champ.q_cd == 0. {
            champ.q(target_stats);
        } else if champ.w_cd == 0. {
            champ.w(target_stats);
        } else if champ.basic_attack_cd == 0. {
            champ.basic_attack(target_stats);
        } else {
            champ.walk(
                F32_TOL
                    + [
                        champ.q_cd,
                        champ.w_cd,
                        if champ.effects_stacks[EffectStackId::XayahNFeathersOnGround]
                            >= XAYAH_N_FEATHERS_BEFORE_RECALL
                        {
                            champ.e_cd
                        } else {
                            fight_duration - champ.time
                        },
                        champ.basic_attack_cd,
                        f32::max(0., fight_duration - champ.time),
                    ]
                    .into_iter()
                    .min_by(|a, b| a.partial_cmp(b).expect("Failed to compare floats"))
                    .unwrap(),
            );
        }
    }
    //add weighted r at the end
    champ.weighted_r(target_stats);
}

const XAYAH_BASE_AS: f32 = 0.658;
impl Unit {
    pub const XAYAH_PROPERTIES: UnitProperties = UnitProperties {
        name: "Xayah",
        as_limit: Unit::DEFAULT_AS_LIMIT,
        as_ratio: XAYAH_BASE_AS, //if not specified, same as base AS
        windup_percent: 0.17687,
        windup_modifier: 1., //get it from https://leagueoflegends.fandom.com/wiki/List_of_champions/Basic_attacks, 1 by default
        base_stats: UnitStats {
            hp: 630.,
            mana: 340.,
            base_ad: 60.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 25.,
            mr: 30.,
            base_as: XAYAH_BASE_AS,
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
            hp: 107.,
            mana: 40.,
            base_ad: 3.5,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 4.2,
            mr: 1.3,
            base_as: 0.,
            bonus_as: 0.039,
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
        basic_attack: xayah_basic_attack,
        q: BasicAbility {
            cast: xayah_q,
            cast_time: 0.2, //average value
            base_cooldown_by_ability_lvl: [10., 0.5, 9., 8.5, 8., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        w: BasicAbility {
            cast: xayah_w,
            cast_time: F32_TOL,
            base_cooldown_by_ability_lvl: [20., 19., 18., 17., 16., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        e: BasicAbility {
            cast: xayah_e,
            cast_time: F32_TOL,
            base_cooldown_by_ability_lvl: [12., 11., 10., 9., 8., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        r: UltimateAbility {
            cast: xayah_r,
            cast_time: F32_TOL,
            base_cooldown_by_ability_lvl: [140., 120., 100.],
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(xayah_init_abilities),
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
        fight_scenarios: &[(xayah_fight_scenario, "all out")],
        unit_defaults: UnitDefaults {
            runes_pages: RunesPage {
                keystone: &RunesPage::EMPTY_RUNE_KEYSTONE, //todo: add keystone
                shard1: RuneShard::Middle,
                shard2: RuneShard::Left,
                shard3: RuneShard::Left,
            },
            skill_order: SkillOrder {
                //lvls:
                //  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18
                e: [0, 1, 0, 1, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                w: [0, 0, 1, 0, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0],
                q: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1],
                r: [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0],
            },
            legendary_items_pool: &[
                //&ABYSSAL_MASK,
                //&AXIOM_ARC,
                //&BANSHEES_VEIL,
                &BLACK_CLEAVER,
                //&BLACKFIRE_TORCH,
                &BLADE_OF_THE_RUINED_KING,
                &BLOODTHIRSTER,
                &CHEMPUNK_CHAINSWORD,
                //&COSMIC_DRIVE,
                //&CRYPTBLOOM,
                &DEAD_MANS_PLATE,
                &DEATHS_DANCE,
                &ECLIPSE,
                &EDGE_OF_NIGHT,
                &ESSENCE_REAVER,
                //&EXPERIMENTAL_HEXPLATE,
                //&FROZEN_HEART,
                &GUARDIAN_ANGEL,
                //&GUINSOOS_RAGEBLADE,
                //&HEXTECH_ROCKETBELT,
                //&HORIZON_FOCUS,
                &HUBRIS,
                &HULLBREAKER,
                //&ICEBORN_GAUNTLET,
                &IMMORTAL_SHIELDBOW,
                &INFINITY_EDGE,
                //&JAKSHO,
                //&KAENIC_ROOKERN,
                &KRAKEN_SLAYER,
                //&LIANDRYS_TORMENT,
                //&LICH_BANE,
                &LORD_DOMINIKS_REGARDS,
                //&LUDENS_COMPANION,
                //&MALIGNANCE,
                &MAW_OF_MALMORTIUS,
                &MERCURIAL_SCIMITAR,
                //&MORELLONOMICON,
                &MORTAL_REMINDER,
                &MURAMANA,
                //&NASHORS_TOOTH,
                &NAVORI_FLICKERBLADE,
                &OPPORTUNITY,
                &OVERLORDS_BLOODMAIL,
                &PHANTOM_DANCER,
                //&PROFANE_HYDRA,
                //&RABADONS_DEATHCAP,
                //&RANDUINS_OMEN,
                &RAPID_FIRECANNON,
                //&RAVENOUS_HYDRA,
                //&RIFTMAKER,
                //&ROD_OF_AGES,
                &RUNAANS_HURRICANE,
                //&RYLAIS_CRYSTAL_SCEPTER,
                //&SERAPHS_EMBRACE,
                &SERPENTS_FANG,
                &SERYLDAS_GRUDGE,
                //&SHADOWFLAME,
                &SPEAR_OF_SHOJIN,
                &STATIKK_SHIV,
                &STERAKS_GAGE,
                //&STORMSURGE,
                //&STRIDEBREAKER,
                &SUNDERED_SKY,
                &TERMINUS,
                &THE_COLLECTOR,
                &TITANIC_HYDRA,
                &TRINITY_FORCE,
                &UMBRAL_GLAIVE,
                //&VOID_STAFF,
                &VOLTAIC_CYCLOSWORD,
                &WITS_END,
                &YOUMUUS_GHOSTBLADE,
                &YUN_TAL_WILDARROWS,
                //&ZHONYAS_HOURGLASS,
            ],
            boots_pool: &[
                &BERSERKERS_GREAVES,
                &BOOTS_OF_SWIFTNESS,
                //&IONIAN_BOOTS_OF_LUCIDITY,
                //&MERCURYS_TREADS,
                //&PLATED_STEELCAPS,
                //&SORCERERS_SHOES,
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
    pub fn test_xayah_constant_parameters() {
        assert!(
            XAYAH_N_FEATHERS_BEFORE_RECALL <= 8,
            "Number of feathers before pressing Xayah E must be less or equal to 8 (got {})",
            XAYAH_N_FEATHERS_BEFORE_RECALL
        )
    }
}
