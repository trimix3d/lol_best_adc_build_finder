use crate::game_data::*;

use items_data::Item;
use runes_data::*;
use units_data::*;

use enumset::enum_set;

//champion parameters (constants):
/// Percentage of target missing hp when second skin 5 stacks procs.
/// The missing hp taken for the calculation is the value AFTER base dmg from the basic attack/w hits,
const SECOND_SKIN_TARGET_MISSING_HP_PERCENT: f32 = 0.5;
const W_HIT_PERCENT: f32 = 0.85;

fn kaisa_init_abilities(champ: &mut Unit) {
    champ.effects_stacks[EffectStackId::KaisaSecondSkinStacks] = 0;
    champ.effects_values[EffectValueId::KaisaSecondSkinLastStackTime] =
        -(SECOND_SKIN_DELAY + F32_TOL); //to allow for effect at time == 0
    champ.effects_values[EffectValueId::KaisaSuperchargeBonusAS] = 0.;

    //Q evolve, items bonus ad + base_ad from lvls must be >= 100
    if champ.items_stats.bonus_ad + champ.lvl_stats.base_ad - champ.properties.base_stats.base_ad
        >= 100.
    {
        champ.effects_stacks[EffectStackId::KaisaQEvolved] = 1;
    } else {
        champ.effects_stacks[EffectStackId::KaisaQEvolved] = 0;
    }

    //W evolve, items ap must be >= 100
    if champ.items_stats.ap() >= 100. {
        champ.effects_stacks[EffectStackId::KaisaWEvolved] = 1;
    } else {
        champ.effects_stacks[EffectStackId::KaisaWEvolved] = 0;
    }

    //E evolve, items bonus_as + bonus_as from lvls must be >= 100% (invisibility not implemented)
    //if champ.items_stats.bonus_as + champ.lvl_stats.bonus_as - champ.properties.base_stats.bonus_as
    //    >= 1.
    //{
    //    champ.effects_stacks[EffectStackId::KaisaEEvolved] = 1;
    //} else {
    //    champ.effects_stacks[EffectStackId::KaisaEEvolved] = 0;
    //}
}

const SECOND_SKIN_BASE_MAGIC_DMG_BY_LVL: [f32; MAX_UNIT_LVL] = [
    4.,    //lvl 1
    5.18,  //lvl 2
    6.35,  //lvl 3
    7.53,  //lvl 4
    8.71,  //lvl 5
    9.88,  //lvl 6
    11.06, //lvl 7
    12.24, //lvl 8
    13.41, //lvl 9
    14.59, //lvl 10
    15.76, //lvl 11
    16.94, //lvl 12
    18.12, //lvl 13
    19.29, //lvl 14
    20.47, //lvl 15
    21.65, //lvl 16
    22.82, //lvl 17
    24.,   //lvl 18
];

const SECOND_SKIN_MAGIC_DMG_PER_STACK_BY_LVL: [f32; MAX_UNIT_LVL] = [
    1.,   //lvl 1
    1.29, //lvl 2
    1.59, //lvl 3
    1.88, //lvl 4
    2.18, //lvl 5
    2.47, //lvl 6
    2.76, //lvl 7
    3.06, //lvl 8
    3.35, //lvl 9
    3.65, //lvl 10
    3.94, //lvl 11
    4.24, //lvl 12
    4.53, //lvl 13
    4.82, //lvl 14
    5.12, //lvl 15
    5.41, //lvl 16
    5.71, //lvl 17
    6.,   //lvl 18
];

const SECOND_SKIN_MAX_STACKS: u8 = 5;
const SECOND_SKIN_AP_COEF_BY_STACK: [f32; SECOND_SKIN_MAX_STACKS as usize] =
    [0.12, 0.15, 0.18, 0.21, 0.24];

