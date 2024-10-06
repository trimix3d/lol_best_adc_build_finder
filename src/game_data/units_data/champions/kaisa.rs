use crate::game_data::units_data::*;

use items_data::items::*;

use enumset::enum_set;

//champion parameters (constants):
/// Percentage of target missing hp when second skin 5 stacks procs.
/// The missing hp taken for the calculation is the value AFTER the phys dmg from the basic attack hits,
const KAISA_SECOND_SKIN_TARGET_MISSING_HP_PERCENT: f32 = 0.55;
const KAISA_W_HIT_PERCENT: f32 = 0.85;

fn kaisa_init_abilities(champ: &mut Unit) {
    champ.effects_stacks[EffectStackId::KaisaSecondSkinStacks] = 0;
    champ.effects_values[EffectValueId::KaisaSuperchargeBonusAS] = 0.;

    //Q evolve, items ad/bonus_ad? + base_ad from lvls must be >= 100
    if champ.items_stats.bonus_ad + champ.lvl_stats.base_ad - champ.properties.base_stats.base_ad
        >= 100.
    {
        champ.effects_stacks[EffectStackId::KaisaQEvolved] = 1;
    } else {
        champ.effects_stacks[EffectStackId::KaisaQEvolved] = 0;
    }

    //W evolve, items ap must ne >= 100
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

const KAISA_SECOND_SKIN_MAX_STACKS: u8 = 5;

const KAISA_SECOND_SKIN_BASE_MAGIC_DMG_BY_LVL: [f32; MAX_UNIT_LVL] = [
    5.,    //lvl 1
    6.06,  //lvl 2
    7.12,  //lvl 3
    8.18,  //lvl 4
    9.24,  //lvl 5
    10.29, //lvl 6
    11.35, //lvl 7
    12.41, //lvl 8
    13.47, //lvl 9
    14.53, //lvl 10
    15.59, //lvl 11
    16.65, //lvl 12
    17.71, //lvl 13
    18.76, //lvl 14
    19.82, //lvl 15
    20.88, //lvl 16
    21.94, //lvl 17
    23.,   //lvl 18
];

const KAISA_SECOND_SKIN_MAGIC_DMG_PER_STACK_BY_LVL: [f32; MAX_UNIT_LVL] = [
    1.,    //lvl 1
    1.65,  //lvl 2
    2.29,  //lvl 3
    2.94,  //lvl 4
    3.59,  //lvl 5
    4.24,  //lvl 6
    4.88,  //lvl 7
    5.53,  //lvl 8
    6.18,  //lvl 9
    6.82,  //lvl 10
    7.47,  //lvl 11
    8.12,  //lvl 12
    8.76,  //lvl 13
    9.41,  //lvl 14
    10.06, //lvl 15
    10.71, //lvl 16
    11.35, //lvl 17
    12.,   //lvl 18
];

const KAISA_SECOND_SKIN_AP_COEF_BY_STACK: [f32; KAISA_SECOND_SKIN_MAX_STACKS as usize] =
    [0.15, 0.175, 0.20, 0.225, 0.25];

/// Assumes stacks application have less than 4s interval (patch 14.08) (doesn't take into account stack duration on the ennemy).
fn kaisa_second_skin_magic_dmg(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let lvl_idx: usize = usize::from(champ.lvl.get() - 1);

    //calculate dmg before stacks application
    let mut magic_dmg: f32 = KAISA_SECOND_SKIN_BASE_MAGIC_DMG_BY_LVL[lvl_idx]
        + f32::from(champ.effects_stacks[EffectStackId::KaisaSecondSkinStacks])
            * KAISA_SECOND_SKIN_MAGIC_DMG_PER_STACK_BY_LVL[lvl_idx]
        + champ.stats.ap()
            * KAISA_SECOND_SKIN_AP_COEF_BY_STACK
                [usize::from(champ.effects_stacks[EffectStackId::KaisaSecondSkinStacks])];

    //update stack count
    if champ.effects_stacks[EffectStackId::KaisaSecondSkinStacks]
        == KAISA_SECOND_SKIN_MAX_STACKS - 1
    {
        magic_dmg += KAISA_SECOND_SKIN_TARGET_MISSING_HP_PERCENT
            * target_stats.hp
            * (0.15 + 0.06 / 100. * champ.stats.ap());
        champ.effects_stacks[EffectStackId::KaisaSecondSkinStacks] = 0;
    } else {
        champ.effects_stacks[EffectStackId::KaisaSecondSkinStacks] += 1;
    }
    magic_dmg
}

fn kaisa_basic_attack(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    //basic attack reduce e cd by 0.5 sec
    champ.e_cd = f32::max(0., champ.e_cd - 0.5);

    let phys_dmg: f32 = champ.stats.ad() * champ.stats.crit_coef();
    let p_magic_dmg: f32 = kaisa_second_skin_magic_dmg(champ, target_stats);
    champ.dmg_on_target(
        target_stats,
        PartDmg(phys_dmg, p_magic_dmg, 0.),
        (1, 1),
        enum_set!(DmgTag::BasicAttack),
        1.,
    )
}

/// Assumes single target dmg.
const KAISA_Q_PHYS_DMG_BY_Q_LVL: [f32; 5] = [90., 123.75, 157.5, 191.25, 225.];
/// Assumes single target dmg.
const KAISA_EVOLVED_Q_PHYS_DMG_BY_Q_LVL: [f32; 5] = [150., 206.25, 262.5, 318.75, 375.];

fn kaisa_q(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    let q_lvl_idx: usize = usize::from(champ.q_lvl - 1); //to index ability ratios by lvl

    let n_missiles: u8;
    let phys_dmg: f32;
    //missiles on the same target do reduced dmg beyond the first
    if champ.effects_stacks[EffectStackId::KaisaQEvolved] == 0 {
        //not evolved
        n_missiles = 6;
        phys_dmg = KAISA_Q_PHYS_DMG_BY_Q_LVL[q_lvl_idx]
            + 1.2375 * champ.stats.bonus_ad
            + 0.45 * champ.stats.ap();
    //assumes single target dmg
    } else {
        //evolved
        n_missiles = 12;
        phys_dmg = KAISA_EVOLVED_Q_PHYS_DMG_BY_Q_LVL[q_lvl_idx]
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
const KAISA_W_PROJECTILE_SPEED: f32 = 1750.;
/// Has an impact on evolved w cd refund (greater travel time -> cd refund is less relevant).
const KAISA_W_TRAVEL_TIME: f32 = 1000. / KAISA_W_PROJECTILE_SPEED;

const KAISA_W_MAGIC_DMG_BY_W_LVL: [f32; 5] = [30., 55., 80., 105., 130.];

fn kaisa_w(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    let w_lvl_idx: usize = usize::from(champ.w_lvl - 1); //to index ability ratios by lvl

    let mut magic_dmg: f32 =
        KAISA_W_MAGIC_DMG_BY_W_LVL[w_lvl_idx] + 1.3 * champ.stats.ad() + 0.45 * champ.stats.ap();
    magic_dmg += kaisa_second_skin_magic_dmg(champ, target_stats)
        + kaisa_second_skin_magic_dmg(champ, target_stats); //applies proc one by one

    if champ.effects_stacks[EffectStackId::KaisaWEvolved] == 1 {
        //if evolved
        champ.w_cd -= KAISA_W_HIT_PERCENT * 0.75 * f32::max(0., champ.w_cd - KAISA_W_TRAVEL_TIME); //account for w travel time (otherwise cd is instantly refunded after casting and that can be op)
        magic_dmg += kaisa_second_skin_magic_dmg(champ, target_stats);
    }

    champ.dmg_on_target(
        target_stats,
        PartDmg(0., KAISA_W_HIT_PERCENT * magic_dmg, 0.),
        (1, 1),
        enum_set!(DmgTag::Ability),
        KAISA_W_HIT_PERCENT,
    )
}

const KAISA_E_MS_PERCENT_BY_E_LVL: [f32; 5] = [0.55, 0.60, 0.65, 0.70, 0.75];

const KAISA_E_BONUS_AS_BY_E_LVL: [f32; 5] = [0.40, 0.50, 0.60, 0.70, 0.80];

fn kaisa_supercharge_as_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_values[EffectValueId::KaisaSuperchargeBonusAS] == 0. {
        let bonus_as_buff: f32 = KAISA_E_BONUS_AS_BY_E_LVL[usize::from(champ.e_lvl - 1)];
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
    let percent_ms_buff: f32 = KAISA_E_MS_PERCENT_BY_E_LVL[e_lvl_idx] * (1. + capped_bonus_as);
    champ.stats.ms_percent += percent_ms_buff;
    champ.walk(1.2 / (1. + capped_bonus_as));
    champ.stats.ms_percent -= percent_ms_buff;

    champ.add_temporary_effect(&KAISA_SUPERCHARGE_AS, 0.);
    PartDmg(0., 0., 0.)
}

const KAISA_R_SHIELD_BY_R_LVL: [f32; 3] = [70., 90., 110.];
const KAISA_R_SHIELD_AD_RATIO_BY_R_LVL: [f32; 3] = [0.90, 1.35, 1.8];

fn kaisa_r(champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    let r_lvl_idx: usize = usize::from(champ.r_lvl - 1); //to index ability ratios by lvl
    champ.sim_logs.single_use_heals_shields += KAISA_R_SHIELD_BY_R_LVL[r_lvl_idx]
        + KAISA_R_SHIELD_AD_RATIO_BY_R_LVL[r_lvl_idx] * champ.stats.ad()
        + 1.2 * champ.stats.ap();
    champ.sim_logs.units_travelled += 425.; //assumed dash range (max r radius around the ennemy - champion width)
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
        windup_modifier: 1., //get it from https://leagueoflegends.fandom.com/wiki/List_of_champions/Basic_attacks, 1 by default
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
        basic_attack: kaisa_basic_attack,
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
            on_basic_attack_cast: None,
            on_basic_attack_hit: None,
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
        fight_scenarios: &[(kaisa_fight_scenario, "all out")],
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
                e: [0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0],
                w: [0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1],
                r: [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0],
            },
            legendary_items_pool: &[
                //&ABYSSAL_MASK,
                //&AXIOM_ARC,
                &BANSHEES_VEIL,
                &BLACK_CLEAVER,
                &BLACKFIRE_TORCH,
                &BLADE_OF_THE_RUINED_KING,
                &BLOODTHIRSTER,
                &CHEMPUNK_CHAINSWORD,
                //&COSMIC_DRIVE,
                &CRYPTBLOOM,
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
                &HORIZON_FOCUS,
                &HUBRIS,
                &HULLBREAKER,
                //&ICEBORN_GAUNTLET,
                &IMMORTAL_SHIELDBOW,
                &INFINITY_EDGE,
                //&JAKSHO,
                //&KAENIC_ROOKERN,
                &KRAKEN_SLAYER,
                &LIANDRYS_TORMENT,
                //&LICH_BANE,
                &LORD_DOMINIKS_REGARDS,
                &LUDENS_COMPANION,
                //&MALIGNANCE, //cannot trigger passive
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
                //&RIFTMAKER,
                &ROD_OF_AGES,
                &RUNAANS_HURRICANE,
                //&RYLAIS_CRYSTAL_SCEPTER,
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
                &YUN_TAL_WILDARROWS,
                &ZHONYAS_HOURGLASS,
            ],
            boots_pool: &[
                &BERSERKERS_GREAVES,
                &BOOTS_OF_SWIFTNESS,
                &IONIAN_BOOTS_OF_LUCIDITY,
                //&MERCURYS_TREADS,
                //&PLATED_STEELCAPS,
                &SORCERERS_SHOES,
            ],
            support_items_pool: &[],
        },
    };
}
