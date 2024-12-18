use crate::game_data::*;

use items_data::Item;
use runes_data::*;
use units_data::*;

use enumset::enum_set;

//champion parameters (constants):
const Q_N_TARGETS: f32 = 1.0;
/// Percentage of the time the q return hit its targets.
const Q_RETURN_PERCENT: f32 = 0.67;
/// Number of targets hit by sivir ricochets (adds to the basic attack that launched the ricochet).
/// Must be less or equal to 9.
const W_N_RICOCHETS: f32 = 1.0;

fn sivir_init_abilities(champ: &mut Unit) {
    champ.effects_values[EffectValueId::SivirRicochetBonusAS] = 0.;
    champ.effects_values[EffectValueId::SivirFleetOfFootMsFlat] = 0.;
    champ.effects_values[EffectValueId::SivirOnTheHuntMsPercent] = 0.;
}

const FLEET_OF_FOOT_MS_FLAT_BY_LVL: [f32; MAX_UNIT_LVL] = [
    55., //lvl 1
    55., //lvl 2
    55., //lvl 3
    55., //lvl 4
    55., //lvl 5
    60., //lvl 6
    60., //lvl 7
    60., //lvl 8
    60., //lvl 9
    60., //lvl 10
    65., //lvl 11
    65., //lvl 12
    65., //lvl 13
    65., //lvl 14
    65., //lvl 15
    70., //lvl 16
    70., //lvl 17
    75., //lvl 18
];

fn sivir_fleet_of_foot_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_values[EffectValueId::SivirFleetOfFootMsFlat] == 0. {
        let ms_flat: f32 = 0.5 * FLEET_OF_FOOT_MS_FLAT_BY_LVL[usize::from(champ.lvl.get() - 1)]; //halved because decaying effect
        champ.stats.ms_flat += ms_flat;
        champ.effects_values[EffectValueId::SivirFleetOfFootMsFlat] = ms_flat;
    }
}

fn sivir_fleet_of_foot_disable(champ: &mut Unit) {
    champ.stats.ms_flat -= champ.effects_values[EffectValueId::SivirFleetOfFootMsFlat];
    champ.effects_values[EffectValueId::SivirFleetOfFootMsFlat] = 0.;
}

const SIVIR_FLEET_OF_FOOT: TemporaryEffect = TemporaryEffect {
    id: EffectId::SivirFleetOfFoot,
    add_stack: sivir_fleet_of_foot_enable,
    remove_every_stack: sivir_fleet_of_foot_disable,
    duration: 1.5,
    cooldown: 0.,
};

fn sivir_basic_attack(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    champ.add_temporary_effect(&SIVIR_FLEET_OF_FOOT, 0.);

    //if buffed by r, basic attacks reduces abilities cooldown
    if champ.effects_values[EffectValueId::SivirOnTheHuntMsPercent] != 0. {
        champ.q_cd = f32::max(0., champ.q_cd - R_ABILITIES_CD_REFUND_TIME);
        champ.w_cd = f32::max(0., champ.w_cd - R_ABILITIES_CD_REFUND_TIME);
        champ.e_cd = f32::max(0., champ.e_cd - R_ABILITIES_CD_REFUND_TIME);
    }

    //basic attack dmg
    let mut tot_dmg: PartDmg = units_data::default_basic_attack(champ, target_stats);

    //w ricochets dmg, instance of dmg must be done after basic attack
    if champ.effects_values[EffectValueId::SivirRicochetBonusAS] != 0. {
        let w_lvl_idx: usize = usize::from(champ.w_lvl - 1);
        let ricochet_phys_dmg: f32 = W_N_RICOCHETS
            * W_AD_RATIO_BY_W_LVL[w_lvl_idx]
            * champ.stats.ad()
            * champ.stats.crit_coef();

        tot_dmg += champ.dmg_on_target(
            target_stats,
            PartDmg(ricochet_phys_dmg, 0., 0.),
            (0, 0), //most abilities effects don't work with sivir ricochets (known exception: shojin), so putting 0 instances cancels their effects -> adapt items pool as a fail safe
            enum_set!(DmgTag::Ability), //abilities coef (shojin) will still run even with 0 instances
            1.,
        );
    }

    tot_dmg
}

const Q_PHYS_DMG_BY_Q_LVL: [f32; 5] = [60., 85., 110., 135., 160.];

