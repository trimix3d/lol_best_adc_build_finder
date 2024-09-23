use crate::game_data::{items_data::items::*, units_data::*};

//champion parameters (constants):
const APHELIOS_Q_CALIBRUM_HIT_PERCENT: f32 = 0.9;
/// Number of ennemies hit by the infernum Q cone.
const APHELIOS_Q_INFERNUM_N_TARGETS: f32 = 1.0;
/// Number of Q sentry attacks.
const APHELIOS_Q_CRESCENDUM_N_SENTRY_ATTACKS: f32 = 1.0;
/// Number of ennemies hit by R.
const APHELIOS_R_N_TARGETS: f32 = 1.5;

fn aphelios_on_lvl_set(champ: &mut Unit) {
    //aphelios passive (gain stats according to spells lvl)
    champ.lvl_stats.bonus_ad += 4.5 * f32::from(champ.q_lvl); //bonus_ad by q_lvl
    champ.lvl_stats.bonus_as += 0.09 * f32::from(champ.w_lvl); //bonus_as by w_lvl
    champ.lvl_stats.lethality += 5.5 * f32::from(champ.e_lvl); //lethality by e_lvl
}

//default_basic_attack with an aditionnal mutliplier due to infernum bonus basic attack dmg (averaged on 5 weapons)
const APHELIOS_BASIC_ATTACKS_MULTIPLIER: f32 = 1. * (4. / 5.) + 1.10 * (1. / 5.);
fn aphelios_basic_attack(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let ad_dmg: f32 =
        APHELIOS_BASIC_ATTACKS_MULTIPLIER * champ.stats.ad() * champ.stats.crit_coef();
    champ.dmg_on_target(
        target_stats,
        (ad_dmg, 0., 0.),
        (1, 1),
        DmgSource::Other,
        true,
        1.,
    )
}

