use crate::game_data::{items_data::items::*, units_data::*};

//champion parameters (constants):
/// 1 proc = 2 basic attacks (gains 1 proc per activation).
const LUCIAN_N_VIGILANCE_PROCS: u8 = 1;
/// Percentage of the r that hits its target, must be between 0. and 1.
const LUCIAN_R_HIT_PERCENT: f32 = 0.75;

fn lucian_init_spells(champ: &mut Unit) {
    champ.effects_stacks[EffectStackId::LucianLightslingerEmpowered] = 0;
    champ.effects_stacks[EffectStackId::LucianVigilanceProcsRemaning] = LUCIAN_N_VIGILANCE_PROCS;
    champ.effects_values[EffectValueId::LucianArdentBlazeMsFlat] = 0.;
}

const LUCIAN_LIGHTSLINGER_BASIC_ATTACKS_AD_RATIO_BY_LVL: [f32; MAX_UNIT_LVL] = [
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

fn lucian_basic_attack(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    if champ.effects_stacks[EffectStackId::LucianLightslingerEmpowered] == 1 {
        champ.effects_stacks[EffectStackId::LucianLightslingerEmpowered] = 0;
        champ.e_cd = f32::max(0., champ.e_cd - 4.); //double basic attack reduce e_cd by 2sec for each hit

        //vigilance passive
        let ap_dmg: f32 = if champ.effects_stacks[EffectStackId::LucianVigilanceProcsRemaning] != 0
        {
            champ.effects_stacks[EffectStackId::LucianVigilanceProcsRemaning] -= 1;
            2. * (15. + 0.20 * champ.stats.ad())
        } else {
            0.
        };

        let ad_dmg: f32 = (1.
            + LUCIAN_LIGHTSLINGER_BASIC_ATTACKS_AD_RATIO_BY_LVL[usize::from(champ.lvl.get() - 1)])
            * champ.stats.ad()
            * champ.stats.crit_coef();
        champ.dmg_on_target(
            target_stats,
            (ad_dmg, ap_dmg, 0.),
            (2, 2),
            DmgType::Other,
            true,
            1.,
        )
    } else {
        Unit::default_basic_attack(champ, target_stats)
    }
}

const LUCIAN_Q_AD_DMG_BY_Q_LVL: [f32; 5] = [85., 115., 145., 175., 205.];
const LUCIAN_Q_BONUS_AD_RATIO_BY_Q_LVL: [f32; 5] = [0.60, 0.75, 0.90, 1.05, 1.20];
fn lucian_q(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let q_lvl_idx: usize = usize::from(champ.q_lvl - 1); //to index spell ratios by lvl
    let ad_dmg: f32 = LUCIAN_Q_AD_DMG_BY_Q_LVL[q_lvl_idx]
        + LUCIAN_Q_BONUS_AD_RATIO_BY_Q_LVL[q_lvl_idx] * champ.stats.bonus_ad;

    champ.effects_stacks[EffectStackId::LucianLightslingerEmpowered] = 1;

    champ.dmg_on_target(
        target_stats,
        (ad_dmg, 0., 0.),
        (1, 1),
        DmgType::Ability,
        false,
        1.,
    )
}

const LUCIAN_ARDENT_BLAZE_MS_BY_W_LVL: [f32; 5] = [60., 65., 70., 75., 80.];

fn lucian_ardent_blaze_ms_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_values[EffectValueId::LucianArdentBlazeMsFlat] == 0. {
        let flat_ms_buff: f32 = LUCIAN_ARDENT_BLAZE_MS_BY_W_LVL[usize::from(champ.w_lvl - 1)];
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

const LUCIAN_W_AP_DMG_BY_W_LVL: [f32; 5] = [75., 110., 145., 180., 215.];
fn lucian_w(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let w_lvl_idx: usize = usize::from(champ.w_lvl - 1); //to index spell ratios by lvl
    let ap_dmg: f32 = LUCIAN_W_AP_DMG_BY_W_LVL[w_lvl_idx] + 0.9 * champ.stats.ap();

    champ.effects_stacks[EffectStackId::LucianLightslingerEmpowered] = 1;

    //for simplicity we apply the ms buff directly
    champ.add_temporary_effect(&LUCIAN_ARDENT_BLAZE_MS, 0.);

    champ.dmg_on_target(
        target_stats,
        (0., ap_dmg, 0.),
        (1, 1),
        DmgType::Ability,
        false,
        1.,
    )
}

fn lucian_e(champ: &mut Unit, _target_stats: &UnitStats) -> f32 {
    champ.sim_results.units_travelled += 425.; //maximum dash range
    champ.effects_stacks[EffectStackId::LucianLightslingerEmpowered] = 1;
    0.
}

const LUCIAN_R_AD_DMG_BY_R_LVL: [f32; 3] = [15., 30., 45.]; //dmg on champions
fn lucian_r(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let r_lvl_idx: usize = usize::from(champ.r_lvl - 1); //to index spell ratios by lvl

    let n_hits: f32 = LUCIAN_R_HIT_PERCENT * (22. + champ.stats.crit_chance / 0.04);
    let ad_dmg: f32 = n_hits
        * (LUCIAN_R_AD_DMG_BY_R_LVL[r_lvl_idx] + 0.25 * champ.stats.ad() + 0.15 * champ.stats.ap());

    champ.effects_stacks[EffectStackId::LucianLightslingerEmpowered] = 1;

    let r_dmg: f32 = champ.dmg_on_target(
        target_stats,
        (ad_dmg, 0., 0.),
        (n_hits as u8, 1),
        DmgType::Ultimate,
        false,
        1.,
    );
    champ.walk(LUCIAN_R_HIT_PERCENT * 3.);
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
            //wait for the basic basic_attackattack cooldown if there is one
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

const LUCIAN_DEFAULT_RUNES_PAGE: RunesPage = RunesPage {
    shard1: RuneShard::Middle,
    shard2: RuneShard::Left,
    shard3: RuneShard::Left,
};

const LUCIAN_DEFAULT_SKILL_ORDER: SkillOrder = SkillOrder {
    //lvls:
    //  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18
    q: [1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    e: [0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0],
    w: [0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1],
    r: [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0],
};

const LUCIAN_DEFAULT_LEGENDARY_ITEMS: [&Item; 43] = [
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
    &ICEBORN_GAUNTLET,
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
];

const LUCIAN_DEFAULT_BOOTS: [&Item; 3] = [
    &BERSERKERS_GREAVES,
    &BOOTS_OF_SWIFTNESS,
    &IONIAN_BOOTS_OF_LUCIDITY,
    //&MERCURYS_TREADS,
    //&PLATED_STEELCAPS,
    //&SORCERERS_SHOES,
];

const LUCIAN_DEFAULT_SUPPORT_ITEMS: [&Item; 0] = [];

const LUCIAN_BASE_AS: f32 = 0.638;
impl Unit {
    pub const LUCIAN_PROPERTIES: UnitProperties = UnitProperties {
        name: "Lucian",
        as_limit: Unit::DEFAULT_AS_LIMIT,
        as_ratio: LUCIAN_BASE_AS, //if not specified, same as base AS
        windup_percent: 0.15,
        windup_modifier: 1., //get it from https://leagueoflegends.fandom.com/wiki/List_of_champions/Basic_attacks, 1 by default
        base_stats: UnitStats {
            hp: 641.,
            mana: 320.,
            base_ad: 60.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_coef: 0.,
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
            ap_coef: 0.,
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
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_lvl_set: None,
        init_abilities: Some(lucian_init_spells),
        basic_attack: lucian_basic_attack,
        q: BasicAbility {
            cast: lucian_q,
            cast_time: 0.33, //average between 0.4 and 0,25
            base_cooldown_by_ability_lvl: [9., 8., 7., 6., 5., F32_TOL], //basic spells only uses the first 5 values (except for aphelios)
        },
        w: BasicAbility {
            cast: lucian_w,
            cast_time: 0.25,
            base_cooldown_by_ability_lvl: [14., 13., 12., 11., 10., F32_TOL], //basic spells only uses the first 5 values (except for aphelios)
        },
        e: BasicAbility {
            cast: lucian_e,
            cast_time: F32_TOL,
            base_cooldown_by_ability_lvl: [19., 17.75, 16.5, 15.25, 14., F32_TOL], //basic spells only uses the first 5 values (except for aphelios)
        },
        r: UltimateAbility {
            cast: lucian_r,
            cast_time: F32_TOL,
            base_cooldown_by_ability_lvl: [110., 100., 90.],
        },
        fight_scenarios: &[
            (lucian_fight_scenario_all_out, "all out"),
            (lucian_fight_scenario_poke, "poke"),
        ],
        unit_defaults: UnitDefaults {
            runes_pages: &LUCIAN_DEFAULT_RUNES_PAGE,
            skill_order: &LUCIAN_DEFAULT_SKILL_ORDER,
            legendary_items_pool: &LUCIAN_DEFAULT_LEGENDARY_ITEMS,
            boots_pool: &LUCIAN_DEFAULT_BOOTS,
            support_items_pool: &LUCIAN_DEFAULT_SUPPORT_ITEMS,
        },
    };
}
