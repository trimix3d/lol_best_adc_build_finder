use crate::game_data::*;

use items_data::Item;
use runes_data::*;
use units_data::*;

use enumset::enum_set;

//champion parameters (constants):
/// 1 proc = 2 basic attacks (gains 1 proc per activation).
const N_VIGILANCE_PROCS: u8 = 1;
/// Percentage of the r that hits its target, must be between 0. and 1.
const R_HIT_PERCENT: f32 = 0.75;

fn lucian_init_abilities(champ: &mut Unit) {
    champ.effects_stacks[EffectStackId::LucianLightslingerEmpowered] = 0;
    champ.effects_stacks[EffectStackId::LucianVigilanceProcsRemaning] = N_VIGILANCE_PROCS;
    champ.effects_values[EffectValueId::LucianArdentBlazeMsFlat] = 0.;
}

const LIGHTSLINGER_BASIC_ATTACKS_AD_RATIO_BY_LVL: [f32; MAX_UNIT_LVL] = [
    0.50, //lvl 1
    0.50, //lvl 2
    0.50, //lvl 3
    0.50, //lvl 4
    0.50, //lvl 5
    0.50, //lvl 6
    0.55, //lvl 7
    0.55, //lvl 8
    0.55, //lvl 9
    0.55, //lvl 10
    0.55, //lvl 11
    0.55, //lvl 12
    0.60, //lvl 13
    0.60, //lvl 14
    0.60, //lvl 15
    0.60, //lvl 16
    0.60, //lvl 17
    0.60, //lvl 18
];

fn lucian_basic_attack(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    if champ.effects_stacks[EffectStackId::LucianLightslingerEmpowered] == 1 {
        champ.effects_stacks[EffectStackId::LucianLightslingerEmpowered] = 0;
        champ.e_cd = f32::max(0., champ.e_cd - 4.); //double basic attack reduce e_cd by 2sec for each hit

        //vigilance passive
        let vigilance_dmg: f32 =
            if champ.effects_stacks[EffectStackId::LucianVigilanceProcsRemaning] != 0 {
                champ.effects_stacks[EffectStackId::LucianVigilanceProcsRemaning] -= 1;
                15. + 0.20 * champ.stats.ad()
            } else {
                0.
            };
        let basic_attack_phys_dmg: f32 = champ.stats.ad() * champ.stats.crit_coef();

        let first_hit: PartDmg = champ.dmg_on_target(
            target_stats,
            PartDmg(basic_attack_phys_dmg, vigilance_dmg, 0.),
            (1, 1),
            enum_set!(DmgTag::BasicAttack),
            1.,
        );
        champ.all_on_basic_attack_cast();
        first_hit
            + champ.dmg_on_target(
                target_stats,
                PartDmg(
                    basic_attack_phys_dmg
                        * LIGHTSLINGER_BASIC_ATTACKS_AD_RATIO_BY_LVL
                            [usize::from(champ.lvl.get() - 1)],
                    vigilance_dmg,
                    0.,
                ),
                (1, 1),
                enum_set!(DmgTag::BasicAttack),
                1.,
            )
    } else {
        units_data::default_basic_attack(champ, target_stats)
    }
}

const Q_PHYS_DMG_BY_Q_LVL: [f32; 5] = [85., 115., 145., 175., 205.];
const Q_BONUS_AD_RATIO_BY_Q_LVL: [f32; 5] = [0.60, 0.75, 0.90, 1.05, 1.20];
fn lucian_q(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    let q_lvl_idx: usize = usize::from(champ.q_lvl - 1); //to index ability ratios by lvl
    let phys_dmg: f32 = Q_PHYS_DMG_BY_Q_LVL[q_lvl_idx]
        + Q_BONUS_AD_RATIO_BY_Q_LVL[q_lvl_idx] * champ.stats.bonus_ad;

    champ.effects_stacks[EffectStackId::LucianLightslingerEmpowered] = 1;

    champ.dmg_on_target(
        target_stats,
        PartDmg(phys_dmg, 0., 0.),
        (1, 1),
        enum_set!(DmgTag::Ability),
        1.,
    )
}

