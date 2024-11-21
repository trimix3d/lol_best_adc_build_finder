use crate::game_data::*;

use items_data::Item;
use runes_data::*;
use units_data::*;

use enumset::enum_set;

//champion parameters (constants):
const JINX_W_HIT_PERCENT: f32 = 0.75;
const JINX_R_TARGET_MISSING_HP_PERCENT: f32 = 0.67;
const JINX_R_AVG_TARGETS: f32 = 1.2;
const JINX_R_HIT_PERCENT: f32 = 0.85;

const JINX_ROCKET_LAUNCHER_AOE_RADIUS: f32 = 250.;
const JINX_ROCKET_LAUNCHER_AOE_AVG_TARGETS: f32 =
    basic_attack_aoe_effect_avg_additionnal_targets!(JINX_ROCKET_LAUNCHER_AOE_RADIUS);

/// Only rocket launcher.
fn jinx_basic_attack(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    //attack speed already slowed in jinx as ratio
    let phys_dmg: f32 = (1. + JINX_ROCKET_LAUNCHER_AOE_AVG_TARGETS)
        * 1.1
        * champ.stats.ad()
        * champ.stats.crit_coef();
    champ.dmg_on_target(
        target_stats,
        PartDmg(phys_dmg, 0., 0.),
        (1, 1),
        enum_set!(DmgTag::BasicAttack),
        1., //rocket launcher aoe doesnt trigger on hit on additionnal targets
    )
}

fn jinx_q(_champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    //does nothing (only rocket launcher is implemented)
    //panic because pressing jinx q doesn't trigger spellblade in game while it currently does in this code, so it should be forbidden
    unreachable!("Jinx q was used but the minigun is not implemented")
}

const JINX_W_PHYS_DMG_BY_W_LVL: [f32; 5] = [10., 60., 110., 160., 210.];

fn jinx_w(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    let w_lvl_idx: usize = usize::from(champ.w_lvl - 1); //to index ability ratios by lvl

    let phys_dmg: f32 = JINX_W_PHYS_DMG_BY_W_LVL[w_lvl_idx] + 1.40 * champ.stats.ad();

    champ.dmg_on_target(
        target_stats,
        PartDmg(JINX_W_HIT_PERCENT * phys_dmg, 0., 0.),
        (1, 1),
        enum_set!(DmgTag::Ability),
        1.,
    )
}

fn jinx_e(_champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    //not implemented (utility ability that roots, its dmg must not be considered, except if riot does silly things in a future patch)
    PartDmg(0., 0., 0.)
}

const JINX_R_PHYS_DMG_BY_R_LVL: [f32; 3] = [325., 475., 625.];
const JINX_R_MISSING_HP_RATIO_BY_R_LVL: [f32; 3] = [0.25, 0.30, 0.35];

fn jinx_r(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    let r_lvl_idx: usize = usize::from(champ.r_lvl - 1); //to index ability ratios by lvl

    //assumes target is at 900 range
    let phys_dmg: f32 = (1. + 0.8 * (JINX_R_AVG_TARGETS - 1.))
        * (0.64 * (JINX_R_PHYS_DMG_BY_R_LVL[r_lvl_idx] + 1.65 * champ.stats.bonus_ad)
            + (JINX_R_MISSING_HP_RATIO_BY_R_LVL[r_lvl_idx]
                * JINX_R_TARGET_MISSING_HP_PERCENT
                * target_stats.hp));

    champ.dmg_on_target(
        target_stats,
        PartDmg(JINX_R_HIT_PERCENT * phys_dmg, 0., 0.),
        (1, 1),
        enum_set!(DmgTag::Ability | DmgTag::Ultimate),
        JINX_R_AVG_TARGETS,
    )
}

