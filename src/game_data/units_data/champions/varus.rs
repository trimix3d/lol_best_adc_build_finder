use crate::game_data::*;

use items_data::Item;
use runes_data::*;
use units_data::*;

use enumset::enum_set;

//champion parameters (constants):
const VARUS_ABILITIES_HIT_PERCENT: f32 = 0.9;
/// Average arrow charge considered to calculate dmg.
const VARUS_Q_CHARGE_PERCENT: f32 = 0.67;
/// Number of targets hit by q arrow.
const VARUS_Q_N_TARGETS: f32 = 1.0;
/// Percentage of the target missing hp for the q arrow empowered by w.
/// The missing hp taken for the calculation is the value AFTER the usual phys dmg from the arrow hits,
/// So don't put the value before the arrow hits in this constant, but a higher value to account for the usual arrow dmg.
const VARUS_W_TARGET_MISSING_HP_PERCENT: f32 = 0.4;
/// Number of targets hit by e.
const VARUS_E_N_TARGETS: f32 = 1.0;

fn varus_init_abilities(champ: &mut Unit) {
    champ.effects_stacks[EffectStackId::VarusBlightStacks] = 0;
    champ.effects_values[EffectValueId::VarusBlightLastStackTime] =
        -(VARUS_BLIGH_STACK_DELAY + F32_TOL); //to allow for effect at time == 0
    champ.effects_stacks[EffectStackId::VarusBlightedQuiverEmpowered] = 0;
}

//passive effect on kill not implemented (too situationnal)

const VARUS_BLIGH_STACK_DELAY: f32 = 6.; //stacks duration
fn varus_blighted_quiver_on_basic_attack_hit(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    n_targets: f32,
    _from_other_effects: bool,
) -> PartDmg {
    //if last hit from too long ago, reset stacks and add 1
    if champ.time - champ.effects_values[EffectValueId::VarusBlightLastStackTime]
        >= VARUS_BLIGH_STACK_DELAY
    {
        champ.effects_values[EffectValueId::VarusBlightLastStackTime] = champ.time;
        champ.effects_stacks[EffectStackId::VarusBlightStacks] = 1;
    } else if champ.effects_stacks[EffectStackId::VarusBlightStacks] < VARUS_MAX_BLIGHT_STACKS {
        //if last hit is recent enough (previous condition) but not fully stacked, add 1 stack
        champ.effects_stacks[EffectStackId::VarusBlightStacks] += 1;
        champ.effects_values[EffectValueId::VarusBlightLastStackTime] = champ.time;
    }

    let magic_dmg: f32 = VARUS_BLIGHT_ON_HIT_MAGIC_DMG_BY_W_LVL[usize::from(champ.w_lvl - 1)]
        + 0.35 * champ.stats.ap();
    PartDmg(0., n_targets * magic_dmg, 0.)
}

const VARUS_Q_CHARGE_SLOW_PERCENT: f32 = 0.20;
const VARUS_Q_MAX_CHARGE_COEF: f32 = 0.50; //bonus dmg coef when arrow is fully charged
const VARUS_Q_MAX_PHYS_DMG_BY_Q_LVL: [f32; 5] = [90., 160., 230., 300., 370.];
const VARUS_Q_MAX_BONUS_AD_RATIO_BY_Q_LVL: [f32; 5] = [1.30, 1.40, 1.50, 1.60, 1.70];

