use crate::game_data::{
    basic_attack_aoe_effect_avg_additionnal_targets, items_data::items::*, units_data::*,
};

//champion parameters (constants):
const JINX_W_HIT_PERCENT: f32 = 0.8;
const JINX_R_TARGET_MISSING_HP_PERCENT: f32 = 0.66;
const JINX_R_AVG_TARGETS: f32 = 1.2;
const JINX_R_HIT_PERCENT: f32 = 0.9;

const JINX_ROCKET_LAUNCHER_AOE_RADIUS: f32 = 250.;
const JINX_ROCKET_LAUNCHER_AOE_AVG_TARGETS: f32 =
    basic_attack_aoe_effect_avg_additionnal_targets!(JINX_ROCKET_LAUNCHER_AOE_RADIUS);

/// Only rocket launcher.
fn jinx_basic_attack(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    //attack speed already slowed in jinx as ratio
    let ad_dmg: f32 = (1. + JINX_ROCKET_LAUNCHER_AOE_AVG_TARGETS)
        * 1.1
        * champ.stats.ad()
        * champ.stats.crit_coef();
    champ.dmg_on_target(
        target_stats,
        (ad_dmg, 0., 0.),
        (1, 1),
        DmgSource::Other,
        true,
        1., //rocket launcher aoe doesnt trigger on hit on additionnal targets
    )
}

fn jinx_q(_champ: &mut Unit, _target_stats: &UnitStats) -> f32 {
    //does nothing (only rocket launcher is implemented)
    //panic because pressing jinx q doesn't trigger spellblade in game while it currently does in this code, so it should be forbidden
    unreachable!("jinx q was used but the minigun is not implemented")
}

const JINX_W_BASE_DMG_BY_W_LVL: [f32; 5] = [10., 60., 110., 160., 210.];

fn jinx_w(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let w_lvl_idx: usize = usize::from(champ.w_lvl - 1); //to index spell ratios by lvl

    let ad_dmg: f32 = JINX_W_BASE_DMG_BY_W_LVL[w_lvl_idx] + 1.60 * champ.stats.ad();

    champ.dmg_on_target(
        target_stats,
        (JINX_W_HIT_PERCENT * ad_dmg, 0., 0.),
        (1, 1),
        DmgSource::BasicSpell,
        false,
        JINX_W_HIT_PERCENT,
    )
}

fn jinx_e(_champ: &mut Unit, _target_stats: &UnitStats) -> f32 {
    //not implemented (utility spell that roots, its dmg must not be considered, except if riot does silly things in a future patch)
    0.
}

const JINX_R_BASE_DMG_BY_R_LVL: [f32; 3] = [325., 475., 625.];
const JINX_R_MISSING_HP_RATIO_BY_R_LVL: [f32; 3] = [0.25, 0.30, 0.35];

fn jinx_r(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let r_lvl_idx: usize = usize::from(champ.r_lvl - 1); //to index spell ratios by lvl

    //assumes target is at 900 range
    let ad_dmg: f32 = (1. + 0.8 * (JINX_R_AVG_TARGETS - 1.))
        * (0.64 * (JINX_R_BASE_DMG_BY_R_LVL[r_lvl_idx] + 1.65 * champ.stats.bonus_ad)
            + (JINX_R_MISSING_HP_RATIO_BY_R_LVL[r_lvl_idx]
                * JINX_R_TARGET_MISSING_HP_PERCENT
                * target_stats.hp));

    champ.dmg_on_target(
        target_stats,
        (JINX_R_HIT_PERCENT * ad_dmg, 0., 0.),
        (1, 1),
        DmgSource::UltimateSpell,
        false,
        JINX_R_AVG_TARGETS * JINX_R_HIT_PERCENT,
    )
}