const ARDENT_BLAZE_MS_BY_W_LVL: [f32; 5] = [60., 65., 70., 75., 80.];

fn lucian_ardent_blaze_ms_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_values[EffectValueId::LucianArdentBlazeMsFlat] == 0. {
        let flat_ms_buff: f32 = ARDENT_BLAZE_MS_BY_W_LVL[usize::from(champ.w_lvl - 1)];
        champ.stats.ms_flat += flat_ms_buff;
        champ.effects_values[EffectValueId::LucianArdentBlazeMsFlat] = flat_ms_buff;
    }
}

fn lucian_ardent_blaze_ms_disable(champ: &mut Unit) {
    champ.stats.ms_flat -= champ.effects_values[EffectValueId::LucianArdentBlazeMsFlat];
    champ.effects_values[EffectValueId::LucianArdentBlazeMsFlat] = 0.;
}

const LUCIAN_ARDENT_BLAZE_MS: TemporaryEffect = TemporaryEffect {
    id: EffectId::LucianArdentBlazeMS,
    add_stack: lucian_ardent_blaze_ms_enable,
    remove_every_stack: lucian_ardent_blaze_ms_disable,
    duration: 1.,
    cooldown: 0.,
};

const W_MAGIC_DMG_BY_W_LVL: [f32; 5] = [75., 110., 145., 180., 215.];
fn lucian_w(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    let w_lvl_idx: usize = usize::from(champ.w_lvl - 1); //to index ability ratios by lvl
    let magic_dmg: f32 = W_MAGIC_DMG_BY_W_LVL[w_lvl_idx] + 0.9 * champ.stats.ap();

    champ.effects_stacks[EffectStackId::LucianLightslingerEmpowered] = 1;

    //for simplicity we apply the ms buff directly
    champ.add_temporary_effect(&LUCIAN_ARDENT_BLAZE_MS, 0.);

    champ.dmg_on_target(
        target_stats,
        PartDmg(0., magic_dmg, 0.),
        (1, 1),
        enum_set!(DmgTag::Ability),
        1.,
    )
}

fn lucian_e(champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    champ.units_travelled += 425.; //maximum dash range
    champ.effects_stacks[EffectStackId::LucianLightslingerEmpowered] = 1;
    PartDmg(0., 0., 0.)
}

const R_PHYS_DMG_BY_R_LVL: [f32; 3] = [15., 30., 45.]; //dmg on champions
fn lucian_r(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    let r_lvl_idx: usize = usize::from(champ.r_lvl - 1); //to index ability ratios by lvl

    let n_hits: f32 = R_HIT_PERCENT * (22. + champ.stats.crit_chance / 0.04);
    let phys_dmg: f32 = n_hits
        * (R_PHYS_DMG_BY_R_LVL[r_lvl_idx] + 0.25 * champ.stats.ad() + 0.15 * champ.stats.ap());

    champ.effects_stacks[EffectStackId::LucianLightslingerEmpowered] = 1;

    let r_dmg: PartDmg = champ.dmg_on_target(
        target_stats,
        PartDmg(phys_dmg, 0., 0.),
        (n_hits as u8, 1),
        enum_set!(DmgTag::Ability | DmgTag::Ultimate),
        1.,
    );
    champ.walk(R_HIT_PERCENT * 3.);
    r_dmg
}

