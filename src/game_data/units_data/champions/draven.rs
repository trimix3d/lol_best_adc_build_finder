use crate::game_data::{items_data::items::*, units_data::*};

//champion parameters (constants):
/// Number of basic attacks to be performed before pressing w again, must be at least 1.
const DRAVEN_BASIC_ATTACKS_PER_W: u8 = 2;
const DRAVEN_R_N_TARGETS: f32 = 1.;
/// Percentage of the time the q return hit its targets.
const DRAVEN_R_RETURN_PERCENT: f32 = 0.75;

fn draven_init_spells(champ: &mut Unit) {
    champ.effects_stacks[EffectStackId::DravenAxesInHand] = 0;
    champ.effects_stacks[EffectStackId::DravenAxesInAir] = 0;
    champ.effects_values[EffectValueId::DravenBloodRushBonusAS] = 0.;
    champ.effects_values[EffectValueId::DravenBloodRushBonusMsPercent] = 0.;
}

const DRAVEN_AXE_TIME_SPENT_IN_AIR: f32 = 2.; //axe travel time before hitting the target not accounted in this duration

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
    duration: DRAVEN_AXE_TIME_SPENT_IN_AIR,
    cooldown: 0.,
};

//effect for axe n2
const DRAVEN_THROW_AXE2: TemporaryEffect = TemporaryEffect {
    id: EffectId::DravenThrowAxe2,
    add_stack: draven_throw_axe,
    remove_every_stack: draven_catch_axe, //effect assumes draven catches every axe
    duration: DRAVEN_AXE_TIME_SPENT_IN_AIR,
    cooldown: 0.,
};

const DRAVEN_SPINNING_AXE_BASE_DMG_BY_Q_LVL: [f32; 5] = [40., 45., 50., 55., 60.];
const DRAVEN_SPINNING_AXE_BONUS_AD_RATIO_BY_Q_LVL: [f32; 5] = [0.75, 0.85, 0.95, 1.05, 1.15];

fn draven_q_axe_bonus_dmg(champ: &Unit) -> f32 {
    let q_lvl_idx: usize = usize::from(champ.q_lvl - 1);
    DRAVEN_SPINNING_AXE_BASE_DMG_BY_Q_LVL[q_lvl_idx]
        + champ.stats.bonus_ad * DRAVEN_SPINNING_AXE_BONUS_AD_RATIO_BY_Q_LVL[q_lvl_idx]
}

fn draven_basic_attack(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let mut ad_dmg: f32 = champ.stats.ad() * champ.stats.crit_coef();

    if champ.effects_stacks[EffectStackId::DravenAxesInHand] >= 1
        && champ.effects_stacks[EffectStackId::DravenAxesInAir] < 2
    {
        //this code only supports 2 axes in the air maximum, but its fine for most cases anyway
        if !champ
            .temporary_effects_durations
            .contains_key(&DRAVEN_THROW_AXE1)
        {
            champ.add_temporary_effect(&DRAVEN_THROW_AXE1, 0.);
            ad_dmg += draven_q_axe_bonus_dmg(champ);
        } else if !champ
            .temporary_effects_durations
            .contains_key(&DRAVEN_THROW_AXE2)
        {
            champ.add_temporary_effect(&DRAVEN_THROW_AXE2, 0.);
            ad_dmg += draven_q_axe_bonus_dmg(champ);
        }
    }

    champ.dmg_on_target(
        target_stats,
        (ad_dmg, 0., 0.),
        (1, 1),
        DmgSource::Other,
        true,
        1.,
    )
}

fn draven_q(champ: &mut Unit, _target_stats: &UnitStats) -> f32 {
    champ.effects_stacks[EffectStackId::DravenAxesInHand] =
        u8::min(2, champ.effects_stacks[EffectStackId::DravenAxesInHand] + 1);
    0.
}