fn varus_q(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    const ARROW_CHARGE_WAIT_TIME: f32 = 1.25 * VARUS_Q_CHARGE_PERCENT;

    //approximate self slow by reducing ms_percent (current code doesn't handle slows)
    //approximating slows by reducing ms_percent is exact only when ms_percent is not modified by other effects during the duration of the self slow.
    let eq_charge_ms_percent: f32 =
        (1. + champ.stats.ms_percent) * (1. - VARUS_Q_CHARGE_SLOW_PERCENT) - 1.; //equivalent ms_percent during arrow charge
    let eq_ms_percent_debuff: f32 = champ.stats.ms_percent - eq_charge_ms_percent;

    champ.stats.ms_percent -= eq_ms_percent_debuff;
    champ.walk(ARROW_CHARGE_WAIT_TIME);
    champ.stats.ms_percent += eq_ms_percent_debuff;

    let q_lvl_idx: usize = usize::from(champ.q_lvl - 1); //to index ability ratios by lvl

    //arrow dmg
    const ARROW_CHARGE_DMG_COEF: f32 = 1. + VARUS_Q_MAX_CHARGE_COEF * VARUS_Q_CHARGE_PERCENT;
    const N_TARGET_COEF: f32 =
        VARUS_Q_N_TARGETS - 0.15 * (VARUS_Q_N_TARGETS * (VARUS_Q_N_TARGETS - 1.) / 2.); //dmg of arrow on all targets (diminishing returns)
    let phys_dmg: f32 = N_TARGET_COEF
        * (ARROW_CHARGE_DMG_COEF / (1. + VARUS_Q_MAX_CHARGE_COEF))
        * (VARUS_Q_MAX_PHYS_DMG_BY_Q_LVL[q_lvl_idx]
            + champ.stats.bonus_ad * VARUS_Q_MAX_BONUS_AD_RATIO_BY_Q_LVL[q_lvl_idx]); // dmg of arrow on 1 target

    //blight stacks
    let mut magic_dmg: f32 =
        ARROW_CHARGE_DMG_COEF * varus_consume_blight_stacks_magic_dmg(champ, target_stats); //assumes only one target has blights stacks

    //empowered by w
    if champ.effects_stacks[EffectStackId::VarusBlightedQuiverEmpowered] == 1 {
        champ.effects_stacks[EffectStackId::VarusBlightedQuiverEmpowered] = 0;
        magic_dmg += N_TARGET_COEF
            * ARROW_CHARGE_DMG_COEF
            * target_stats.hp
            * VARUS_W_TARGET_MISSING_HP_PERCENT
            * VARUS_W_TARGET_MISSING_HP_COEF_BY_W_LVL[usize::from(champ.w_lvl - 1)];
    }

    champ.dmg_on_target(
        target_stats,
        PartDmg(
            VARUS_ABILITIES_HIT_PERCENT * phys_dmg,
            VARUS_ABILITIES_HIT_PERCENT * magic_dmg,
            0.,
        ),
        (1, 1),
        enum_set!(DmgTag::Ability),
        VARUS_Q_N_TARGETS,
    )
}

const VARUS_MAX_BLIGHT_STACKS: u8 = 3;
const VARUS_BLIGHT_ON_HIT_MAGIC_DMG_BY_W_LVL: [f32; 5] = [8., 13., 18., 23., 28.];

const VARUS_W_TARGET_MISSING_HP_COEF_BY_W_LVL: [f32; 5] = [0.06, 0.08, 0.10, 0.12, 0.14];

fn varus_w(champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    //w passive is implemented inside varus basic attacks
    champ.effects_stacks[EffectStackId::VarusBlightedQuiverEmpowered] = 1;
    PartDmg(0., 0., 0.)
}

const VARUS_TARGET_HP_COEF_PER_BLIGHT_STACK_BY_W_LVL: [f32; 5] = [0.03, 0.035, 0.04, 0.045, 0.05];
const VARUS_TOT_CD_REFUND_PERCENT_PER_BLIGHT_STACK: f32 = 0.13;

/// Consumes blights stacks and return proc dmg.
/// Always assumes blight stacks are applied on one target only.
fn varus_consume_blight_stacks_magic_dmg(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let n_stacks: f32 = f32::from(champ.effects_stacks[EffectStackId::VarusBlightStacks]);
    champ.effects_stacks[EffectStackId::VarusBlightStacks] = 0; //consume all blight stacks

    champ.q_cd = f32::max(
        0.,
        champ.q_cd
            - n_stacks
                * VARUS_ABILITIES_HIT_PERCENT
                * VARUS_TOT_CD_REFUND_PERCENT_PER_BLIGHT_STACK
                * champ.properties.q.base_cooldown_by_ability_lvl[usize::from(champ.q_lvl - 1)],
    );
    champ.w_cd = f32::max(
        0.,
        champ.w_cd
            - n_stacks
                * VARUS_ABILITIES_HIT_PERCENT
                * VARUS_TOT_CD_REFUND_PERCENT_PER_BLIGHT_STACK
                * champ.properties.w.base_cooldown_by_ability_lvl[usize::from(champ.w_lvl - 1)],
    );
    champ.e_cd = f32::max(
        0.,
        champ.e_cd
            - n_stacks
                * VARUS_ABILITIES_HIT_PERCENT
                * VARUS_TOT_CD_REFUND_PERCENT_PER_BLIGHT_STACK
                * champ.properties.e.base_cooldown_by_ability_lvl[usize::from(champ.e_lvl - 1)],
    );

    n_stacks
        * target_stats.hp
        * (VARUS_TARGET_HP_COEF_PER_BLIGHT_STACK_BY_W_LVL[usize::from(champ.w_lvl - 1)]
            + 0.00015 * champ.stats.ap())
}

const VARUS_E_PHYS_DMG_BY_E_LVL: [f32; 5] = [60., 100., 140., 180., 220.];

