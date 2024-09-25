use crate::game_data::{items_data::items::*, units_data::*};

//champion parameters (constants):
const ASHE_Q_MAX_STACKS: u8 = 4;
const ASHE_W_N_TARGETS: f32 = 1.2;

fn ashe_init_abilities(champ: &mut Unit) {
    champ.effects_stacks[EffectStackId::AsheFrosted] = 0;

    champ.effects_stacks[EffectStackId::AsheFocusStacks] = 0;
    champ.effects_values[EffectValueId::AsheRangersFocusBonusAS] = 0.;
}

pub fn ashe_basic_attack(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    //check if target is frosted
    let special_crit_coef: f32 = if champ.effects_stacks[EffectStackId::AsheFrosted] == 1 {
        1.15 + champ.stats.crit_chance * (0.75 + champ.stats.crit_dmg - Unit::BASE_CRIT_DMG)
    } else {
        champ.effects_stacks[EffectStackId::AsheFrosted] = 1; //apply frost, assumes slow never expires on target (real duration is 2s on patch 14.14)
        1.
    };
    //check if q buff is active (max stacks + 1 indicates that q buff is active)
    if champ.effects_stacks[EffectStackId::AsheFocusStacks] == ASHE_Q_MAX_STACKS + 1 {
        let phys_dmg: f32 = ASHE_Q_AD_RATIO_BY_Q_LVL[usize::from(champ.q_lvl - 1)]
            * champ.stats.ad()
            * special_crit_coef;
        champ.dmg_on_target(
            target_stats,
            (phys_dmg, 0., 0.),
            (5, 1),
            DmgType::Other,
            true,
            1.,
        )
    } else {
        //add focus stack if not maxed
        if champ.effects_stacks[EffectStackId::AsheFocusStacks] < ASHE_Q_MAX_STACKS {
            champ.effects_stacks[EffectStackId::AsheFocusStacks] += 1;
        }
        let phys_dmg: f32 = champ.stats.ad() * special_crit_coef;
        champ.dmg_on_target(
            target_stats,
            (phys_dmg, 0., 0.),
            (1, 1),
            DmgType::Other,
            true,
            1.,
        )
    }
}

const ASHE_Q_BONUS_AS_BY_Q_LVL: [f32; 5] = [0.25, 0.325, 0.40, 0.475, 0.55];
const ASHE_Q_AD_RATIO_BY_Q_LVL: [f32; 5] = [1.05, 1.10, 1.15, 1.20, 1.25];

fn ashe_rangers_focus_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_values[EffectValueId::AsheRangersFocusBonusAS] == 0. {
        champ.effects_stacks[EffectStackId::AsheFocusStacks] = ASHE_Q_MAX_STACKS + 1; //max stacks + 1 indicates that q buff is active
        let bonus_as_buff: f32 = ASHE_Q_BONUS_AS_BY_Q_LVL[usize::from(champ.q_lvl - 1)];
        champ.stats.bonus_as += bonus_as_buff;
        champ.effects_values[EffectValueId::AsheRangersFocusBonusAS] = bonus_as_buff;
    }
}

fn ashe_rangers_focus_disable(champ: &mut Unit) {
    champ.stats.bonus_as -= champ.effects_values[EffectValueId::AsheRangersFocusBonusAS];
    champ.effects_values[EffectValueId::AsheRangersFocusBonusAS] = 0.;
    champ.effects_stacks[EffectStackId::AsheFocusStacks] = 0;
}

const ASHE_RANGERS_FOCUS_BUFF: TemporaryEffect = TemporaryEffect {
    id: EffectId::AsheRangersFocus,
    add_stack: ashe_rangers_focus_enable,
    remove_every_stack: ashe_rangers_focus_disable,
    duration: 6.,
    cooldown: 0.,
};

fn ashe_q(champ: &mut Unit, _target_stats: &UnitStats) -> f32 {
    champ.add_temporary_effect(&ASHE_RANGERS_FOCUS_BUFF, 0.);
    champ.basic_attack_cd = 0.; //q resets basic attack cd
    0.
}

const ASHE_W_PHYS_DMG_BY_W_LVL: [f32; 5] = [20., 35., 50., 65., 80.];

fn ashe_w(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let w_lvl_idx: usize = usize::from(champ.w_lvl - 1); //to index ability ratios by lvl

    let phys_dmg: f32 = ASHE_W_N_TARGETS * (ASHE_W_PHYS_DMG_BY_W_LVL[w_lvl_idx] + champ.stats.ad());

    champ.effects_stacks[EffectStackId::AsheFrosted] = 1; //apply frost
    champ.dmg_on_target(
        target_stats,
        (phys_dmg, 0., 0.),
        (1, 1),
        DmgType::Ability,
        false,
        ASHE_W_N_TARGETS,
    )
}

fn ashe_e(_champ: &mut Unit, _target_stats: &UnitStats) -> f32 {
    //bird does nothing
    0.
}

const ASHE_R_MAGIC_DMG_BY_R_LVL: [f32; 3] = [200., 400., 600.];