const DRAVEN_BLOOD_RUSH_BONUS_AS_BY_W_LVL: [f32; 5] = [0.20, 0.25, 0.30, 0.35, 0.40];
const DRAVEN_BLOOD_RUSH_MS_PERCENT_BY_W_LVL: [f32; 5] =
    [0.50 / 2., 0.55 / 2., 0.60 / 2., 0.65 / 2., 0.70 / 2.]; //halved because decaying effect

fn draven_blood_rush_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_values[EffectValueId::DravenBloodRushBonusAS] == 0. {
        let w_lvl_idx: usize = usize::from(champ.w_lvl - 1);
        let bonus_as: f32 = DRAVEN_BLOOD_RUSH_BONUS_AS_BY_W_LVL[w_lvl_idx];
        let ms_percent: f32 = DRAVEN_BLOOD_RUSH_MS_PERCENT_BY_W_LVL[w_lvl_idx];
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

fn draven_w(champ: &mut Unit, _target_stats: &UnitStats) -> f32 {
    champ.add_temporary_effect(&DRAVEN_BLOOD_RUSH, 0.);
    0.
}

const DRAVEN_E_BASE_DMG_BY_E_LVL: [f32; 5] = [75., 110., 145., 180., 215.];

fn draven_e(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let e_lvl_idx: usize = usize::from(champ.e_lvl - 1); //to index spell ratios by lvl

    let ad_dmg: f32 = DRAVEN_E_BASE_DMG_BY_E_LVL[e_lvl_idx] + 0.5 * champ.stats.bonus_ad;

    champ.dmg_on_target(
        target_stats,
        (ad_dmg, 0., 0.),
        (1, 1),
        DmgSource::BasicSpell,
        false,
        1.,
    )
}

const DRAVEN_R_BASE_DMG_BY_R_LVL: [f32; 3] = [175., 275., 375.];
const DRAVEN_R_BONUS_AD_RATIO_BY_R_LVL: [f32; 3] = [1.10, 1.30, 1.50];

fn draven_r(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let r_lvl_idx: usize = usize::from(champ.r_lvl - 1); //to index spell ratios by lvl

    let ad_dmg: f32 = DRAVEN_R_N_TARGETS
        * (1. + DRAVEN_R_RETURN_PERCENT)
        * (DRAVEN_R_BASE_DMG_BY_R_LVL[r_lvl_idx]
            + champ.stats.bonus_ad * DRAVEN_R_BONUS_AD_RATIO_BY_R_LVL[r_lvl_idx]);

    champ.dmg_on_target(
        target_stats,
        (ad_dmg, 0., 0.),
        ((1. + DRAVEN_R_RETURN_PERCENT) as u8, 1),
        DmgSource::UltimateSpell,
        false,
        DRAVEN_R_N_TARGETS,
    )
}

fn draven_fight_scenario(champ: &mut Unit, target_stats: &UnitStats, fight_duration: f32) {
    //e once at the begginning
    champ.e(target_stats);

    let mut basic_attacks_count: u8 = DRAVEN_BASIC_ATTACKS_PER_W - 1;
    while champ.time < fight_duration {
        //priority order: q before basic attacking if less than 2 axes in hand, basic attack if at least one axe and less than 2 axes in air, w every x basic attack
        if champ.basic_attack_cd == 0. && champ.effects_stacks[EffectStackId::DravenAxesInAir] < 2 {
            //q before launching basic attack if available
            if champ.q_cd == 0. && champ.effects_stacks[EffectStackId::DravenAxesInHand] < 2 {
                champ.q(target_stats);
            }
            champ.basic_attack(target_stats);
            basic_attacks_count += 1;
        } else if champ.w_cd == 0. && basic_attacks_count >= DRAVEN_BASIC_ATTACKS_PER_W {
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
                        if basic_attacks_count >= DRAVEN_BASIC_ATTACKS_PER_W {
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

const DRAVEN_DEFAULT_RUNES_PAGE: RunesPage = RunesPage {
    shard1: RuneShard::Left,
    shard2: RuneShard::Left,
    shard3: RuneShard::Left,
};

const DRAVEN_DEFAULT_SKILL_ORDER: SkillOrder = SkillOrder {
    //lvls:
    //  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18
    q: [1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    w: [0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0],
    e: [0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1],
    r: [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0],
};

const DRAVEN_DEFAULT_LEGENDARY_ITEMS: [&Item; 40] = [
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
    //&RUNAANS_HURRICANE,
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

const DRAVEN_DEFAULT_BOOTS: [&Item; 2] = [
    &BERSERKERS_GREAVES,
    &BOOTS_OF_SWIFTNESS,
    //&IONIAN_BOOTS_OF_LUCIDITY,
    //&MERCURYS_TREADS,
    //&PLATED_STEELCAPS,
    //&SORCERERS_SHOES,
];

const DRAVEN_DEFAULT_SUPPORT_ITEMS: [&Item; 0] = [];

const DRAVEN_BASE_AS: f32 = 0.679;
impl Unit {
    pub const DRAVEN_PROPERTIES_REF: &UnitProperties = &UnitProperties {
        name: "Draven",
        as_limit: Unit::DEFAULT_AS_LIMIT,
        as_ratio: DRAVEN_BASE_AS, //if not specified, same as base AS
        windup_percent: 0.15614,
        windup_modifier: 1., //get it from https://leagueoflegends.fandom.com/wiki/List_of_champions/Basic_attacks, 1 by default
        base_stats: UnitStats {
            hp: 675.,
            mana: 361.,
            base_ad: 62.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_coef: 0.,
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
        },
        growth_stats: UnitStats {
            hp: 104.,
            mana: 39.,
            base_ad: 3.6,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_coef: 0.,
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
        },
        on_lvl_set: None,
        init_unit: Some(draven_init_spells),
        basic_attack: draven_basic_attack,
        q: BasicSpell {
            cast: draven_q,
            cast_time: F32_TOL,
            base_cooldown_by_spell_lvl: [12., 11., 10., 9., 8., F32_TOL], //basic spells only uses the first 5 values (except for aphelios)
        },
        w: BasicSpell {
            cast: draven_w,
            cast_time: F32_TOL,
            base_cooldown_by_spell_lvl: [12., 12., 12., 12., 12., F32_TOL], //basic spells only uses the first 5 values (except for aphelios)
        },
        e: BasicSpell {
            cast: draven_e,
            cast_time: 0.25,
            base_cooldown_by_spell_lvl: [18., 17., 16., 15., 14., F32_TOL], //basic spells only uses the first 5 values (except for aphelios)
        },
        r: UltimateSpell {
            cast: draven_r,
            cast_time: 0.5,
            base_cooldown_by_spell_lvl: [100., 90., 80.],
        },
        on_trigger_event: OnTriggerEvent {
            on_fight_init: vec![],
            special_active: vec![],
            on_basic_spell_cast: vec![],
            on_ultimate_cast: vec![],
            on_basic_spell_hit: vec![],
            on_ultimate_spell_hit: vec![],
            spell_coef: vec![],
            on_basic_attack_hit_static: vec![],
            on_basic_attack_hit_dynamic: vec![],
            on_any_hit: vec![],
            on_ad_hit: vec![],
            ap_true_dmg_coef: vec![],
            tot_dmg_coef: vec![],
        },
        fight_scenarios: &[
            (draven_fight_scenario_start_with_one_axe, "start with 1 axe"),
            (draven_fight_scenario, "start with no axe"),
        ],
        unit_defaults: UnitDefaults {
            runes_pages: &DRAVEN_DEFAULT_RUNES_PAGE,
            skill_order: &DRAVEN_DEFAULT_SKILL_ORDER,
            legendary_items_pool: &DRAVEN_DEFAULT_LEGENDARY_ITEMS,
            boots_pool: &DRAVEN_DEFAULT_BOOTS,
            support_items_pool: &DRAVEN_DEFAULT_SUPPORT_ITEMS,
        },
    };
}