fn sivir_q(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    champ.add_temporary_effect(&SIVIR_FLEET_OF_FOOT, 0.);

    let q_lvl_idx: usize = usize::from(champ.q_lvl - 1); //to index ability ratios by lvl

    let phys_dmg: f32 = Q_N_TARGETS
        * (1. + Q_RETURN_PERCENT)
        * (1. + 0.5 * champ.stats.crit_chance)
        * (Q_PHYS_DMG_BY_Q_LVL[q_lvl_idx] + champ.stats.bonus_ad + 0.6 * champ.stats.ap());

    champ.dmg_on_target(
        target_stats,
        PartDmg(phys_dmg, 0., 0.),
        (1 + (Q_RETURN_PERCENT as u8), 1),
        enum_set!(DmgTag::Ability),
        Q_N_TARGETS,
    )
}

const W_BONUS_AS_BY_W_LVL: [f32; 5] = [0.20, 0.25, 0.30, 0.35, 0.40];

fn sivir_ricochet_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_values[EffectValueId::SivirRicochetBonusAS] == 0. {
        let w_lvl_idx: usize = usize::from(champ.w_lvl - 1);
        let bonus_as: f32 = W_BONUS_AS_BY_W_LVL[w_lvl_idx];
        champ.stats.bonus_as += bonus_as;
        champ.effects_values[EffectValueId::SivirRicochetBonusAS] = bonus_as;
    }
}

fn sivir_ricochet_disable(champ: &mut Unit) {
    champ.stats.bonus_as -= champ.effects_values[EffectValueId::SivirRicochetBonusAS];
    champ.effects_values[EffectValueId::SivirRicochetBonusAS] = 0.;
}

const SIVIR_RICOCHET: TemporaryEffect = TemporaryEffect {
    id: EffectId::SivirRicochet,
    add_stack: sivir_ricochet_enable,
    remove_every_stack: sivir_ricochet_disable,
    duration: 4.,
    cooldown: 0.,
};

const W_AD_RATIO_BY_W_LVL: [f32; 5] = [0.30, 0.35, 0.40, 0.45, 0.50];

fn sivir_w(champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    champ.add_temporary_effect(&SIVIR_RICOCHET, 0.);

    //reset basic attack cd
    champ.basic_attack_cd = 0.;

    PartDmg(0., 0., 0.)
}

fn sivir_e(_champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    //does nothing (spellshield)
    PartDmg(0., 0., 0.)
}

//effect is weighted by r cooldown
fn sivir_on_the_hunt_lvl_1_enable(champ: &mut Unit, availability_coef: f32) {
    if champ.effects_values[EffectValueId::SivirOnTheHuntMsPercent] == 0. {
        let ms_percent: f32 = availability_coef * 0.20;
        champ.stats.ms_percent += ms_percent;
        champ.effects_values[EffectValueId::SivirOnTheHuntMsPercent] = ms_percent;
    }
}

fn sivir_on_the_hunt_lvl_2_enable(champ: &mut Unit, availability_coef: f32) {
    if champ.effects_values[EffectValueId::SivirOnTheHuntMsPercent] == 0. {
        let ms_percent: f32 = availability_coef * 0.25;
        champ.stats.ms_percent += ms_percent;
        champ.effects_values[EffectValueId::SivirOnTheHuntMsPercent] = ms_percent;
    }
}

fn sivir_on_the_hunt_lvl_3_enable(champ: &mut Unit, availability_coef: f32) {
    if champ.effects_values[EffectValueId::SivirOnTheHuntMsPercent] == 0. {
        let ms_percent: f32 = availability_coef * 0.30;
        champ.stats.ms_percent += ms_percent;
        champ.effects_values[EffectValueId::SivirOnTheHuntMsPercent] = ms_percent;
    }
}

fn sivir_on_the_hunt_disable(champ: &mut Unit) {
    champ.stats.ms_percent -= champ.effects_values[EffectValueId::SivirOnTheHuntMsPercent];
    champ.effects_values[EffectValueId::SivirOnTheHuntMsPercent] = 0.;
}

const SIVIR_ON_THE_HUNT_MS_LVL_1: TemporaryEffect = TemporaryEffect {
    id: EffectId::SivirOnTheHuntMS,
    add_stack: sivir_on_the_hunt_lvl_1_enable,
    remove_every_stack: sivir_on_the_hunt_disable,
    duration: 8.,
    cooldown: Unit::SIVIR_PROPERTIES.r.base_cooldown_by_ability_lvl[0],
};

const SIVIR_ON_THE_HUNT_MS_LVL_2: TemporaryEffect = TemporaryEffect {
    id: EffectId::SivirOnTheHuntMS,
    add_stack: sivir_on_the_hunt_lvl_2_enable,
    remove_every_stack: sivir_on_the_hunt_disable,
    duration: 10.,
    cooldown: Unit::SIVIR_PROPERTIES.r.base_cooldown_by_ability_lvl[1],
};