fn varus_e(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    let e_lvl_idx: usize = usize::from(champ.e_lvl - 1); //to index ability ratios by lvl

    let phys_dmg: f32 =
        VARUS_E_N_TARGETS * VARUS_E_PHYS_DMG_BY_E_LVL[e_lvl_idx] + champ.stats.bonus_ad;
    let magic_dmg: f32 = varus_consume_blight_stacks_magic_dmg(champ, target_stats); //assumes only one target has blights stacks

    champ.dmg_on_target(
        target_stats,
        PartDmg(
            VARUS_ABILITIES_HIT_PERCENT * phys_dmg,
            VARUS_ABILITIES_HIT_PERCENT * magic_dmg,
            0.,
        ),
        (1, 1),
        enum_set!(DmgTag::Ability),
        VARUS_E_N_TARGETS,
    )
}

fn varus_r_add_delayed_blight_stack_enable(_champ: &mut Unit, _availability_coef: f32) {}

fn varus_r_add_delayed_blight_stack_disable(champ: &mut Unit) {
    //add blight stack after a set duration
    champ.effects_stacks[EffectStackId::VarusBlightStacks] = u8::min(
        VARUS_MAX_BLIGHT_STACKS,
        champ.effects_stacks[EffectStackId::VarusBlightStacks] + 1,
    );
}

const VARUS_R_ADD_DELAYED_BLIGHT_STACKS_0_5: TemporaryEffect = TemporaryEffect {
    id: EffectId::VarusRAddDelayedBlightStacks05,
    add_stack: varus_r_add_delayed_blight_stack_enable,
    remove_every_stack: varus_r_add_delayed_blight_stack_disable,
    duration: 0.5 + VARUS_R_TRAVEL_TIME,
    cooldown: 0.,
};

const VARUS_R_ADD_DELAYED_BLIGHT_STACKS_1_0: TemporaryEffect = TemporaryEffect {
    id: EffectId::VarusRAddDelayedBlightStacks10,
    add_stack: varus_r_add_delayed_blight_stack_enable,
    remove_every_stack: varus_r_add_delayed_blight_stack_disable,
    duration: 1. + VARUS_R_TRAVEL_TIME,
    cooldown: 0.,
};

const VARUS_R_ADD_DELAYED_BLIGHT_STACKS_1_5: TemporaryEffect = TemporaryEffect {
    id: EffectId::VarusRAddDelayedBlightStacks15,
    add_stack: varus_r_add_delayed_blight_stack_enable,
    remove_every_stack: varus_r_add_delayed_blight_stack_disable,
    duration: 1.5 + VARUS_R_TRAVEL_TIME,
    cooldown: 0.,
};

/// Used to calculate the average travel time of the projectile.
const VARUS_R_PROJECTILE_SPEED: f32 = 1500.;
/// Affects how fast the blight stacks are applied after cast.
const VARUS_R_TRAVEL_TIME: f32 = 600. / VARUS_R_PROJECTILE_SPEED;
const VARUS_R_MAGIC_DMG_BY_R_LVL: [f32; 3] = [150., 250., 350.];

fn varus_r(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    let r_lvl_idx: usize = usize::from(champ.r_lvl - 1); //to index ability ratios by lvl

    let mut magic_dmg: f32 = VARUS_R_MAGIC_DMG_BY_R_LVL[r_lvl_idx] + champ.stats.ap();
    magic_dmg += varus_consume_blight_stacks_magic_dmg(champ, target_stats); //assumes only one target has blights stacks

    //add delayed blights stacks
    champ.add_temporary_effect(&VARUS_R_ADD_DELAYED_BLIGHT_STACKS_0_5, 0.);
    champ.add_temporary_effect(&VARUS_R_ADD_DELAYED_BLIGHT_STACKS_1_0, 0.);
    champ.add_temporary_effect(&VARUS_R_ADD_DELAYED_BLIGHT_STACKS_1_5, 0.);

    champ.dmg_on_target(
        target_stats,
        PartDmg(0., VARUS_ABILITIES_HIT_PERCENT * magic_dmg, 0.),
        (1, 1),
        enum_set!(DmgTag::Ability | DmgTag::Ultimate),
        1.,
    )
}