const SECOND_SKIN_DELAY: f32 = 4.; //stack duration
fn kaisa_second_skin(
    champ: &mut Unit,
    target_stats: &UnitStats,
    n_targets: f32,
    _from_other_effects: bool,
) -> PartDmg {
    //if last hit from too long ago, reset stacks
    if champ.time - champ.effects_values[EffectValueId::KaisaSecondSkinLastStackTime]
        >= SECOND_SKIN_DELAY
    {
        champ.effects_stacks[EffectStackId::KaisaSecondSkinStacks] = 0;
    }

    let lvl_idx: usize = usize::from(champ.lvl.get() - 1);

    //calculate dmg (before increasing stacks count)
    let mut magic_dmg: f32 = n_targets
        * (SECOND_SKIN_BASE_MAGIC_DMG_BY_LVL[lvl_idx]
            + f32::from(champ.effects_stacks[EffectStackId::KaisaSecondSkinStacks])
                * SECOND_SKIN_MAGIC_DMG_PER_STACK_BY_LVL[lvl_idx]
            + champ.stats.ap()
                * SECOND_SKIN_AP_COEF_BY_STACK
                    [usize::from(champ.effects_stacks[EffectStackId::KaisaSecondSkinStacks])]);

    //update stack count
    if champ.effects_stacks[EffectStackId::KaisaSecondSkinStacks] < SECOND_SKIN_MAX_STACKS - 1 {
        champ.effects_stacks[EffectStackId::KaisaSecondSkinStacks] += 1;
        champ.effects_values[EffectValueId::KaisaSecondSkinLastStackTime] = champ.time;
    } else {
        champ.effects_stacks[EffectStackId::KaisaSecondSkinStacks] = 0;

        //Second skin missing hp dmg not affected by n_targets because runaans bolts don't apply guinsoos bonus phantom hit,
        //so they shouldn't benefit from phantom hit bonus stack.
        //This is effectively a "nerf" to runaans compared to in game behavior because bolts actually apply the missing hp passive
        //(but without guinsoos phantom hit extra stack generation), whereas here it's considered as if they didn't at all.
        //(the idea is that no item should be artificially "buffed", but they can be "nerfed" so when it's picked by the optimiser, it's guaranteed to be good).
        //With the current dmg system I basically have to choose which between guinsoos and runaans work with the second skin passive.
        //I chose guinsoos because its interaction with the second skin passive seems more important than runaan's.
        magic_dmg += SECOND_SKIN_TARGET_MISSING_HP_PERCENT
            * target_stats.hp
            * (0.15 + 0.06 / 100. * champ.stats.ap());
    }

    PartDmg(0., magic_dmg, 0.)
}

fn kaisa_on_basic_attack_cast(champ: &mut Unit) {
    //basic attack reduce e cd by 0.5 sec
    champ.e_cd = f32::max(0., champ.e_cd - 0.5);
}

/// Assumes single target dmg.
const Q_PHYS_DMG_BY_Q_LVL: [f32; 5] = [90., 123.75, 157.5, 191.25, 225.];
/// Assumes single target dmg.
const EVOLVED_Q_PHYS_DMG_BY_Q_LVL: [f32; 5] = [150., 206.25, 262.5, 318.75, 375.];

fn kaisa_q(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    let q_lvl_idx: usize = usize::from(champ.q_lvl - 1); //to index ability ratios by lvl

    let n_missiles: u8;
    let phys_dmg: f32;
    //missiles on the same target do reduced dmg beyond the first
    if champ.effects_stacks[EffectStackId::KaisaQEvolved] == 0 {
        //not evolved
        n_missiles = 6;
        phys_dmg = Q_PHYS_DMG_BY_Q_LVL[q_lvl_idx]
            + 1.2375 * champ.stats.bonus_ad
            + 0.45 * champ.stats.ap();
    //assumes single target dmg
    } else {
        //evolved
        n_missiles = 12;
        phys_dmg = EVOLVED_Q_PHYS_DMG_BY_Q_LVL[q_lvl_idx]
            + 2.0625 * champ.stats.bonus_ad
            + 0.75 * champ.stats.ap(); //assumes single target dmg
    };

    champ.dmg_on_target(
        target_stats,
        PartDmg(phys_dmg, 0., 0.),
        (n_missiles, 1),
        enum_set!(DmgTag::Ability),
        1.,
    )
}

/// Used to calculate the average travel time of the projectile.
const W_PROJECTILE_SPEED: f32 = 1750.;
/// Has an impact on evolved w cd refund (greater travel time -> cd refund is less relevant).
const W_TRAVEL_TIME: f32 = 1000. / W_PROJECTILE_SPEED;

const W_MAGIC_DMG_BY_W_LVL: [f32; 5] = [30., 55., 80., 105., 130.];

fn kaisa_w(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    let w_lvl_idx: usize = usize::from(champ.w_lvl - 1); //to index ability ratios by lvl

    let magic_dmg: f32 =
        W_MAGIC_DMG_BY_W_LVL[w_lvl_idx] + 1.3 * champ.stats.ad() + 0.45 * champ.stats.ap();
    let mut second_skin_dmg: PartDmg = kaisa_second_skin(champ, target_stats, 1., false)
        + kaisa_second_skin(champ, target_stats, 1., false); //applies proc one by one

    if champ.effects_stacks[EffectStackId::KaisaWEvolved] == 1 {
        //if evolved
        champ.w_cd -= W_HIT_PERCENT * 0.75 * f32::max(0., champ.w_cd - W_TRAVEL_TIME); //account for w travel time (otherwise cd is instantly refunded after casting and that can be op)
        second_skin_dmg += kaisa_second_skin(champ, target_stats, 1., false);
    }

    champ.dmg_on_target(
        target_stats,
        W_HIT_PERCENT * (PartDmg(0., magic_dmg, 0.) + second_skin_dmg),
        (1, 1),
        enum_set!(DmgTag::Ability),
        1.,
    )
}