fn jinx_fight_scenario(champ: &mut Unit, target_stats: &UnitStats, fight_duration: f32) {
    while champ.time < fight_duration {
        //priority order: w, basic attack
        if champ.w_cd == 0. {
            champ.w(target_stats);
        } else if champ.basic_attack_cd == 0. {
            champ.basic_attack(target_stats);
        } else {
            champ.walk(
                F32_TOL
                    + [
                        champ.w_cd,
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

fn jinx_fight_scenario_basic_attacks_only(
    champ: &mut Unit,
    target_stats: &UnitStats,
    fight_duration: f32,
) {
    //w once at the beginning
    champ.w(target_stats);

    while champ.time < fight_duration {
        //priority order: w, basic attack
        if champ.basic_attack_cd == 0. {
            champ.basic_attack(target_stats);
        } else {
            champ.walk(
                F32_TOL
                    + [
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
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        growth_stats: UnitStats {
            hp: 105.,
            mana: 50.,
            base_ad: 3.15,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 4.7,
            mr: 1.3,
            base_as: 0.,
            bonus_as: 0.014,
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
        basic_attack: jinx_basic_attack,
        q: BasicAbility {
            cast: jinx_q,
            cast_time: F32_TOL,
            base_cooldown_by_ability_lvl: [0.9, 0.9, 0.9, 0.9, 0.9, F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        w: BasicAbility {
            cast: jinx_w,
            cast_time: 0.55, //averaged value
            base_cooldown_by_ability_lvl: [8., 7., 6., 5., 4., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        e: BasicAbility {
            cast: jinx_e,
            cast_time: F32_TOL,
            base_cooldown_by_ability_lvl: [24., 20.5, 17., 13.5, 10., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        r: UltimateAbility {
            cast: jinx_r,
            cast_time: 0.6,
            base_cooldown_by_ability_lvl: [85., 65., 45.],
        },
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
        fight_scenarios: &[
            (jinx_fight_scenario, "all out (rocket launcher)"),
            (
                jinx_fight_scenario_basic_attacks_only,
                "basic attacks only (rocket launcher)",
            ),
        ],
        defaults: UnitDefaults {
            runes_pages: RunesPage {
                keystone: &RuneKeystone::PRESS_THE_ATTACK, //todo: prone to change
                shard1: RuneShard::Middle,
                shard2: RuneShard::Left,
                shard3: RuneShard::Left,
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
                //&Item::FROZEN_HEART,
                &Item::GUARDIAN_ANGEL,
                &Item::GUINSOOS_RAGEBLADE,
                //&Item::HEXTECH_ROCKETBELT,
                //&Item::HORIZON_FOCUS,
                &Item::HUBRIS,
                &Item::HULLBREAKER,
                //&Item::ICEBORN_GAUNTLET,
                &Item::IMMORTAL_SHIELDBOW,
                &Item::INFINITY_EDGE,
                //&Item::JAKSHO,
                //&Item::KAENIC_ROOKERN,
                &Item::KRAKEN_SLAYER,
                //&Item::LIANDRYS_TORMENT,
                //&Item::LICH_BANE,
                &Item::LORD_DOMINIKS_REGARDS,
                //&Item::LUDENS_COMPANION,
                //&Item::MALIGNANCE,
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
                //&Item::PROFANE_HYDRA,
                //&Item::RABADONS_DEATHCAP,
                //&Item::RANDUINS_OMEN,
                &Item::RAPID_FIRECANNON,
                //&Item::RAVENOUS_HYDRA,
                //&Item::RIFTMAKER,
                //&Item::ROD_OF_AGES,
                &Item::RUNAANS_HURRICANE,
                //&Item::RYLAIS_CRYSTAL_SCEPTER,
                //&Item::SERAPHS_EMBRACE,
                &Item::SERPENTS_FANG,
                &Item::SERYLDAS_GRUDGE,
                //&Item::SHADOWFLAME,
                //&Item::SPEAR_OF_SHOJIN,
                &Item::STATIKK_SHIV,
                &Item::STERAKS_GAGE,
                //&Item::STORMSURGE,
                //&Item::STRIDEBREAKER,
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
                //&Item::IONIAN_BOOTS_OF_LUCIDITY,
                //&Item::MERCURYS_TREADS,
                //&Item::PLATED_STEELCAPS,
                //&Item::SORCERERS_SHOES,
            ],
            supp_items_pool: &[],
        },
    };
}
