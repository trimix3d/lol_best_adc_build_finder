use crate::game_data::*;

use items_data::Item;
use runes_data::*;
use units_data::*;

use enumset::enum_set;

//champion parameters (constants):
/// Number of basic attacks to be performed before pressing w again, must be at least 1.
const BASIC_ATTACKS_PER_W: u8 = 2;
const R_N_TARGETS: f32 = 1.;
/// Percentage of the time the q return hit its targets.
const R_RETURN_PERCENT: f32 = 0.75;

fn draven_init_abilities(champ: &mut Unit) {
    champ.effects_stacks[EffectStackId::DravenAxesInHand] = 0;
    champ.effects_stacks[EffectStackId::DravenAxesInAir] = 0;
    champ.effects_values[EffectValueId::DravenBloodRushBonusAS] = 0.;
    champ.effects_values[EffectValueId::DravenBloodRushBonusMsPercent] = 0.;
}

const AXE_TIME_SPENT_IN_AIR: f32 = 2.; //axe travel time before hitting the target not accounted in this duration

fn draven_throw_axe(champ: &mut Unit, _availability_coef: f32) {
    champ.effects_stacks[EffectStackId::DravenAxesInHand] -= 1;
    champ.effects_stacks[EffectStackId::DravenAxesInAir] += 1;
}

fn draven_catch_axe(champ: &mut Unit) {
    champ.effects_stacks[EffectStackId::DravenAxesInAir] -= 1;
    champ.effects_stacks[EffectStackId::DravenAxesInHand] += 1;
    champ.w_cd = 0.; //catching axe resets w cd
}

//effect for axe n1
const DRAVEN_THROW_AXE1: TemporaryEffect = TemporaryEffect {
    id: EffectId::DravenThrowAxe1,
    add_stack: draven_throw_axe,
    remove_every_stack: draven_catch_axe, //effect assumes draven catches every axe
    duration: AXE_TIME_SPENT_IN_AIR,
    cooldown: 0.,
};

//effect for axe n2
const DRAVEN_THROW_AXE2: TemporaryEffect = TemporaryEffect {
    id: EffectId::DravenThrowAxe2,
    add_stack: draven_throw_axe,
    remove_every_stack: draven_catch_axe, //effect assumes draven catches every axe
    duration: AXE_TIME_SPENT_IN_AIR,
    cooldown: 0.,
};

const SPINNING_AXE_PHYS_DMG_BY_Q_LVL: [f32; 5] = [40., 45., 50., 55., 60.];
const SPINNING_AXE_BONUS_AD_RATIO_BY_Q_LVL: [f32; 5] = [0.75, 0.85, 0.95, 1.05, 1.15];

fn draven_q_axe_bonus_dmg(champ: &Unit) -> f32 {
    let q_lvl_idx: usize = usize::from(champ.q_lvl - 1);
    SPINNING_AXE_PHYS_DMG_BY_Q_LVL[q_lvl_idx]
        + champ.stats.bonus_ad * SPINNING_AXE_BONUS_AD_RATIO_BY_Q_LVL[q_lvl_idx]
}

fn draven_basic_attack(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    let mut phys_dmg: f32 = champ.stats.ad() * champ.stats.crit_coef();

    if champ.effects_stacks[EffectStackId::DravenAxesInHand] >= 1
        && champ.effects_stacks[EffectStackId::DravenAxesInAir] < 2
    {
        //this code only supports 2 axes in the air maximum, but its fine for most cases anyway
        if !champ
            .temporary_effects_durations
            .contains_key(&DRAVEN_THROW_AXE1)
        {
            champ.add_temporary_effect(&DRAVEN_THROW_AXE1, 0.);
            phys_dmg += draven_q_axe_bonus_dmg(champ);
        } else if !champ
            .temporary_effects_durations
            .contains_key(&DRAVEN_THROW_AXE2)
        {
            champ.add_temporary_effect(&DRAVEN_THROW_AXE2, 0.);
            phys_dmg += draven_q_axe_bonus_dmg(champ);
        }
    }

    champ.dmg_on_target(
        target_stats,
        PartDmg(phys_dmg, 0., 0.),
        (1, 1),
        enum_set!(DmgTag::BasicAttack),
        1.,
    )
}