const E_MS_PERCENT_BY_E_LVL: [f32; 5] = [0.55, 0.60, 0.65, 0.70, 0.75];

const E_BONUS_AS_BY_E_LVL: [f32; 5] = [0.40, 0.50, 0.60, 0.70, 0.80];

fn kaisa_supercharge_as_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_values[EffectValueId::KaisaSuperchargeBonusAS] == 0. {
        let bonus_as_buff: f32 = E_BONUS_AS_BY_E_LVL[usize::from(champ.e_lvl - 1)];
        champ.stats.bonus_as += bonus_as_buff;
        champ.effects_values[EffectValueId::KaisaSuperchargeBonusAS] = bonus_as_buff;
    }
}

fn kaisa_supercharge_as_disable(champ: &mut Unit) {
    champ.stats.bonus_as -= champ.effects_values[EffectValueId::KaisaSuperchargeBonusAS];
    champ.effects_values[EffectValueId::KaisaSuperchargeBonusAS] = 0.;
}

const KAISA_SUPERCHARGE_AS: TemporaryEffect = TemporaryEffect {
    id: EffectId::KaisaSuperchargeAS,
    add_stack: kaisa_supercharge_as_enable,
    remove_every_stack: kaisa_supercharge_as_disable,
    duration: 4.,
    cooldown: 0.,
};

fn kaisa_e(champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    let e_lvl_idx: usize = usize::from(champ.e_lvl - 1); //to index ability ratios by lvl

    let capped_bonus_as: f32 = f32::min(
        1.,
        champ.items_stats.bonus_as + champ.lvl_stats.bonus_as
            - champ.properties.base_stats.bonus_as,
    );
    let percent_ms_buff: f32 = E_MS_PERCENT_BY_E_LVL[e_lvl_idx] * (1. + capped_bonus_as);
    champ.stats.ms_percent += percent_ms_buff;
    champ.walk(1.2 / (1. + capped_bonus_as));
    champ.stats.ms_percent -= percent_ms_buff;

    champ.add_temporary_effect(&KAISA_SUPERCHARGE_AS, 0.);
    PartDmg(0., 0., 0.)
}

const R_SHIELD_BY_R_LVL: [f32; 3] = [70., 90., 110.];
const R_SHIELD_AD_RATIO_BY_R_LVL: [f32; 3] = [0.90, 1.35, 1.8];

fn kaisa_r(champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    let r_lvl_idx: usize = usize::from(champ.r_lvl - 1); //to index ability ratios by lvl
    champ.single_use_heals_shields += R_SHIELD_BY_R_LVL[r_lvl_idx]
        + R_SHIELD_AD_RATIO_BY_R_LVL[r_lvl_idx] * champ.stats.ad()
        + 1.2 * champ.stats.ap();
    champ.units_travelled += 425.; //assumed dash range (max r radius around the ennemy - champion width)
    PartDmg(0., 0., 0.)
}

