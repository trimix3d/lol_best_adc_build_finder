use crate::game_data::{items_data::items::*, units_data::*};

//champion parameters (constants):
const EZREAL_Q_HIT_PERCENT: f32 = 0.9;
const EZREAL_W_HIT_PERCENT: f32 = 0.8;
/// Number of targets hit by ezreal R.
const EZREAL_R_N_TARGETS: f32 = 1.;
const EZREAL_R_HIT_PERCENT: f32 = 0.8;

fn ezreal_init_spells(champ: &mut Unit) {
    champ.buffs_stacks[BuffStackId::EzrealWMark] = 0;
    champ.buffs_stacks[BuffStackId::EzrealRisingSpellForceStacks] = 0;
    champ.buffs_values[BuffValueId::EzrealRisingSpellForceBonusAS] = 0.;
}

fn ezreal_basic_attack(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let w_mark_dmg: f32 = if champ.buffs_stacks[BuffStackId::EzrealWMark] == 1 {
        ezreal_consume_w_mark(champ, target_stats)
    } else {
        0.
    };

    let ad_dmg: f32 = champ.stats.ad() * champ.stats.crit_coef();

    champ.dmg_on_target(
        target_stats,
        (ad_dmg, 0., 0.),
        (1, 1),
        DmgSource::Other,
        true,
        1.,
    ) + w_mark_dmg
}

const EZREAL_RISING_SPELL_FORCE_BONUS_AS_PER_STACK: f32 = 0.10;

fn ezreal_rising_spell_force_add_stack(champ: &mut Unit, _availability_coef: f32) {
    if champ.buffs_stacks[BuffStackId::EzrealRisingSpellForceStacks] < 5 {
        champ.buffs_stacks[BuffStackId::EzrealRisingSpellForceStacks] += 1;
        champ.stats.bonus_as += EZREAL_RISING_SPELL_FORCE_BONUS_AS_PER_STACK;
        champ.buffs_values[BuffValueId::EzrealRisingSpellForceBonusAS] +=
            EZREAL_RISING_SPELL_FORCE_BONUS_AS_PER_STACK;
    }
}

fn ezreal_rising_spell_force_disable(champ: &mut Unit) {
    champ.stats.bonus_as -= champ.buffs_values[BuffValueId::EzrealRisingSpellForceBonusAS];
    champ.buffs_values[BuffValueId::EzrealRisingSpellForceBonusAS] = 0.;
    champ.buffs_stacks[BuffStackId::EzrealRisingSpellForceStacks] = 0;
}

const EZREAL_RISING_SPELL_FORCE: TemporaryBuff = TemporaryBuff {
    id: BuffId::EzrealRisingSpellForce,
    add_stack: ezreal_rising_spell_force_add_stack,
    remove_every_stack: ezreal_rising_spell_force_disable,
    duration: 6.,
    cooldown: 0.,
};

const EZREAL_Q_BASE_DMG_BY_Q_LVL: [f32; 5] = [20., 45., 70., 95., 120.];
const EZREAL_Q_CD_REFUND: f32 = 1.5;

fn ezreal_q(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let w_mark_dmg: f32 = if champ.buffs_stacks[BuffStackId::EzrealWMark] == 1 {
        ezreal_consume_w_mark(champ, target_stats)
    } else {
        0.
    };

    let q_lvl_idx: usize = usize::from(champ.q_lvl - 1); //to index spell ratios by lvl

    let ad_dmg: f32 =
        EZREAL_Q_BASE_DMG_BY_Q_LVL[q_lvl_idx] + 1.30 * champ.stats.ad() + 0.15 * champ.stats.ap();

    //q hit reduces spells cooldown
    champ.q_cd = f32::max(0., champ.q_cd - EZREAL_Q_HIT_PERCENT * EZREAL_Q_CD_REFUND);
    champ.w_cd = f32::max(0., champ.w_cd - EZREAL_Q_HIT_PERCENT * EZREAL_Q_CD_REFUND);
    champ.e_cd = f32::max(0., champ.e_cd - EZREAL_Q_HIT_PERCENT * EZREAL_Q_CD_REFUND);
    champ.r_cd = f32::max(0., champ.r_cd - EZREAL_Q_HIT_PERCENT * EZREAL_Q_CD_REFUND);

    //add passive stack
    champ.add_temporary_buff(&EZREAL_RISING_SPELL_FORCE, 0.);

    champ.dmg_on_target(
        target_stats,
        (EZREAL_Q_HIT_PERCENT * ad_dmg, 0., 0.),
        (1, 1),
        DmgSource::BasicSpell,
        true,
        EZREAL_Q_HIT_PERCENT,
    ) + w_mark_dmg
}

const EZREAL_W_MARK_BASE_DMG_BY_W_LVL: [f32; 5] = [80., 135., 190., 245., 300.];
const EZREAL_W_MARK_AP_RATIO_BY_W_LVL: [f32; 5] = [0.70, 0.75, 0.80, 0.85, 0.90];

fn ezreal_consume_w_mark(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    champ.buffs_stacks[BuffStackId::EzrealWMark] = 0; //detonate mark

    let w_lvl_idx: usize = usize::from(champ.w_lvl - 1); //to index spell ratios by lvl

    let ap_dmg: f32 = EZREAL_W_MARK_BASE_DMG_BY_W_LVL[w_lvl_idx]
        + champ.stats.bonus_ad
        + EZREAL_W_MARK_AP_RATIO_BY_W_LVL[w_lvl_idx] * champ.stats.ap();

    champ.dmg_on_target(
        target_stats,
        (0., EZREAL_W_HIT_PERCENT * ap_dmg, 0.),
        (1, 1),
        DmgSource::BasicSpell,
        false,
        EZREAL_Q_HIT_PERCENT,
    )
}