const APHELIOS_Q_CALIBRUM_AD_DMG_BY_LVL: [f32; MAX_UNIT_LVL] = [
    60.,    //lvl 1
    60.,    //lvl 2
    76.67,  //lvl 3
    76.67,  //lvl 4
    93.33,  //lvl 5
    93.33,  //lvl 6
    110.,   //lvl 7
    110.,   //lvl 8
    126.67, //lvl 9
    126.67, //lvl 10
    143.33, //lvl 11
    143.33, //lvl 12
    160.,   //lvl 13
    160.,   //lvl 14
    160.,   //lvl 15
    160.,   //lvl 16
    160.,   //lvl 17
    160.,   //lvl 18
];
const APHELIOS_Q_CALIBRUM_BONUS_AD_RATIO_BY_LVL: [f32; MAX_UNIT_LVL] = [
    0.42, //lvl 1
    0.42, //lvl 2
    0.45, //lvl 3
    0.45, //lvl 4
    0.48, //lvl 5
    0.48, //lvl 6
    0.51, //lvl 7
    0.51, //lvl 8
    0.54, //lvl 9
    0.54, //lvl 10
    0.57, //lvl 11
    0.57, //lvl 12
    0.6,  //lvl 13
    0.6,  //lvl 14
    0.6,  //lvl 15
    0.6,  //lvl 16
    0.6,  //lvl 17
    0.6,  //lvl 18
];
const APHELIOS_Q_SEVERUM_AD_DMG_BY_LVL: [f32; MAX_UNIT_LVL] = [
    10., //lvl 1
    10., //lvl 2
    15., //lvl 3
    15., //lvl 4
    20., //lvl 5
    20., //lvl 6
    25., //lvl 7
    25., //lvl 8
    30., //lvl 9
    30., //lvl 10
    35., //lvl 11
    35., //lvl 12
    40., //lvl 13
    40., //lvl 14
    40., //lvl 15
    40., //lvl 16
    40., //lvl 17
    40., //lvl 18
];
const APHELIOS_Q_SEVERUM_BONUS_AD_RATIO_BY_LVL: [f32; MAX_UNIT_LVL] = [
    0.22, //lvl 1
    0.22, //lvl 2
    0.25, //lvl 3
    0.25, //lvl 4
    0.28, //lvl 5
    0.28, //lvl 6
    0.31, //lvl 7
    0.31, //lvl 8
    0.34, //lvl 9
    0.34, //lvl 10
    0.37, //lvl 11
    0.37, //lvl 12
    0.40, //lvl 13
    0.40, //lvl 14
    0.40, //lvl 15
    0.40, //lvl 16
    0.40, //lvl 17
    0.40, //lvl 18
];
const APHELIOS_Q_GRAVITUM_AP_DMG_BY_LVL: [f32; MAX_UNIT_LVL] = [
    50.,  //lvl 1
    50.,  //lvl 2
    65.,  //lvl 3
    65.,  //lvl 4
    80.,  //lvl 5
    80.,  //lvl 6
    95.,  //lvl 7
    95.,  //lvl 8
    110., //lvl 9
    110., //lvl 10
    125., //lvl 11
    125., //lvl 12
    140., //lvl 13
    140., //lvl 14
    140., //lvl 15
    140., //lvl 16
    140., //lvl 17
    140., //lvl 18
];
const APHELIOS_Q_GRAVITUM_BONUS_AD_RATIO_BY_LVL: [f32; MAX_UNIT_LVL] = [
    0.32, //lvl 1
    0.32, //lvl 2
    0.35, //lvl 3
    0.35, //lvl 4
    0.38, //lvl 5
    0.38, //lvl 6
    0.41, //lvl 7
    0.41, //lvl 8
    0.44, //lvl 9
    0.44, //lvl 10
    0.47, //lvl 11
    0.47, //lvl 12
    0.50, //lvl 13
    0.50, //lvl 14
    0.50, //lvl 15
    0.50, //lvl 16
    0.50, //lvl 17
    0.50, //lvl 18
];
const APHELIOS_Q_CRESCENDUM_AD_DMG_BY_LVL: [f32; MAX_UNIT_LVL] = [
    35.,  //lvl 1
    35.,  //lvl 2
    50.,  //lvl 3
    50.,  //lvl 4
    65.,  //lvl 5
    65.,  //lvl 6
    80.,  //lvl 7
    80.,  //lvl 8
    95.,  //lvl 9
    95.,  //lvl 10
    110., //lvl 11
    110., //lvl 12
    125., //lvl 13
    125., //lvl 14
    125., //lvl 15
    125., //lvl 16
    125., //lvl 17
    125., //lvl 18
];
const APHELIOS_Q_CRESCENDUM_BONUS_AD_RATIO_BY_LVL: [f32; MAX_UNIT_LVL] = [
    0.42, //lvl 1
    0.42, //lvl 2
    0.45, //lvl 3
    0.45, //lvl 4
    0.48, //lvl 5
    0.48, //lvl 6
    0.51, //lvl 7
    0.51, //lvl 8
    0.54, //lvl 9
    0.54, //lvl 10
    0.57, //lvl 11
    0.57, //lvl 12
    0.60, //lvl 13
    0.60, //lvl 14
    0.60, //lvl 15
    0.60, //lvl 16
    0.60, //lvl 17
    0.60, //lvl 18
];
const APHELIOS_Q_INFERNUM_AD_DMG_BY_LVL: [f32; MAX_UNIT_LVL] = [
    25.,   //lvl 1
    25.,   //lvl 2
    31.67, //lvl 3
    31.67, //lvl 4
    38.33, //lvl 5
    38.33, //lvl 6
    45.,   //lvl 7
    45.,   //lvl 8
    51.67, //lvl 9
    51.67, //lvl 10
    58.33, //lvl 11
    58.33, //lvl 12
    65.,   //lvl 13
    65.,   //lvl 14
    65.,   //lvl 15
    65.,   //lvl 16
    65.,   //lvl 17
    65.,   //lvl 18
];
const APHELIOS_Q_INFERNUM_BONUS_AD_RATIO_BY_LVL: [f32; MAX_UNIT_LVL] = [
    0.56, //lvl 1
    0.56, //lvl 2
    0.6,  //lvl 3
    0.6,  //lvl 4
    0.64, //lvl 5
    0.64, //lvl 6
    0.68, //lvl 7
    0.68, //lvl 8
    0.72, //lvl 9
    0.72, //lvl 10
    0.76, //lvl 11
    0.76, //lvl 12
    0.8,  //lvl 13
    0.8,  //lvl 14
    0.8,  //lvl 15
    0.8,  //lvl 16
    0.8,  //lvl 17
    0.8,  //lvl 18
];

