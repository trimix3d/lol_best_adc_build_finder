use crate::game_data::units_data::*;

use items_data::items::*;

use enumset::enum_set;

//champion parameters (constants):
const SIVIR_Q_N_TARGETS: f32 = 1.0;
/// Percentage of the time the q return hit its targets.
const SIVIR_Q_RETURN_PERCENT: f32 = 2. / 3.;
/// Number of targets hit by sivir ricochets (adds to the basic attack that launched the ricochet).
const SIVIR_W_N_RICOCHETS: f32 = 1.0;

fn sivir_init_abilities(champ: &mut Unit) {
    champ.effects_values[EffectValueId::SivirRicochetBonusAS] = 0.;
    champ.effects_values[EffectValueId::SivirFleetOfFootMsFlat] = 0.;
    champ.effects_values[EffectValueId::SivirOnTheHuntMsPercent] = 0.;
}

const SIVIR_FLEET_OF_FOOT_MS_FLAT_BY_LVL: [f32; MAX_UNIT_LVL] = [
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
        let ms_flat: f32 =
            0.5 * SIVIR_FLEET_OF_FOOT_MS_FLAT_BY_LVL[usize::from(champ.lvl.get() - 1)]; //halved because decaying effect
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
        champ.q_cd = f32::max(0., champ.q_cd - SIVIR_R_ABILITIES_CD_REFUND_TIME);
        champ.w_cd = f32::max(0., champ.w_cd - SIVIR_R_ABILITIES_CD_REFUND_TIME);
        champ.e_cd = f32::max(0., champ.e_cd - SIVIR_R_ABILITIES_CD_REFUND_TIME);
    }

    let basic_attack_phys_dmg: f32 = champ.stats.ad() * champ.stats.crit_coef();

    //basic attack dmg, instance of dmg must be done before w ricochets
    let mut tot_dmg: PartDmg = champ.dmg_on_target(
        target_stats,
        PartDmg(basic_attack_phys_dmg, 0., 0.),
        (1, 1),
        enum_set!(DmgTag::BasicAttack),
        1.,
    );

    //w ricochets dmg, instance of dmg must be done after basic attack
    if champ.effects_values[EffectValueId::SivirRicochetBonusAS] != 0. {
        let w_lvl_idx: usize = usize::from(champ.w_lvl - 1);
        let ricochet_phys_dmg: f32 = SIVIR_W_N_RICOCHETS
            * SIVIR_W_AD_RATIO_BY_W_LVL[w_lvl_idx]
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

const SIVIR_Q_PHYS_DMG_BY_Q_LVL: [f32; 5] = [15., 30., 45., 60., 75.];
const SIVIR_Q_AD_RATIO_BY_Q_LVL: [f32; 5] = [0.80, 0.85, 0.90, 0.95, 1.0];

fn sivir_q(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    let q_lvl_idx: usize = usize::from(champ.q_lvl - 1); //to index ability ratios by lvl

    let phys_dmg: f32 = SIVIR_Q_N_TARGETS
        * (1. + SIVIR_Q_RETURN_PERCENT)
        * (1. + 0.5 * champ.stats.crit_chance)
        * (SIVIR_Q_PHYS_DMG_BY_Q_LVL[q_lvl_idx]
            + champ.stats.ad() * SIVIR_Q_AD_RATIO_BY_Q_LVL[q_lvl_idx]
            + 0.6 * champ.stats.ap());

    champ.dmg_on_target(
        target_stats,
        PartDmg(phys_dmg, 0., 0.),
        (1 + (SIVIR_Q_RETURN_PERCENT as u8), 1),
        enum_set!(DmgTag::Ability),
        SIVIR_Q_N_TARGETS,
    )
}

const SIVIR_W_BONUS_AS_BY_W_LVL: [f32; 5] = [0.20, 0.25, 0.30, 0.35, 0.40];

fn sivir_ricochet_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_values[EffectValueId::SivirRicochetBonusAS] == 0. {
        let w_lvl_idx: usize = usize::from(champ.w_lvl - 1);
        let bonus_as: f32 = SIVIR_W_BONUS_AS_BY_W_LVL[w_lvl_idx];
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

const SIVIR_W_AD_RATIO_BY_W_LVL: [f32; 5] = [0.30, 0.35, 0.40, 0.45, 0.50];

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
const SIVIR_R_ABILITIES_CD_REFUND_TIME: f32 = 0.5;

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
        windup_modifier: 1., //get it from https://leagueoflegends.fandom.com/wiki/List_of_champions/Basic_attacks, 1 by default
        base_stats: UnitStats {
            hp: 600.,
            mana: 340.,
            base_ad: 58.,
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
                q: [1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                w: [0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0],
                e: [0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1],
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
                //&MALIGNANCE, //cannot trigger passive
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
                &IONIAN_BOOTS_OF_LUCIDITY,
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
    pub fn test_sivir_constant_parameters() {
        assert!(
            SIVIR_W_N_RICOCHETS <= 9.,
            "Number of sivir's W ricochets must be less or equal to 9 (got {})",
            SIVIR_W_N_RICOCHETS
        )
    }
}