fn ashe_r(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let r_lvl_idx: usize = usize::from(champ.r_lvl - 1); //to index ability ratios by lvl

    let magic_dmg: f32 = ASHE_R_MAGIC_DMG_BY_R_LVL[r_lvl_idx] + 1.20 * champ.stats.ap();

    champ.effects_stacks[EffectStackId::AsheFrosted] = 1; //apply frost
    champ.dmg_on_target(
        target_stats,
        (0., magic_dmg, 0.),
        (1, 1),
        DmgType::Ultimate,
        false,
        1.,
    )
}

fn ashe_fight_scenario(champ: &mut Unit, target_stats: &UnitStats, fight_duration: f32) {
    //add weighted r dmg at the beggining
    champ.weighted_r(target_stats);

    while champ.time < fight_duration {
        //priority order: w, q, basic attack
        if champ.w_cd == 0. {
            champ.w(target_stats);
        } else if champ.effects_stacks[EffectStackId::AsheFocusStacks] == ASHE_Q_MAX_STACKS {
            //ashe q has no cd
            champ.q(target_stats);
            //basic attack directly after
            //ashe q resets cooldown (patch 14.17) but we keep that here for consistency in case it gets changed
            if champ.basic_attack_cd != 0. {
                champ.walk(champ.basic_attack_cd + F32_TOL);
            }
            champ.basic_attack(target_stats);
        } else if champ.basic_attack_cd == 0. {
            champ.basic_attack(target_stats);
        } else {
            champ.walk(
                F32_TOL
                    + [
                        //ashe q has no cd
                        champ.w_cd,
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

const ASHE_DEFAULT_RUNES_PAGE: RunesPage = RunesPage {
    shard1: RuneShard::Middle,
    shard2: RuneShard::Left,
    shard3: RuneShard::Left,
};

const ASHE_DEFAULT_SKILL_ORDER: SkillOrder = SkillOrder {
    //lvls:
    //  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18
    w: [1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    q: [0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0],
    e: [0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1],
    r: [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0],
};

const ASHE_DEFAULT_LEGENDARY_ITEMS: [&Item; 41] = [
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
    //&OPPORTUNITY,
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
    //&YOUMUUS_GHOSTBLADE,
    &YUN_TAL_WILDARROWS,
    //&ZHONYAS_HOURGLASS,
];

const ASHE_DEFAULT_BOOTS: [&Item; 2] = [
    &BERSERKERS_GREAVES,
    &BOOTS_OF_SWIFTNESS,
    //&IONIAN_BOOTS_OF_LUCIDITY,
    //&MERCURYS_TREADS,
    //&PLATED_STEELCAPS,
    //&SORCERERS_SHOES,
];

const ASHE_DEFAULT_SUPPORT_ITEMS: [&Item; 0] = [];

const ASHE_BASE_AS: f32 = 0.658;
impl Unit {
    pub const ASHE_PROPERTIES: UnitProperties = UnitProperties {
        name: "Ashe",
        as_limit: Unit::DEFAULT_AS_LIMIT,
        as_ratio: ASHE_BASE_AS, //if not specified, same as base AS
        windup_percent: 0.2193,
        windup_modifier: 1., //get it from https://leagueoflegends.fandom.com/wiki/List_of_champions/Basic_attacks, 1 by default
        base_stats: UnitStats {
            hp: 610.,
            mana: 280.,
            base_ad: 59.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 26.,
            mr: 30.,
            base_as: ASHE_BASE_AS,
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
            hp: 101.,
            mana: 35.,
            base_ad: 2.95,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 4.6,
            mr: 1.3,
            base_as: 0.,
            bonus_as: 0.0333,
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
        on_lvl_set: None,
        init_abilities: Some(ashe_init_abilities),
        basic_attack: ashe_basic_attack,
        q: BasicAbility {
            cast: ashe_q,
            cast_time: F32_TOL,
            base_cooldown_by_ability_lvl: [F32_TOL, F32_TOL, F32_TOL, F32_TOL, F32_TOL, F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        w: BasicAbility {
            cast: ashe_w,
            cast_time: 0.25,
            base_cooldown_by_ability_lvl: [18., 14.5, 11., 7.5, 4., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        e: BasicAbility {
            cast: ashe_e,
            cast_time: 0.25,
            base_cooldown_by_ability_lvl: [90., 80., 70., 60., 50., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        r: UltimateAbility {
            cast: ashe_r,
            cast_time: 0.25,
            base_cooldown_by_ability_lvl: [100., 80., 60.],
        },
        fight_scenarios: &[(ashe_fight_scenario, "all out")],
        unit_defaults: UnitDefaults {
            runes_pages: &ASHE_DEFAULT_RUNES_PAGE,
            skill_order: &ASHE_DEFAULT_SKILL_ORDER,
            legendary_items_pool: &ASHE_DEFAULT_LEGENDARY_ITEMS,
            boots_pool: &ASHE_DEFAULT_BOOTS,
            support_items_pool: &ASHE_DEFAULT_SUPPORT_ITEMS,
        },
    };
}
