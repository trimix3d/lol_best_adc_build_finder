use crate::game_data::{items_data::items::*, units_data::*};

use enumset::enum_set;

//champion parameters (constants):
const EZREAL_Q_HIT_PERCENT: f32 = 1.; //0.9 !!!
const EZREAL_W_HIT_PERCENT: f32 = 0.8;
/// Number of targets hit by ezreal R.
const EZREAL_R_N_TARGETS: f32 = 1.;
const EZREAL_R_HIT_PERCENT: f32 = 0.85;

fn ezreal_init_abilities(champ: &mut Unit) {
    champ.effects_stacks[EffectStackId::EzrealEssenceFluxMark] = 0;
    champ.effects_values[EffectValueId::EzrealEssenceFluxHitTime] =
        -(EZREAL_W_MARK_DURATION + F32_TOL); //to allow for effect at time == 0
    champ.effects_stacks[EffectStackId::EzrealRisingSpellForceStacks] = 0;
    champ.effects_values[EffectValueId::EzrealRisingSpellForceBonusAS] = 0.;
}

fn ezreal_basic_attack(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let w_mark_dmg: f32 = ezreal_detonate_w_mark_if_any(champ, target_stats);

    let phys_dmg: f32 = champ.stats.ad() * champ.stats.crit_coef();

    w_mark_dmg
        + champ.dmg_on_target(
            target_stats,
            (phys_dmg, 0., 0.),
            (1, 1),
            enum_set!(DmgTag::BasickAttack),
            1.,
        )
}

const EZREAL_RISING_SPELL_FORCE_BONUS_AS_PER_STACK: f32 = 0.10;