fn draven_q(champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    champ.effects_stacks[EffectStackId::DravenAxesInHand] =
        u8::min(2, champ.effects_stacks[EffectStackId::DravenAxesInHand] + 1);
    PartDmg(0., 0., 0.)
}

const BLOOD_RUSH_BONUS_AS_BY_W_LVL: [f32; 5] = [0.20, 0.25, 0.30, 0.35, 0.40];
const BLOOD_RUSH_MS_PERCENT_BY_W_LVL: [f32; 5] =
    [0.50 / 2., 0.55 / 2., 0.60 / 2., 0.65 / 2., 0.70 / 2.]; //halved because decaying buff

fn draven_blood_rush_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_values[EffectValueId::DravenBloodRushBonusAS] == 0. {
        let w_lvl_idx: usize = usize::from(champ.w_lvl - 1);
        let bonus_as: f32 = BLOOD_RUSH_BONUS_AS_BY_W_LVL[w_lvl_idx];
        let ms_percent: f32 = BLOOD_RUSH_MS_PERCENT_BY_W_LVL[w_lvl_idx];
        champ.stats.bonus_as += bonus_as;
        champ.stats.ms_percent += ms_percent;
        champ.effects_values[EffectValueId::DravenBloodRushBonusAS] = bonus_as;
        champ.effects_values[EffectValueId::DravenBloodRushBonusMsPercent] = ms_percent;
    }
}

fn draven_blood_rush_disable(champ: &mut Unit) {
    champ.stats.bonus_as -= champ.effects_values[EffectValueId::DravenBloodRushBonusAS];
    champ.stats.ms_percent -= champ.effects_values[EffectValueId::DravenBloodRushBonusMsPercent];
    champ.effects_values[EffectValueId::DravenBloodRushBonusAS] = 0.;
    champ.effects_values[EffectValueId::DravenBloodRushBonusMsPercent] = 0.;
}

const DRAVEN_BLOOD_RUSH: TemporaryEffect = TemporaryEffect {
    id: EffectId::DravenBloodRush,
    add_stack: draven_blood_rush_enable,
    remove_every_stack: draven_blood_rush_disable,
    duration: 1.5,
    cooldown: 0.,
};

fn draven_w(champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    champ.add_temporary_effect(&DRAVEN_BLOOD_RUSH, 0.);
    PartDmg(0., 0., 0.)
}

const E_PHYS_DMG_BY_E_LVL: [f32; 5] = [75., 110., 145., 180., 215.];

fn draven_e(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    let e_lvl_idx: usize = usize::from(champ.e_lvl - 1); //to index ability ratios by lvl

    let phys_dmg: f32 = E_PHYS_DMG_BY_E_LVL[e_lvl_idx] + 0.5 * champ.stats.bonus_ad;

    champ.dmg_on_target(
        target_stats,
        PartDmg(phys_dmg, 0., 0.),
        (1, 1),
        enum_set!(DmgTag::Ability),
        1.,
    )
}

const R_PHYS_DMG_BY_R_LVL: [f32; 3] = [175., 275., 375.];
const R_BONUS_AD_RATIO_BY_R_LVL: [f32; 3] = [1.10, 1.30, 1.50];

fn draven_r(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    let r_lvl_idx: usize = usize::from(champ.r_lvl - 1); //to index ability ratios by lvl

    let phys_dmg: f32 = R_N_TARGETS
        * (1. + R_RETURN_PERCENT)
        * (R_PHYS_DMG_BY_R_LVL[r_lvl_idx]
            + champ.stats.bonus_ad * R_BONUS_AD_RATIO_BY_R_LVL[r_lvl_idx]);

    champ.dmg_on_target(
        target_stats,
        PartDmg(phys_dmg, 0., 0.),
        ((1. + R_RETURN_PERCENT) as u8, 1),
        enum_set!(DmgTag::Ability | DmgTag::Ultimate),
        R_N_TARGETS,
    )
}