fn kaisa_fight_scenario(champ: &mut Unit, target_stats: &UnitStats, fight_duration: f32) {
    while champ.time < fight_duration {
        //priority order: w, e, q, basic attack
        if champ.w_cd == 0. {
            champ.w(target_stats);
        } else if champ.e_cd == 0. {
            champ.e(target_stats);
        } else if champ.q_cd == 0. {
            champ.q(target_stats);
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

const KAISA_BASE_AS: f32 = 0.644;
impl Unit {
    pub const KAISA_PROPERTIES: UnitProperties = UnitProperties {
        name: "Kaisa",
        as_limit: Unit::DEFAULT_AS_LIMIT,
        as_ratio: KAISA_BASE_AS, //if not specified, same as base AS
        windup_percent: 0.16108,
        windup_modifier: 1., //"mod" next to attack windup, 1 by default
        base_stats: UnitStats {
            hp: 640.,
            mana: 345.,
            base_ad: 59.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 25.,
            mr: 30.,
            base_as: KAISA_BASE_AS,
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
            hp: 102.,
            mana: 40.,
            base_ad: 2.6,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 4.2,
            mr: 1.3,
            base_as: 0.,
            bonus_as: 0.018,
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
            cast: kaisa_q,
            cast_time: F32_TOL,
            base_cooldown_by_ability_lvl: [10., 9., 8., 7., 6., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        w: BasicAbility {
            cast: kaisa_w,
            cast_time: 0.4,
            base_cooldown_by_ability_lvl: [22., 20., 18., 16., 14., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        e: BasicAbility {
            cast: kaisa_e,
            cast_time: F32_TOL, //e cast time is spend walking in the ability function
            base_cooldown_by_ability_lvl: [16., 14.5, 13., 11.5, 10., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        r: UltimateAbility {
            cast: kaisa_r,
            cast_time: F32_TOL,
            base_cooldown_by_ability_lvl: [130., 100., 70.],
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(kaisa_init_abilities),
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: Some(kaisa_on_basic_attack_cast),
            on_basic_attack_hit: Some(kaisa_second_skin),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
        fight_scenarios: &[(kaisa_fight_scenario, "all out")],
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
                e: [0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0],
                w: [0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1],
                r: [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0],
            },
            legendary_items_pool: &[
                //&Item::ABYSSAL_MASK,
                //&Item::AXIOM_ARC,
                &Item::BANSHEES_VEIL,
                &Item::BLACK_CLEAVER,
                &Item::BLACKFIRE_TORCH,
                &Item::BLADE_OF_THE_RUINED_KING,
                &Item::BLOODTHIRSTER,
                &Item::CHEMPUNK_CHAINSWORD,
                //&Item::COSMIC_DRIVE,
                &Item::CRYPTBLOOM,
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
                &Item::HORIZON_FOCUS,
                &Item::HUBRIS,
                &Item::HULLBREAKER,
                //&Item::ICEBORN_GAUNTLET,
                &Item::IMMORTAL_SHIELDBOW,
                &Item::INFINITY_EDGE,
                //&Item::JAKSHO,
                //&Item::KAENIC_ROOKERN,
                &Item::KRAKEN_SLAYER,
                &Item::LIANDRYS_TORMENT,
                //&Item::LICH_BANE,
                &Item::LORD_DOMINIKS_REGARDS,
                &Item::LUDENS_COMPANION,
                //&Item::MALIGNANCE, //cannot trigger passive
                &Item::MAW_OF_MALMORTIUS,
                &Item::MERCURIAL_SCIMITAR,
                &Item::MORELLONOMICON,
                &Item::MORTAL_REMINDER,
                &Item::MURAMANA,
                &Item::NASHORS_TOOTH,
                &Item::NAVORI_FLICKERBLADE,
                &Item::OPPORTUNITY,
                &Item::OVERLORDS_BLOODMAIL,
                &Item::PHANTOM_DANCER,
                //&Item::PROFANE_HYDRA,
                &Item::RABADONS_DEATHCAP,
                //&Item::RANDUINS_OMEN,
                &Item::RAPID_FIRECANNON,
                //&Item::RAVENOUS_HYDRA,
                //&Item::RIFTMAKER,
                &Item::ROD_OF_AGES,
                &Item::RUNAANS_HURRICANE,
                //&Item::RYLAIS_CRYSTAL_SCEPTER,
                &Item::SERAPHS_EMBRACE,
                &Item::SERPENTS_FANG,
                &Item::SERYLDAS_GRUDGE,
                &Item::SHADOWFLAME,
                &Item::SPEAR_OF_SHOJIN,
                &Item::STATIKK_SHIV,
                &Item::STERAKS_GAGE,
                &Item::STORMSURGE,
                //&Item::STRIDEBREAKER,
                &Item::SUNDERED_SKY,
                &Item::TERMINUS,
                &Item::THE_COLLECTOR,
                &Item::TITANIC_HYDRA,
                &Item::TRINITY_FORCE,
                &Item::UMBRAL_GLAIVE,
                &Item::VOID_STAFF,
                &Item::VOLTAIC_CYCLOSWORD,
                &Item::WITS_END,
                &Item::YOUMUUS_GHOSTBLADE,
                &Item::YUN_TAL_WILDARROWS,
                &Item::ZHONYAS_HOURGLASS,
            ],
            boots_pool: &[
                &Item::BERSERKERS_GREAVES,
                &Item::BOOTS_OF_SWIFTNESS,
                &Item::IONIAN_BOOTS_OF_LUCIDITY,
                //&Item::MERCURYS_TREADS,
                //&Item::PLATED_STEELCAPS,
                &Item::SORCERERS_SHOES,
            ],
            supp_items_pool: &[],
        },
    };
}
