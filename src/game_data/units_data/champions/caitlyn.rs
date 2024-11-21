use crate::game_data::*;

use items_data::Item;
use runes_data::*;
use units_data::*;

use enumset::enum_set;

//champion parameters (constants):
const CAITLYN_Q_N_TARGETS: f32 = 1.0;
const CAITLYN_Q_HIT_PERCENT: f32 = 0.85;

fn caitlyn_init_abilities(champ: &mut Unit) {
    champ.effects_stacks[EffectStackId::CaitlynHeadshotStacks] = 0;
    champ.effects_stacks[EffectStackId::CaitlynBonusHeadshot] = 0;
}

const CAITLYN_HEADSHOT_AD_RATIO_BY_LVL: [f32; MAX_UNIT_LVL] = [
    0.60, //lvl 1
    0.60, //lvl 2
    0.60, //lvl 3
    0.60, //lvl 4
    0.60, //lvl 5
    0.60, //lvl 6
    0.90, //lvl 7
    0.90, //lvl 8
    0.90, //lvl 9
    0.90, //lvl 10
    0.90, //lvl 11
    0.90, //lvl 12
    1.20, //lvl 13
    1.20, //lvl 14
    1.20, //lvl 15
    1.20, //lvl 16
    1.20, //lvl 17
    1.20, //lvl 18
];

fn caitlyn_heatshot_phys_dmg(champ: &Unit) -> f32 {
    champ.stats.ad()
        * (CAITLYN_HEADSHOT_AD_RATIO_BY_LVL[usize::from(champ.lvl.get() - 1)]
            + champ.stats.crit_dmg * 0.85 * champ.stats.crit_chance)
}

fn caitlyn_basic_attack(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    let mut phys_dmg: f32 = champ.stats.ad() * champ.stats.crit_coef();

    if champ.effects_stacks[EffectStackId::CaitlynBonusHeadshot] == 1 {
        champ.effects_stacks[EffectStackId::CaitlynBonusHeadshot] = 0;
        phys_dmg += caitlyn_heatshot_phys_dmg(champ);
    } else if champ.effects_stacks[EffectStackId::CaitlynHeadshotStacks] == 5 {
        champ.effects_stacks[EffectStackId::CaitlynHeadshotStacks] = 0;
        phys_dmg += caitlyn_heatshot_phys_dmg(champ);
    } else {
        champ.effects_stacks[EffectStackId::CaitlynHeadshotStacks] += 1;
    }

    champ.dmg_on_target(
        target_stats,
        PartDmg(phys_dmg, 0., 0.),
        (1, 1),
        enum_set!(DmgTag::BasicAttack),
        1.,
    )
}

const CAITLYN_Q_PHYS_DMG_BY_Q_LVL: [f32; 5] = [50., 90., 130., 170., 210.];
const CAITLYN_Q_AD_RATIO_BY_Q_LVL: [f32; 5] = [1.25, 1.45, 1.65, 1.85, 2.05];

fn caitlyn_q(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    let q_lvl_idx: usize = usize::from(champ.q_lvl - 1); //to index ability ratios by lvl

    let phys_dmg: f32 = (1. + 0.6 * f32::max(0., CAITLYN_Q_N_TARGETS - 1.))
        * (CAITLYN_Q_PHYS_DMG_BY_Q_LVL[q_lvl_idx]
            + champ.stats.ad() * CAITLYN_Q_AD_RATIO_BY_Q_LVL[q_lvl_idx]);

    champ.dmg_on_target(
        target_stats,
        PartDmg(CAITLYN_Q_HIT_PERCENT * phys_dmg, 0., 0.),
        (1, 1),
        enum_set!(DmgTag::Ability),
        CAITLYN_Q_N_TARGETS,
    )
}

fn caitlyn_w(_champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    //do nothing
    PartDmg(0., 0., 0.)
}

const CAITLYN_E_MAGIC_DMG_BY_E_LVL: [f32; 5] = [80., 130., 180., 230., 280.];