fn jinx_fight_scenario(champ: &mut Unit, target_stats: &UnitStats, fight_duration: f32) {
    //w once at the beggining
    champ.w(target_stats);

    champ.basic_attack(target_stats);
    while champ.time < fight_duration {
        //only basic attacks
        champ.walk(champ.basic_attack_cd + F32_TOL);
        champ.basic_attack(target_stats);
    }
    //add weighted r dmg at the end
    champ.weighted_r(target_stats);
}

const JINX_DEFAULT_RUNES_PAGE: RunesPage = RunesPage {
    shard1: RuneShard::Middle,
    shard2: RuneShard::Left,
    shard3: RuneShard::Left,
};

const JINX_DEFAULT_SKILL_ORDER: SkillOrder = SkillOrder {
    //lvls:
    //  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18
    q: [1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    w: [0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0],
    e: [0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1],
    r: [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0],
};

const JINX_DEFAULT_LEGENDARY_ITEMS: [&Item; 41] = [
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
    &GUINSOOS_RAGEBLADE,
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
    //&SPEAR_OF_SHOJIN,
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
];

const JINX_DEFAULT_BOOTS: [&Item; 2] = [
    &BERSERKERS_GREAVES,
    &BOOTS_OF_SWIFTNESS,
    //&IONIAN_BOOTS_OF_LUCIDITY,
    //&MERCURYS_TREADS,
    //&PLATED_STEELCAPS,
    //&SORCERERS_SHOES,
];

const JINX_DEFAULT_SUPPORT_ITEMS: [&Item; 0] = [];

const JINX_BASE_AS: f32 = 0.625;
impl Unit {
    pub const JINX_PROPERTIES: UnitProperties = UnitProperties {
        name: "Jinx",
        as_limit: Unit::DEFAULT_AS_LIMIT,
        as_ratio: 0.9 * JINX_BASE_AS, //rocket launcher scales with 90% of bonus as
        windup_percent: 0.16875,
        windup_modifier: 1., //get it from https://leagueoflegends.fandom.com/wiki/List_of_champions/Basic_attacks, 1 by default
        base_stats: UnitStats {
            hp: 630.,
            mana: 260.,
            base_ad: 59.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_coef: 0.,
            armor: 26.,
            mr: 30.,
            base_as: JINX_BASE_AS,
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
            hp: 105.,
            mana: 50.,
            base_ad: 2.9,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_coef: 0.,
            armor: 4.7,
            mr: 1.3,
            base_as: 0.,
            bonus_as: 0.01,
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
        init_spells: None,
        basic_attack: jinx_basic_attack,
        q: Spell {
            cast: jinx_q,
            cast_time: F32_TOL,
            base_cooldown_by_spell_lvl: [0.9, 0.9, 0.9, 0.9, 0.9, F32_TOL], //basic spells only uses the first 5 values (except for aphelios)
        },
        w: Spell {
            cast: jinx_w,
            cast_time: 0.55, //averaged value
            base_cooldown_by_spell_lvl: [8., 7., 6., 5., 4., F32_TOL], //basic spells only uses the first 5 values (except for aphelios)
        },
        e: Spell {
            cast: jinx_e,
            cast_time: F32_TOL,
            base_cooldown_by_spell_lvl: [24., 20.5, 17., 13.5, 10., F32_TOL], //basic spells only uses the first 5 values (except for aphelios)
        },
        r: Spell {
            cast: jinx_r,
            cast_time: 0.6,
            base_cooldown_by_spell_lvl: [85., 65., 45., F32_TOL, F32_TOL, F32_TOL], //ultimate only uses the first 3 values
        },
        fight_scenarios: &[(jinx_fight_scenario, "all out (with rocket launcher only)")],
        unit_defaults: UnitDefaults {
            runes_pages: &JINX_DEFAULT_RUNES_PAGE,
            skill_order: &JINX_DEFAULT_SKILL_ORDER,
            legendary_items_pool: &JINX_DEFAULT_LEGENDARY_ITEMS,
            boots_pool: &JINX_DEFAULT_BOOTS,
            support_items_pool: &JINX_DEFAULT_SUPPORT_ITEMS,
        },
    };
}