fn draven_fight_scenario(champ: &mut Unit, target_stats: &UnitStats, fight_duration: f32) {
    //e once at the begginning
    champ.e(target_stats);

    let mut basic_attacks_count: u8 = BASIC_ATTACKS_PER_W - 1;
    while champ.time < fight_duration {
        //priority order: q before basic attacking if less than 2 axes in hand, basic attack if at least one axe and less than 2 axes in air, w every x basic attack
        if champ.basic_attack_cd == 0. && champ.effects_stacks[EffectStackId::DravenAxesInAir] < 2 {
            //q before launching basic attack if available
            if champ.q_cd == 0. && champ.effects_stacks[EffectStackId::DravenAxesInHand] < 2 {
                champ.q(target_stats);
            }
            champ.basic_attack(target_stats);
            basic_attacks_count += 1;
        } else if champ.w_cd == 0. && basic_attacks_count >= BASIC_ATTACKS_PER_W {
            champ.w(target_stats);
            basic_attacks_count = 0;
        } else {
            champ.walk(
                F32_TOL
                    + [
                        //cap minimum waiting time by TIME_BETWEEN_CLICKS if forced to wait for the axes in air to launch basic attack
                        //(waiting repeatedly by small time steps is faster than retrieving axes remaining in air duration)
                        if champ.effects_stacks[EffectStackId::DravenAxesInAir] < 2 {
                            champ.basic_attack_cd
                        } else {
                            f32::max(champ.basic_attack_cd, TIME_BETWEEN_CLICKS)
                        },
                        if basic_attacks_count >= BASIC_ATTACKS_PER_W {
                            champ.w_cd
                        } else {
                            fight_duration - champ.time
                        },
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

fn draven_fight_scenario_start_with_one_axe(
    champ: &mut Unit,
    target_stats: &UnitStats,
    fight_duration: f32,
) {
    champ.effects_stacks[EffectStackId::DravenAxesInHand] = 1;
    draven_fight_scenario(champ, target_stats, fight_duration);
}

const DRAVEN_BASE_AS: f32 = 0.679;
impl Unit {
    pub const DRAVEN_PROPERTIES: UnitProperties = UnitProperties {
        name: "Draven",
        as_limit: Unit::DEFAULT_AS_LIMIT,
        as_ratio: DRAVEN_BASE_AS, //if not specified, same as base AS
        windup_percent: 0.15614,
        windup_modifier: 1., //"mod" next to attack windup, 1 by default
        base_stats: UnitStats {
            hp: 675.,
            mana: 361.,
            base_ad: 62.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 29.,
            mr: 30.,
            base_as: DRAVEN_BASE_AS,
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
            hp: 104.,
            mana: 39.,
            base_ad: 3.6,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 4.5,
            mr: 1.3,
            base_as: 0.,
            bonus_as: 0.027,
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
        basic_attack: draven_basic_attack,
        q: BasicAbility {
            cast: draven_q,
            cast_time: F32_TOL,
            base_cooldown_by_ability_lvl: [12., 11., 10., 9., 8., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        w: BasicAbility {
            cast: draven_w,
            cast_time: F32_TOL,
            base_cooldown_by_ability_lvl: [12., 12., 12., 12., 12., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        e: BasicAbility {
            cast: draven_e,
            cast_time: 0.25,
            base_cooldown_by_ability_lvl: [18., 17., 16., 15., 14., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        r: UltimateAbility {
            cast: draven_r,
            cast_time: 0.5,
            base_cooldown_by_ability_lvl: [100., 90., 80.],
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(draven_init_abilities),
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
            (draven_fight_scenario_start_with_one_axe, "start with 1 axe"),
            (draven_fight_scenario, "start with no axe"),
        ],
        defaults: UnitDefaults {
            runes_pages: RunesPage {
                keystone: &RuneKeystone::PRESS_THE_ATTACK, //todo: prone to change
                shard1: RuneShard::Left,
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
                //&Item::GUINSOOS_RAGEBLADE,
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
                //&Item::RUNAANS_HURRICANE,
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