fn aphelios_q(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let lvl_idx: usize = usize::from(champ.lvl.get() - 1); //to index spell ratios by lvl

    //calibrium weighted 1/5 (doesn't count the basic attack that triggers the mark)
    let mut ad_dmg: f32 = APHELIOS_Q_CALIBRUM_HIT_PERCENT / 5.
        * (APHELIOS_Q_CALIBRUM_AD_DMG_BY_LVL[lvl_idx]
            + APHELIOS_Q_CALIBRUM_BONUS_AD_RATIO_BY_LVL[lvl_idx] * champ.stats.bonus_ad
            + champ.stats.ap()); //projectile dmg
    let mut basic_attack_ad_dmg: f32 =
        APHELIOS_Q_CALIBRUM_HIT_PERCENT / 5. * (15. + 0.2 * champ.stats.bonus_ad); //mark dmg (considered basic attack dmg)

    //severum weighted 1/5 (no on_hit applied because we don't want to stack those effects since we consider the average q_cast)
    basic_attack_ad_dmg += 1. / 5.
        * (6. + 2. * champ.stats.bonus_as).round()
        * ((APHELIOS_Q_SEVERUM_AD_DMG_BY_LVL[lvl_idx]
            + APHELIOS_Q_SEVERUM_BONUS_AD_RATIO_BY_LVL[lvl_idx] * champ.stats.bonus_ad)
            * champ.stats.crit_coef());

    //gravitum weighted 1/5
    let ap_dmg: f32 = 1. / 5.
        * (APHELIOS_Q_GRAVITUM_AP_DMG_BY_LVL[lvl_idx]
            + APHELIOS_Q_GRAVITUM_BONUS_AD_RATIO_BY_LVL[lvl_idx] * champ.stats.bonus_ad
            + 0.7 * champ.stats.ap());

    //infernum weighted 1/5
    ad_dmg += 1. / 5.
        * APHELIOS_Q_INFERNUM_N_TARGETS
        * (APHELIOS_Q_INFERNUM_AD_DMG_BY_LVL[lvl_idx]
            + APHELIOS_Q_INFERNUM_BONUS_AD_RATIO_BY_LVL[lvl_idx] * champ.stats.bonus_ad
            + 0.7 * champ.stats.ap()); //cone AoE dmg
    basic_attack_ad_dmg +=
        1. / 5. * APHELIOS_Q_INFERNUM_N_TARGETS * champ.stats.ad() * champ.stats.crit_coef(); //additionnal basic attack dmg (no on_hit applied because we don't want to stack those effects since we consider the average q_cast)

    //crescendum weighted 1/5, considered spell dmg
    ad_dmg += 1. / 5.
        * APHELIOS_Q_CRESCENDUM_N_SENTRY_ATTACKS
        * (APHELIOS_Q_CRESCENDUM_AD_DMG_BY_LVL[lvl_idx]
            + APHELIOS_Q_CRESCENDUM_BONUS_AD_RATIO_BY_LVL[lvl_idx] * champ.stats.bonus_ad
            + 0.5 * champ.stats.ap())
        * champ.stats.crit_coef();

    //consider 2 hits: initial spell + basic attack (severum, infernum)
    champ.dmg_on_target(
        target_stats,
        (ad_dmg, ap_dmg, 0.),
        (1, 1),
        DmgSource::BasicSpell,
        false,
        (2. + APHELIOS_Q_CALIBRUM_HIT_PERCENT + APHELIOS_Q_INFERNUM_N_TARGETS) / 5.,
    ) + champ.dmg_on_target(
        target_stats,
        (basic_attack_ad_dmg, 0., 0.),
        (1, 1),
        DmgSource::Other,
        true,
        (1. + APHELIOS_Q_CALIBRUM_HIT_PERCENT + APHELIOS_Q_INFERNUM_N_TARGETS) / 5.,
    )
}

const APHELIOS_R_INITIAL_AD_DMG_BY_R_LVL: [f32; 3] = [125., 175., 225.];
const APHELIOS_R_CALIBRUM_AD_DMG_BY_R_LVL: [f32; 3] = [50., 80., 110.];
const APHELIOS_R_SEVERUM_HEAL_BY_R_LVL: [f32; 3] = [250., 350., 450.];
const APHELIOS_R_INFERNUM_AD_DMG_BY_R_LVL: [f32; 3] = [50., 100., 150.];
fn aphelios_r(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let r_lvl_idx: usize = usize::from(champ.r_lvl - 1); //to index spell ratios by lvl

    //initial projectile dmg
    let ad_dmg: f32 = APHELIOS_R_N_TARGETS
        * (APHELIOS_R_INITIAL_AD_DMG_BY_R_LVL[r_lvl_idx]
            + 0.2 * champ.stats.bonus_ad
            + champ.stats.ap());

    //basic attack coming after
    let special_crit_coef: f32 =
        1. + champ.stats.crit_chance * (0.2 + champ.stats.crit_dmg - Unit::BASE_CRIT_DMG); //special crit coef for R basic attack
    let basic_attack_ad_dmg: f32 = APHELIOS_R_N_TARGETS
        * APHELIOS_BASIC_ATTACKS_MULTIPLIER
        * (champ.stats.ad() * special_crit_coef);

    //calibrum, mark (considered basic attack dmg) weighted 1/5
    let mark_ad_dmg: f32 = 1. / 5.
        * APHELIOS_R_N_TARGETS
        * (APHELIOS_R_CALIBRUM_AD_DMG_BY_R_LVL[r_lvl_idx] + 15. + 0.2 * champ.stats.bonus_ad);

    //severum, heal weighted 1/5
    champ.sim_results.heals_shields += 1. / 5. * (APHELIOS_R_SEVERUM_HEAL_BY_R_LVL[r_lvl_idx]);

    //gravitum, root not taken into account

    //infernum, AoE dmg weighted 1/5
    let mut infernum_ad_dmg: f32 = 1. / 5.
        * APHELIOS_R_N_TARGETS
        * (APHELIOS_R_INFERNUM_AD_DMG_BY_R_LVL[r_lvl_idx] + 0.25 * champ.stats.bonus_ad); //initial spell additionnal dmg
    infernum_ad_dmg += 1. / 5. * (APHELIOS_R_N_TARGETS - 1.) * (0.9 * basic_attack_ad_dmg); //infernum 200 years AoE dmgs coming from other targets hit

    //crescendum, chakrams not taken into account

    //2 hits: initial projectile + basic attack
    champ.dmg_on_target(
        target_stats,
        (ad_dmg + infernum_ad_dmg, 0., 0.),
        (1, 1),
        DmgSource::UltimateSpell,
        false,
        APHELIOS_R_N_TARGETS,
    ) + champ.dmg_on_target(
        target_stats,
        (basic_attack_ad_dmg + mark_ad_dmg, 0., 0.),
        (1, 1),
        DmgSource::Other,
        true,
        APHELIOS_R_N_TARGETS,
    )
}