fn ezreal_rising_spell_force_add_stack(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_stacks[EffectStackId::EzrealRisingSpellForceStacks] < 5 {
        champ.effects_stacks[EffectStackId::EzrealRisingSpellForceStacks] += 1;
        champ.stats.bonus_as += EZREAL_RISING_SPELL_FORCE_BONUS_AS_PER_STACK;
        champ.effects_values[EffectValueId::EzrealRisingSpellForceBonusAS] +=
            EZREAL_RISING_SPELL_FORCE_BONUS_AS_PER_STACK;
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

fn ezreal_q(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let w_mark_dmg: f32 = ezreal_detonate_w_mark_if_any(champ, target_stats);

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

    //todo: check behavior after on action fn changes (with luden, shojin, etc)
    w_mark_dmg
        + champ.dmg_on_target(
            target_stats,
            (EZREAL_Q_HIT_PERCENT * phys_dmg, 0., 0.),
            (1, 1),
            enum_set!(DmgTag::Ability | DmgTag::BasickAttack),
            EZREAL_Q_HIT_PERCENT,
        )
}

const EZREAL_W_MARK_MAGIC_DMG_BY_W_LVL: [f32; 5] = [80., 135., 190., 245., 300.];
const EZREAL_W_MARK_AP_RATIO_BY_W_LVL: [f32; 5] = [0.70, 0.75, 0.80, 0.85, 0.90];
const EZREAL_W_MARK_DURATION: f32 = 4.;

fn ezreal_detonate_w_mark_if_any(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    if champ.effects_stacks[EffectStackId::EzrealEssenceFluxMark] == 0 {
        //if no mark, return
        return 0.;
    } else if champ.time - champ.effects_values[EffectValueId::EzrealEssenceFluxHitTime]
        >= EZREAL_W_MARK_DURATION
    {
        //if mark from too long ago, reset
        champ.effects_stacks[EffectStackId::EzrealEssenceFluxMark] = 0; //reset mark
        return 0.;
    } else {
        //detonate mark
        champ.effects_stacks[EffectStackId::EzrealEssenceFluxMark] = 0;

        let w_lvl_idx: usize = usize::from(champ.w_lvl - 1); //to index ability ratios by lvl

        let magic_dmg: f32 = EZREAL_W_MARK_MAGIC_DMG_BY_W_LVL[w_lvl_idx]
            + champ.stats.bonus_ad
            + EZREAL_W_MARK_AP_RATIO_BY_W_LVL[w_lvl_idx] * champ.stats.ap();

        champ.dmg_on_target(
            target_stats,
            (0., EZREAL_W_HIT_PERCENT * magic_dmg, 0.),
            (1, 1),
            enum_set!(DmgTag::Ability),
            EZREAL_Q_HIT_PERCENT,
        )
    }
}

fn ezreal_w(champ: &mut Unit, _target_stats: &UnitStats) -> f32 {
    champ.effects_stacks[EffectStackId::EzrealEssenceFluxMark] = 1;
    champ.effects_values[EffectValueId::EzrealEssenceFluxHitTime] = champ.time;

    //add passive stack
    champ.add_temporary_effect(&EZREAL_RISING_SPELL_FORCE, 0.);

    0.
}

const EZREAL_E_MAGIC_DMG_BY_E_LVL: [f32; 5] = [80., 130., 180., 230., 280.];

fn ezreal_e(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    champ.sim_results.units_travelled += 475.; //blink range

    let w_mark_dmg: f32 = ezreal_detonate_w_mark_if_any(champ, target_stats);

    //add passive stack
    champ.add_temporary_effect(&EZREAL_RISING_SPELL_FORCE, 0.);

    let e_lvl_idx: usize = usize::from(champ.e_lvl - 1); //to index ability ratios by lvl

    let magic_dmg: f32 = EZREAL_E_MAGIC_DMG_BY_E_LVL[e_lvl_idx]
        + 0.5 * champ.stats.bonus_ad
        + 0.75 * champ.stats.ap();

    w_mark_dmg
        + champ.dmg_on_target(
            target_stats,
            (0., magic_dmg, 0.),
            (1, 1),
            enum_set!(DmgTag::Ability),
            1.,
        )
}

const EZREAL_R_MAGIC_DMG_BY_R_LVL: [f32; 3] = [350., 550., 750.];

fn ezreal_r(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let w_mark_dmg: f32 = ezreal_detonate_w_mark_if_any(champ, target_stats);

    //add passive stack
    champ.add_temporary_effect(&EZREAL_RISING_SPELL_FORCE, 0.);

    let r_lvl_idx: usize = usize::from(champ.r_lvl - 1); //to index ability ratios by lvl

    let magic_dmg: f32 = EZREAL_R_N_TARGETS
        * (EZREAL_R_MAGIC_DMG_BY_R_LVL[r_lvl_idx] + champ.stats.bonus_ad + 0.9 * champ.stats.ap());

    w_mark_dmg
        + champ.dmg_on_target(
            target_stats,
            (0., EZREAL_R_HIT_PERCENT * magic_dmg, 0.),
            (1, 1),
            enum_set!(DmgTag::Ability | DmgTag::Ultimate),
            EZREAL_R_N_TARGETS * EZREAL_R_HIT_PERCENT,
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

const EZREAL_DEFAULT_RUNES_PAGE: RunesPage = RunesPage {
    shard1: RuneShard::Left,
    shard2: RuneShard::Left,
    shard3: RuneShard::Left,
};

const EZREAL_DEFAULT_SKILL_ORDER: SkillOrder = SkillOrder {
    //lvls:
    //  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18
    q: [1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    e: [0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0],
    w: [0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1],
    r: [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0],
};

const EZREAL_DEFAULT_LEGENDARY_ITEMS: [&Item; 61] = [
    //&ABYSSAL_MASK,
    //&AXIOM_ARC,
    &BANSHEES_VEIL,
    &BLACK_CLEAVER,
    &BLACKFIRE_TORCH,
    &BLADE_OF_THE_RUINED_KING,
    &BLOODTHIRSTER,
    &CHEMPUNK_CHAINSWORD,
    &COSMIC_DRIVE,
    &CRYPTBLOOM,
    &DEAD_MANS_PLATE,
    &DEATHS_DANCE,
    &ECLIPSE,
    &EDGE_OF_NIGHT,
    &ESSENCE_REAVER,
    //&EXPERIMENTAL_HEXPLATE,
    &FROZEN_HEART,
    &GUARDIAN_ANGEL,
    &GUINSOOS_RAGEBLADE,
    //&HEXTECH_ROCKETBELT,
    &HORIZON_FOCUS,
    &HUBRIS,
    &HULLBREAKER,
    &ICEBORN_GAUNTLET,
    &IMMORTAL_SHIELDBOW,
    &INFINITY_EDGE,
    //&JAKSHO,
    //&KAENIC_ROOKERN,
    &KRAKEN_SLAYER,
    &LIANDRYS_TORMENT,
    &LICH_BANE,
    &LORD_DOMINIKS_REGARDS,
    &LUDENS_COMPANION,
    //&MALIGNANCE,
    &MAW_OF_MALMORTIUS,
    &MERCURIAL_SCIMITAR,
    &MORELLONOMICON,
    &MORTAL_REMINDER,
    &MURAMANA,
    &NASHORS_TOOTH,
    &NAVORI_FLICKERBLADE,
    &OPPORTUNITY,
    &OVERLORDS_BLOODMAIL,
    &PHANTOM_DANCER,
    //&PROFANE_HYDRA,
    &RABADONS_DEATHCAP,
    //&RANDUINS_OMEN,
    &RAPID_FIRECANNON,
    //&RAVENOUS_HYDRA,
    &RIFTMAKER,
    &ROD_OF_AGES,
    //&RUNAANS_HURRICANE, //passive works with q but only when in basic attack range, so doesn't synergise well
    &RYLAIS_CRYSTAL_SCEPTER,
    &SERAPHS_EMBRACE,
    &SERPENTS_FANG,
    &SERYLDAS_GRUDGE,
    &SHADOWFLAME,
    &SPEAR_OF_SHOJIN,
    &STATIKK_SHIV,
    &STERAKS_GAGE,
    &STORMSURGE,
    //&STRIDEBREAKER,
    &SUNDERED_SKY,
    &TERMINUS,
    &THE_COLLECTOR,
    &TITANIC_HYDRA,
    &TRINITY_FORCE,
    &UMBRAL_GLAIVE,
    &VOID_STAFF,
    &VOLTAIC_CYCLOSWORD,
    &WITS_END,
    &YOUMUUS_GHOSTBLADE,
    //&YUN_TAL_WILDARROWS, //q doesn't proc passive because cannot crit
    &ZHONYAS_HOURGLASS,
];

const EZREAL_DEFAULT_BOOTS: [&Item; 4] = [
    &BERSERKERS_GREAVES,
    &BOOTS_OF_SWIFTNESS,
    &IONIAN_BOOTS_OF_LUCIDITY,
    //&MERCURYS_TREADS,
    //&PLATED_STEELCAPS,
    &SORCERERS_SHOES,
];

const EZREAL_DEFAULT_SUPPORT_ITEMS: [&Item; 0] = [];

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
        on_lvl_set: None,
        init_abilities: Some(ezreal_init_abilities),
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
        unit_defaults: UnitDefaults {
            runes_pages: &EZREAL_DEFAULT_RUNES_PAGE,
            skill_order: &EZREAL_DEFAULT_SKILL_ORDER,
            legendary_items_pool: &EZREAL_DEFAULT_LEGENDARY_ITEMS,
            boots_pool: &EZREAL_DEFAULT_BOOTS,
            support_items_pool: &EZREAL_DEFAULT_SUPPORT_ITEMS,
        },
    };
}