fn lucian_fight_scenario_all_out(champ: &mut Unit, target_stats: &UnitStats, fight_duration: f32) {
    while champ.time < fight_duration {
        //priority order: empowered basic attack, e, q, w, unempowered basic attack
        if champ.effects_stacks[EffectStackId::LucianLightslingerEmpowered] == 1 {
            //wait for the basic attack cooldown if there is one
            if champ.basic_attack_cd != 0. {
                champ.walk(champ.basic_attack_cd + F32_TOL);
            }
            champ.basic_attack(target_stats);
        } else if champ.e_cd == 0. {
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
                        champ.e_cd,
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

fn lucian_fight_scenario_poke(champ: &mut Unit, target_stats: &UnitStats, fight_duration: f32) {
    while champ.time < fight_duration {
        //priority order: empowered basic attack, e, q, w (no unempowered basic attack)
        if champ.effects_stacks[EffectStackId::LucianLightslingerEmpowered] == 1 {
            //wait for the basic basic_attack cooldown if there is one
            if champ.basic_attack_cd != 0. {
                champ.walk(champ.basic_attack_cd + F32_TOL);
            }
            champ.basic_attack(target_stats);
        } else if champ.e_cd == 0. {
            champ.e(target_stats);
        } else if champ.q_cd == 0. {
            champ.q(target_stats);
        } else if champ.w_cd == 0. {
            champ.w(target_stats);
        } else {
            champ.walk(
                F32_TOL
                    + [
                        champ.q_cd,
                        champ.w_cd,
                        champ.e_cd,
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

const LUCIAN_BASE_AS: f32 = 0.638;
impl Unit {
    pub const LUCIAN_PROPERTIES: UnitProperties = UnitProperties {
        name: "Lucian",
        as_limit: Unit::DEFAULT_AS_LIMIT,
        as_ratio: LUCIAN_BASE_AS, //if not specified, same as base AS
        windup_percent: 0.15,
        windup_modifier: 1., //"mod" next to attack windup, 1 by default
        base_stats: UnitStats {
            hp: 641.,
            mana: 320.,
            base_ad: 60.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 28.,
            mr: 30.,
            base_as: LUCIAN_BASE_AS,
            bonus_as: 0.,
            ability_haste: 0.,
            basic_haste: 0.,
            ultimate_haste: 0.,
            item_haste: 0.,
            crit_chance: 0.,
            crit_dmg: Unit::BASE_CRIT_DMG,
            ms_flat: 335.,
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
            hp: 100.,
            mana: 43.,
            base_ad: 2.9,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 4.2,
            mr: 1.3,
            base_as: 0.,
            bonus_as: 0.033,
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
        basic_attack: lucian_basic_attack,
        q: BasicAbility {
            cast: lucian_q,
            cast_time: 0.33, //average between 0.4 and 0,25
            base_cooldown_by_ability_lvl: [9., 8., 7., 6., 5., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        w: BasicAbility {
            cast: lucian_w,
            cast_time: 0.25,
            base_cooldown_by_ability_lvl: [14., 13., 12., 11., 10., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        e: BasicAbility {
            cast: lucian_e,
            cast_time: F32_TOL,
            base_cooldown_by_ability_lvl: [19., 17.75, 16.5, 15.25, 14., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        r: UltimateAbility {
            cast: lucian_r,
            cast_time: F32_TOL,
            base_cooldown_by_ability_lvl: [110., 100., 90.],
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(lucian_init_abilities),
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
            (lucian_fight_scenario_all_out, "all out"),
            (
                lucian_fight_scenario_poke,
                "all out but basic attack only when empowered after an ability (~= poke)",
            ),
        ],
        defaults: UnitDefaults {
            runes_pages: RunesPage {
                keystone: &RuneKeystone::PRESS_THE_ATTACK, //conq better for long fights, PTA better for poke (14.20) //todo: prone to change
                shard1: RuneShard::Middle,
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
                //&Item::BANSHEES_VEIL,
                &Item::BLACK_CLEAVER,
                //&Item::BLACKFIRE_TORCH,
                &Item::BLADE_OF_THE_RUINED_KING,
                &Item::BLOODTHIRSTER,
                &Item::CHEMPUNK_CHAINSWORD,
                //&Item::COSMIC_DRIVE,
                //&Item::CRYPTBLOOM,
                //&Item::DEAD_MANS_PLATE, //passive handles dashes incorrectly
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
                &Item::ICEBORN_GAUNTLET,
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
                &Item::SPEAR_OF_SHOJIN,
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
                &Item::IONIAN_BOOTS_OF_LUCIDITY,
                //&Item::MERCURYS_TREADS,
                //&Item::PLATED_STEELCAPS,
                //&Item::SORCERERS_SHOES,
            ],
            supp_items_pool: &[],
        },
    };
}