const SIVIR_ON_THE_HUNT_MS_LVL_3: TemporaryEffect = TemporaryEffect {
    id: EffectId::SivirOnTheHuntMS,
    add_stack: sivir_on_the_hunt_lvl_3_enable,
    remove_every_stack: sivir_on_the_hunt_disable,
    duration: 12.,
    cooldown: Unit::SIVIR_PROPERTIES.r.base_cooldown_by_ability_lvl[2],
};

/// Basic abilities cooldown refunded by each basic attack when under r effect
const R_ABILITIES_CD_REFUND_TIME: f32 = 0.5;

fn sivir_r(champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    match champ.r_lvl {
        1 => champ.add_temporary_effect(
            &SIVIR_ON_THE_HUNT_MS_LVL_1,
            champ.stats.ability_haste_ultimate(),
        ),
        2 => champ.add_temporary_effect(
            &SIVIR_ON_THE_HUNT_MS_LVL_2,
            champ.stats.ability_haste_ultimate(),
        ),
        3 => champ.add_temporary_effect(
            &SIVIR_ON_THE_HUNT_MS_LVL_3,
            champ.stats.ability_haste_ultimate(),
        ),
        _ => unreachable!(
            "{} R lvl is outside of range (using sivir R)",
            champ.properties.name
        ),
    };
    PartDmg(0., 0., 0.)
}

fn sivir_fight_scenario(champ: &mut Unit, target_stats: &UnitStats, fight_duration: f32) {
    //r at the beginning (effect is already weighted)
    champ.r(target_stats);

    while champ.time < fight_duration {
        //priority order: q, basic attack, w (w after basic attack so it performs basic attack reset)
        if champ.q_cd == 0. {
            champ.q(target_stats);
        } else if champ.basic_attack_cd == 0. {
            champ.basic_attack(target_stats);
        } else if champ.w_cd == 0. {
            champ.w(target_stats);
            //basic attack directly after
            //sivir w resets cooldown (patch 14.17) but we keep that here for consistency in case it gets changed
            if champ.basic_attack_cd != 0. {
                champ.walk(champ.basic_attack_cd + F32_TOL);
            }
            champ.basic_attack(target_stats);
        } else {
            champ.walk(
                F32_TOL
                    + [
                        champ.q_cd,
                        champ.basic_attack_cd,
                        champ.w_cd,
                        f32::max(0., fight_duration - champ.time),
                    ]
                    .into_iter()
                    .min_by(|a, b| a.partial_cmp(b).expect("Failed to compare floats"))
                    .unwrap(),
            );
        }
    }
}

const SIVIR_BASE_AS: f32 = 0.625;
impl Unit {
    pub const SIVIR_PROPERTIES: UnitProperties = UnitProperties {
        name: "Sivir",
        as_limit: Unit::DEFAULT_AS_LIMIT,
        as_ratio: SIVIR_BASE_AS,
        windup_percent: 0.12,
        windup_modifier: 1., //"mod" next to attack windup, 1 by default
        base_stats: UnitStats {
            hp: 600.,
            mana: 340.,
            base_ad: 60.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 30.,
            mr: 30.,
            base_as: SIVIR_BASE_AS,
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
            hp: 104.,
            mana: 45.,
            base_ad: 2.5,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 4.45,
            mr: 1.3,
            base_as: 0.,
            bonus_as: 0.02,
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
        basic_attack: sivir_basic_attack,
        q: BasicAbility {
            cast: sivir_q,
            cast_time: 0.175,
            base_cooldown_by_ability_lvl: [10., 9.5, 9., 8.5, 8., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        w: BasicAbility {
            cast: sivir_w,
            cast_time: F32_TOL,
            base_cooldown_by_ability_lvl: [12., 12., 12., 12., 12., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        e: BasicAbility {
            cast: sivir_e,
            cast_time: F32_TOL,
            base_cooldown_by_ability_lvl: [24., 22.5, 21., 19.5, 18., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        r: UltimateAbility {
            cast: sivir_r,
            cast_time: F32_TOL,
            base_cooldown_by_ability_lvl: [120., 100., 80.],
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(sivir_init_abilities),
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
        fight_scenarios: &[(sivir_fight_scenario, "all out")],
        defaults: UnitDefaults {
            runes_pages: RunesPage {
                keystone: &RuneKeystone::LETHAL_TEMPO, //todo: prone to change
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
                //&Item::MALIGNANCE, //cannot trigger passive
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
