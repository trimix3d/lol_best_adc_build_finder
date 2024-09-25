use crate::game_data::{items_data::items::*, units_data::*};

//champion parameters (constants):
const CAITLYN_Q_N_TARGETS: f32 = 1.0;
const CAITLYN_Q_HIT_PERCENT: f32 = 0.85;

fn caitlyn_init_spells(champ: &mut Unit) {
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

fn caitlyn_heatshot_ad_dmg(champ: &Unit) -> f32 {
    champ.stats.ad()
        * (CAITLYN_HEADSHOT_AD_RATIO_BY_LVL[usize::from(champ.lvl.get() - 1)]
            + champ.stats.crit_dmg * 0.85 * champ.stats.crit_chance)
}

fn caitlyn_basic_attack(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let mut ad_dmg: f32 = champ.stats.ad() * champ.stats.crit_coef();

    if champ.effects_stacks[EffectStackId::CaitlynBonusHeadshot] == 1 {
        champ.effects_stacks[EffectStackId::CaitlynBonusHeadshot] = 0;
        ad_dmg += caitlyn_heatshot_ad_dmg(champ);
    } else if champ.effects_stacks[EffectStackId::CaitlynHeadshotStacks] == 5 {
        champ.effects_stacks[EffectStackId::CaitlynHeadshotStacks] = 0;
        ad_dmg += caitlyn_heatshot_ad_dmg(champ);
    } else {
        champ.effects_stacks[EffectStackId::CaitlynHeadshotStacks] += 1;
    }

    champ.dmg_on_target(
        target_stats,
        (ad_dmg, 0., 0.),
        (1, 1),
        DmgType::Other,
        true,
        1.,
    )
}

const CAITLYN_Q_BASE_DMG_BY_Q_LVL: [f32; 5] = [50., 90., 130., 170., 210.];
const CAITLYN_Q_AD_RATIO_BY_Q_LVL: [f32; 5] = [1.25, 1.45, 1.65, 1.85, 2.05];

fn caitlyn_q(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let q_lvl_idx: usize = usize::from(champ.q_lvl - 1); //to index spell ratios by lvl

    let ad_dmg: f32 = (1. + 0.5 * f32::max(0., CAITLYN_Q_N_TARGETS - 1.))
        * (CAITLYN_Q_BASE_DMG_BY_Q_LVL[q_lvl_idx]
            + champ.stats.ad() * CAITLYN_Q_AD_RATIO_BY_Q_LVL[q_lvl_idx]);

    champ.dmg_on_target(
        target_stats,
        (CAITLYN_Q_HIT_PERCENT * ad_dmg, 0., 0.),
        (1, 1),
        DmgType::Ability,
        false,
        CAITLYN_Q_N_TARGETS * CAITLYN_Q_HIT_PERCENT,
    )
}

fn caitlyn_w(_champ: &mut Unit, _target_stats: &UnitStats) -> f32 {
    //do nothing
    0.
}

const CAITLYN_E_BASE_DMG_BY_E_LVL: [f32; 5] = [80., 130., 180., 230., 280.];

fn caitlyn_e(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    champ.sim_results.units_travelled += 390.;

    let e_lvl_idx: usize = usize::from(champ.e_lvl - 1); //to index spell ratios by lvl

    let ap_dmg: f32 = CAITLYN_E_BASE_DMG_BY_E_LVL[e_lvl_idx] + 0.80 * champ.stats.ap();
    champ.effects_stacks[EffectStackId::CaitlynBonusHeadshot] = 1;

    champ.dmg_on_target(
        target_stats,
        (0., ap_dmg, 0.),
        (1, 1),
        DmgType::Ability,
        false,
        1.,
    )
}

const CAITLYN_R_BASE_DMG_BY_R_LVL: [f32; 3] = [300., 500., 700.];

fn caitlyn_r(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let r_lvl_idx: usize = usize::from(champ.r_lvl - 1); //to index spell ratios by lvl

    let ad_dmg: f32 = (CAITLYN_R_BASE_DMG_BY_R_LVL[r_lvl_idx] + 1.50 * champ.stats.bonus_ad)
        * (1. + 0.5 * champ.stats.crit_chance);

    champ.dmg_on_target(
        target_stats,
        (ad_dmg, 0., 0.),
        (1, 1),
        DmgType::Ultimate,
        false,
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

const CAITLYN_DEFAULT_RUNES_PAGE: RunesPage = RunesPage {
    shard1: RuneShard::Middle,
    shard2: RuneShard::Left,
    shard3: RuneShard::Left,
};

const CAITLYN_DEFAULT_SKILL_ORDER: SkillOrder = SkillOrder {
    //lvls:
    //  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18
    q: [1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    w: [0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0],
    e: [0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1],
    r: [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0],
};

const CAITLYN_DEFAULT_LEGENDARY_ITEMS: [&Item; 42] = [
    //&ABYSSAL_MASK,
    &AXIOM_ARC,
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

const CAITLYN_DEFAULT_BOOTS: [&Item; 2] = [
    &BERSERKERS_GREAVES,
    &BOOTS_OF_SWIFTNESS,
    //&IONIAN_BOOTS_OF_LUCIDITY,
    //&MERCURYS_TREADS,
    //&PLATED_STEELCAPS,
    //&SORCERERS_SHOES,
];

const CAITLYN_DEFAULT_SUPPORT_ITEMS: [&Item; 0] = [];

const CAITLYN_BASE_AS: f32 = 0.681;
impl Unit {
    pub const CAITLYN_PROPERTIES: UnitProperties = UnitProperties {
        name: "Caitlyn",
        as_limit: Unit::DEFAULT_AS_LIMIT,
        as_ratio: 0.610,
        windup_percent: 0.17708,
        windup_modifier: 1., //get it from https://leagueoflegends.fandom.com/wiki/List_of_champions/Basic_attacks, 1 by default
        base_stats: UnitStats {
            hp: 580.,
            mana: 315.,
            base_ad: 60.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_coef: 0.,
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
            ap_coef: 0.,
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
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_lvl_set: None,
        init_abilities: Some(caitlyn_init_spells),
        basic_attack: caitlyn_basic_attack,
        q: BasicAbility {
            cast: caitlyn_q,
            cast_time: 0.625,
            base_cooldown_by_ability_lvl: [10., 9., 8., 7., 6., F32_TOL], //basic spells only uses the first 5 values (except for aphelios)
        },
        w: BasicAbility {
            cast: caitlyn_w,
            cast_time: 0.25,
            base_cooldown_by_ability_lvl: [26., 22., 18., 14., 10., F32_TOL], //recharge time
        },
        e: BasicAbility {
            cast: caitlyn_e,
            cast_time: 0.15,
            base_cooldown_by_ability_lvl: [16., 14., 12., 10., 8., F32_TOL], //basic spells only uses the first 5 values (except for aphelios)
        },
        r: UltimateAbility {
            cast: caitlyn_r,
            cast_time: 1. + 0.375, //lock time + cast time
            base_cooldown_by_ability_lvl: [90., 90., 90.],
        },
        fight_scenarios: &[(caitlyn_fight_scenario, "all out")],
        unit_defaults: UnitDefaults {
            runes_pages: &CAITLYN_DEFAULT_RUNES_PAGE,
            skill_order: &CAITLYN_DEFAULT_SKILL_ORDER,
            legendary_items_pool: &CAITLYN_DEFAULT_LEGENDARY_ITEMS,
            boots_pool: &CAITLYN_DEFAULT_BOOTS,
            support_items_pool: &CAITLYN_DEFAULT_SUPPORT_ITEMS,
        },
    };
}