fn ezreal_w(champ: &mut Unit, _target_stats: &UnitStats) -> f32 {
    champ.buffs_stacks[BuffStackId::EzrealWMark] = 1;

    //add passive stack
    champ.add_temporary_buff(&EZREAL_RISING_SPELL_FORCE, 0.);

    0.
}

const EZREAL_E_BASE_DMG_BY_E_LVL: [f32; 5] = [80., 130., 180., 230., 280.];

fn ezreal_e(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let w_mark_dmg: f32 = if champ.buffs_stacks[BuffStackId::EzrealWMark] == 1 {
        ezreal_consume_w_mark(champ, target_stats)
    } else {
        0.
    };

    champ.sim_results.units_travelled += 475.; //blink range

    //add passive stack
    champ.add_temporary_buff(&EZREAL_RISING_SPELL_FORCE, 0.);

    let e_lvl_idx: usize = usize::from(champ.e_lvl - 1); //to index spell ratios by lvl

    let ap_dmg: f32 = EZREAL_E_BASE_DMG_BY_E_LVL[e_lvl_idx]
        + 0.5 * champ.stats.bonus_ad
        + 0.75 * champ.stats.ap();

    champ.dmg_on_target(
        target_stats,
        (0., ap_dmg, 0.),
        (1, 1),
        DmgSource::BasicSpell,
        false,
        1.,
    ) + w_mark_dmg
}

const EZREAL_R_BASE_DMG_BY_R_LVL: [f32; 3] = [350., 550., 750.];

fn ezreal_r(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let w_mark_dmg: f32 = if champ.buffs_stacks[BuffStackId::EzrealWMark] == 1 {
        ezreal_consume_w_mark(champ, target_stats)
    } else {
        0.
    };

    //add passive stack
    champ.add_temporary_buff(&EZREAL_RISING_SPELL_FORCE, 0.);

    let r_lvl_idx: usize = usize::from(champ.r_lvl - 1); //to index spell ratios by lvl

    let ap_dmg: f32 = EZREAL_R_N_TARGETS
        * (EZREAL_R_BASE_DMG_BY_R_LVL[r_lvl_idx] + champ.stats.bonus_ad + 0.9 * champ.stats.ap());

    champ.dmg_on_target(
        target_stats,
        (0., EZREAL_R_HIT_PERCENT * ap_dmg, 0.),
        (1, 1),
        DmgSource::UltimateSpell,
        false,
        EZREAL_R_N_TARGETS * EZREAL_R_HIT_PERCENT,
    ) + w_mark_dmg
}

fn ezreal_fight_scenario_basic_attack_in_between_spells(
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
                    .min_by(|a, b| a.partial_cmp(b).expect("failed to compare floats"))
                    .unwrap(),
            );
        }
    }
    //add weighted r dmg at the end
    champ.weighted_r(target_stats);
}

fn ezreal_fight_scenario_only_spells(
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
                    .min_by(|a, b| a.partial_cmp(b).expect("failed to compare floats"))
                    .unwrap(),
            );
        }
    }
    //add weighted r dmg at the end
    champ.weighted_r(target_stats);
}

const EZREAL_DEFAULT_RUNES_PAGE: RunesPage = RunesPage {
    shard1: RuneShard::Left, //might still take attack speed for lane comfort
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
    //&RUNAANS_HURRICANE, //works with q but only when in basic attack range, so doesn't synergise well
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
    //&YUN_TAL_WILDARROWS, //passive doesn't proc q
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
            ap_coef: 0.,
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
        },
        growth_stats: UnitStats {
            hp: 102.,
            mana: 70.,
            base_ad: 2.75,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_coef: 0.,
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
        },
        on_lvl_set: None,
        init_spells: Some(ezreal_init_spells),
        basic_attack: ezreal_basic_attack,
        q: Spell {
            cast: ezreal_q,
            cast_time: 0.25,
            base_cooldown_by_spell_lvl: [5.5, 5.25, 5., 4.75, 4.5, F32_TOL], //basic spells only uses the first 5 values (except for aphelios)
        },
        w: Spell {
            cast: ezreal_w,
            cast_time: 0.25,
            base_cooldown_by_spell_lvl: [8., 8., 8., 8., 8., F32_TOL], //basic spells only uses the first 5 values (except for aphelios)
        },
        e: Spell {
            cast: ezreal_e,
            cast_time: 0.25,
            base_cooldown_by_spell_lvl: [26., 23., 20., 17., 14., F32_TOL], //basic spells only uses the first 5 values (except for aphelios)
        },
        r: Spell {
            cast: ezreal_r,
            cast_time: 1.,
            base_cooldown_by_spell_lvl: [120., 105., 90., F32_TOL, F32_TOL, F32_TOL], //ultimate only uses the first 3 values
        },
        fight_scenarios: &[
            (
                ezreal_fight_scenario_basic_attack_in_between_spells,
                "basic attack in between spells",
            ),
            (ezreal_fight_scenario_only_spells, "only launch spells"),
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