fn varus_fight_scenario_all_out(champ: &mut Unit, target_stats: &UnitStats, fight_duration: f32) {
    //add e + weighted r dmg at the beginning
    champ.e(target_stats);
    champ.weighted_r(target_stats);

    while champ.time < fight_duration {
        //priority order: q (+w when available) when at least 2 blight stacks, e when at least 1 blight stacks, basic attack
        if champ.q_cd == 0. && champ.effects_stacks[EffectStackId::VarusBlightStacks] >= 2 {
            if champ.w_cd == 0. {
                champ.w(target_stats);
            }
            champ.q(target_stats);
        } else if champ.e_cd == 0. && champ.effects_stacks[EffectStackId::VarusBlightStacks] >= 1 {
            champ.e(target_stats);
        } else if champ.basic_attack_cd == 0. {
            champ.basic_attack(target_stats);
        } else {
            champ.walk(
                F32_TOL
                    + [
                        if champ.effects_stacks[EffectStackId::VarusBlightStacks] >= 2 {
                            champ.q_cd
                        } else {
                            champ.basic_attack_cd
                        },
                        if champ.effects_stacks[EffectStackId::VarusBlightStacks] >= 1 {
                            champ.e_cd
                        } else {
                            champ.basic_attack_cd
                        },
                        champ.basic_attack_cd,
                        f32::max(0., fight_duration - champ.time),
                    ]
                    .into_iter()
                    .min_by(|a, b| a.partial_cmp(b).expect("Failed to compare floats"))
                    .unwrap(),
            );
        }
    }
}

fn varus_fight_scenario_poke(champ: &mut Unit, target_stats: &UnitStats, fight_duration: f32) {
    while champ.time < fight_duration {
        //priority order: q (+w when available), e (dont use blight stacks for poke scenario)
        if champ.q_cd == 0. {
            if champ.w_cd == 0. {
                champ.w(target_stats);
            }
            champ.q(target_stats);
        } else if champ.e_cd == 0. {
            champ.e(target_stats);
        } else {
            champ.walk(
                F32_TOL
                    + [
                        champ.q_cd,
                        champ.e_cd,
                        f32::max(0., fight_duration - champ.time),
                    ]
                    .into_iter()
                    .min_by(|a, b| a.partial_cmp(b).expect("Failed to compare floats"))
                    .unwrap(),
            );
        }
    }
    //add weighed r dmg + 2 basic attacks at the end
    champ.weighted_r(target_stats);
    champ.basic_attack(target_stats);
    champ.walk(champ.basic_attack_cd + F32_TOL);
    champ.basic_attack(target_stats);
}

const VARUS_BASE_AS: f32 = 0.658;
impl Unit {
    pub const VARUS_PROPERTIES: UnitProperties = UnitProperties {
        name: "Varus",
        as_limit: Unit::DEFAULT_AS_LIMIT,
        as_ratio: VARUS_BASE_AS,
        windup_percent: 0.17544,
        windup_modifier: 1., //get it from https://leagueoflegends.fandom.com/wiki/List_of_champions/Basic_attacks, 1 by default
        base_stats: UnitStats {
            hp: 600.,
            mana: 360.,
            base_ad: 59.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 27.,
            mr: 30.,
            base_as: VARUS_BASE_AS,
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
            hp: 105.,
            mana: 40.,
            base_ad: 3.4,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 4.6,
            mr: 1.3,
            base_as: 0.,
            bonus_as: 0.035,
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
            cast: varus_q,
            cast_time: F32_TOL, //cast time done inside ability function
            base_cooldown_by_ability_lvl: [16., 15., 14., 13., 12., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        w: BasicAbility {
            cast: varus_w,
            cast_time: F32_TOL,
            base_cooldown_by_ability_lvl: [40., 40., 40., 40., 40., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        e: BasicAbility {
            cast: varus_e,
            cast_time: 0.2419,
            base_cooldown_by_ability_lvl: [18., 16., 14., 12., 10., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        r: UltimateAbility {
            cast: varus_r,
            cast_time: 0.2419,
            base_cooldown_by_ability_lvl: [100., 80., 60.],
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(varus_init_abilities),
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: Some(varus_blighted_quiver_on_basic_attack_hit),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
        fight_scenarios: &[
            (varus_fight_scenario_all_out, "all out"),
            (varus_fight_scenario_poke, "poke"),
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
                &Item::BANSHEES_VEIL,
                &Item::BLACK_CLEAVER,
                &Item::BLACKFIRE_TORCH,
                &Item::BLADE_OF_THE_RUINED_KING,
                &Item::BLOODTHIRSTER,
                &Item::CHEMPUNK_CHAINSWORD,
                &Item::COSMIC_DRIVE,
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
                //&Item::HORIZON_FOCUS,
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
                //&Item::LUDENS_COMPANION,
                &Item::MALIGNANCE,
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
                //&Item::SPEAR_OF_SHOJIN,
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
            support_items_pool: &[],
        },
    };
}