fn aphelios_fight_scenario(champ: &mut Unit, target_stats: &UnitStats, fight_duration: f32) {
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

const APHELIOS_DEFAULT_RUNES_PAGE: RunesPage = RunesPage {
    shard1: RuneShard::Middle,
    shard2: RuneShard::Left,
    shard3: RuneShard::Left,
};

const APHELIOS_DEFAULT_SKILL_ORDER: SkillOrder = SkillOrder {
    //lvls:
    //  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18
    q: [1, 1, 1, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    e: [0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0],
    w: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1],
    r: [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0],
};

const APHELIOS_DEFAULT_LEGENDARY_ITEMS: [&Item; 40] = [
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
    //&GUINSOOS_RAGEBLADE,
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

const APHELIOS_DEFAULT_BOOTS: [&Item; 2] = [
    &BERSERKERS_GREAVES,
    &BOOTS_OF_SWIFTNESS,
    //&IONIAN_BOOTS_OF_LUCIDITY,
    //&MERCURYS_TREADS,
    //&PLATED_STEELCAPS,
    //&SORCERERS_SHOES,
];

const APHELIOS_DEFAULT_SUPPORT_ITEMS: [&Item; 0] = [];

const APHELIOS_BASE_AS: f32 = 0.64;
impl Unit {
    pub const APHELIOS_PROPERTIES: UnitProperties = UnitProperties {
        name: "Aphelios",
        as_limit: Unit::DEFAULT_AS_LIMIT,
        as_ratio: APHELIOS_BASE_AS, //if not specified, same as base AS
        windup_percent: 0.15333,
        windup_modifier: 1., //get it from https://leagueoflegends.fandom.com/wiki/List_of_champions/Basic_attacks, 1 by default
        base_stats: UnitStats {
            hp: 580.,
            mana: 348.,
            base_ad: 55.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_coef: 0.,
            armor: 26.,
            mr: 30.,
            base_as: APHELIOS_BASE_AS,
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
        },
        growth_stats: UnitStats {
            hp: 102.,
            mana: 42.,
            base_ad: 2.3,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_coef: 0.,
            armor: 4.2,
            mr: 1.3,
            base_as: 0.,
            bonus_as: 0.021,
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
        },
        on_lvl_set: Some(aphelios_on_lvl_set),
        init_abilities: None,
        basic_attack: aphelios_basic_attack,
        q: BasicSpell {
            cast: aphelios_q,
            cast_time: 0.62,
            base_cooldown_by_spell_lvl: [8.4, 8.4, 8.4, 8.4, 8.4, 8.4], //mean cd for all weapons at lvl 9 (aphelios q_lvl gives bonus_ad and doesn't affect cd)
        },
        w: NULL_BASIC_SPELL,
        e: NULL_BASIC_SPELL,
        r: UltimateSpell {
            cast: aphelios_r,
            cast_time: 0.6,
            base_cooldown_by_spell_lvl: [120., 110., 100.],
        },
        fight_scenarios: &[(aphelios_fight_scenario, "all out")],
        unit_defaults: UnitDefaults {
            runes_pages: &APHELIOS_DEFAULT_RUNES_PAGE,
            skill_order: &APHELIOS_DEFAULT_SKILL_ORDER,
            legendary_items_pool: &APHELIOS_DEFAULT_LEGENDARY_ITEMS,
            boots_pool: &APHELIOS_DEFAULT_BOOTS,
            support_items_pool: &APHELIOS_DEFAULT_SUPPORT_ITEMS,
        },
    };
}