fn caitlyn_e(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    champ.units_travelled += 390.;

    let e_lvl_idx: usize = usize::from(champ.e_lvl - 1); //to index ability ratios by lvl

    let magic_dmg: f32 = CAITLYN_E_MAGIC_DMG_BY_E_LVL[e_lvl_idx] + 0.80 * champ.stats.ap();
    champ.effects_stacks[EffectStackId::CaitlynBonusHeadshot] = 1;

    champ.dmg_on_target(
        target_stats,
        PartDmg(0., magic_dmg, 0.),
        (1, 1),
        enum_set!(DmgTag::Ability),
        1.,
    )
}

const CAITLYN_R_PHYS_DMG_BY_R_LVL: [f32; 3] = [300., 500., 700.];

fn caitlyn_r(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    let r_lvl_idx: usize = usize::from(champ.r_lvl - 1); //to index ability ratios by lvl

    let phys_dmg: f32 = (CAITLYN_R_PHYS_DMG_BY_R_LVL[r_lvl_idx] + champ.stats.bonus_ad)
        * (1. + 0.5 * champ.stats.crit_chance);

    champ.dmg_on_target(
        target_stats,
        PartDmg(phys_dmg, 0., 0.),
        (1, 1),
        enum_set!(DmgTag::Ability | DmgTag::Ultimate),
        1.,
    )
}

fn caitlyn_fight_scenario(champ: &mut Unit, target_stats: &UnitStats, fight_duration: f32) {
    //e once at the beggining
    champ.e(target_stats);

    while champ.time < fight_duration {
        //priority order: q, basic attack
        if champ.q_cd == 0. {
            champ.q(target_stats);
        } else if champ.basic_attack_cd == 0. {
            champ.basic_attack(target_stats);
        } else {
            champ.walk(
                F32_TOL
                    + [
                        champ.q_cd,
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

const CAITLYN_BASE_AS: f32 = 0.681;
impl Unit {
    pub const CAITLYN_PROPERTIES: UnitProperties = UnitProperties {
        name: "Caitlyn",
        as_limit: Unit::DEFAULT_AS_LIMIT,
        as_ratio: 0.625,
        windup_percent: 0.17708,
        windup_modifier: 1., //get it from https://leagueoflegends.fandom.com/wiki/List_of_champions/Basic_attacks, 1 by default
        base_stats: UnitStats {
            hp: 580.,
            mana: 315.,
            base_ad: 60.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 27.,
            mr: 30.,
            base_as: CAITLYN_BASE_AS,
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
            hp: 107.,
            mana: 40.,
            base_ad: 3.8,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 4.7,
            mr: 1.3,
            base_as: 0.,
            bonus_as: 0.04,
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
        basic_attack: caitlyn_basic_attack,
        q: BasicAbility {
            cast: caitlyn_q,
            cast_time: 0.625,
            base_cooldown_by_ability_lvl: [10., 9., 8., 7., 6., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        w: BasicAbility {
            cast: caitlyn_w,
            cast_time: 0.25,
            base_cooldown_by_ability_lvl: [26., 22., 18., 14., 10., F32_TOL], //recharge time
        },
        e: BasicAbility {
            cast: caitlyn_e,
            cast_time: 0.15,
            base_cooldown_by_ability_lvl: [16., 14., 12., 10., 8., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        r: UltimateAbility {
            cast: caitlyn_r,
            cast_time: 1. + 0.375, //lock time + cast time
            base_cooldown_by_ability_lvl: [90., 90., 90.],
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(caitlyn_init_abilities),
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
        fight_scenarios: &[(caitlyn_fight_scenario, "all out")],
        defaults: UnitDefaults {
            runes_pages: RunesPage {
                keystone: &RuneKeystone::FLEET_FOOTWORK, //todo: prone to change (no real good rune for cait rn in 14.20 kek)
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
                &Item::AXIOM_ARC,
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
