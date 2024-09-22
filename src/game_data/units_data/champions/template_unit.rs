use crate::game_data::{items_data::items::*, units_data::*};

//champion parameters (constants):
//todo

fn template_unit_on_lvl_set(champ: &mut Unit) {
    //todo
}

fn template_unit_init_spells(champ: &mut Unit) {
    //todo
}

fn template_unit_q(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let q_lvl_idx: usize = usize::from(champ.q_lvl - 1); //to index spell ratios by lvl

    //todo
    let ad_dmg: f32 = 0.;
    let ap_dmg: f32 = 0.;
    let true_dmg: f32 = 0.;

    champ.dmg_on_target(
        target_stats,
        (ad_dmg, ap_dmg, true_dmg),
        (1, 1),
        DmgSource::BasicSpell,
        false,
        1.,
    )
}

fn template_unit_w(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let w_lvl_idx: usize = usize::from(champ.w_lvl - 1); //to index spell ratios by lvl

    //todo
    let ad_dmg: f32 = 0.;
    let ap_dmg: f32 = 0.;
    let true_dmg: f32 = 0.;

    champ.dmg_on_target(
        target_stats,
        (ad_dmg, ap_dmg, true_dmg),
        (1, 1),
        DmgSource::BasicSpell,
        false,
        1.,
    )
}

fn template_unit_e(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let e_lvl_idx: usize = usize::from(champ.e_lvl - 1); //to index spell ratios by lvl

    //todo
    let ad_dmg: f32 = 0.;
    let ap_dmg: f32 = 0.;
    let true_dmg: f32 = 0.;

    champ.dmg_on_target(
        target_stats,
        (ad_dmg, ap_dmg, true_dmg),
        (1, 1),
        DmgSource::BasicSpell,
        false,
        1.,
    )
}

fn template_unit_r(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let r_lvl_idx: usize = usize::from(champ.r_lvl - 1); //to index spell ratios by lvl

    //todo
    let ad_dmg: f32 = 0.;
    let ap_dmg: f32 = 0.;
    let true_dmg: f32 = 0.;

    champ.dmg_on_target(
        target_stats,
        (ad_dmg, ap_dmg, true_dmg),
        (1, 1),
        DmgSource::UltimateSpell,
        false,
        1.,
    )
}

fn template_unit_fight_scenario(champ: &mut Unit, target_stats: &UnitStats, fight_duration: f32) {
    //todo
    while champ.time < fight_duration {
        //priority order: q, w, e, basic attack
        if champ.q_cd == 0. {
            champ.q(target_stats);
        } else if champ.w_cd == 0. {
            champ.w(target_stats);
        } else if champ.e_cd == 0. {
            champ.e(target_stats);
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

const TEMPLATE_UNIT_DEFAULT_RUNES_PAGE: RunesPage = RunesPage {
    //todo
    shard1: RuneShard::Middle,
    shard2: RuneShard::Left,
    shard3: RuneShard::Left,
};

const TEMPLATE_UNIT_DEFAULT_SKILL_ORDER: SkillOrder = SkillOrder {
    //todo
    //lvls:
    //  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18
    q: [1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    w: [0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0],
    e: [0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1],
    r: [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0],
};

//todo
const TEMPLATE_UNIT_DEFAULT_LEGENDARY_ITEMS: [&Item; 74] = [
    &ABYSSAL_MASK,
    &AXIOM_ARC,
    &BANSHEES_VEIL,
    &BLACK_CLEAVER,
    &BLACKFIRE_TORCH,
    &BLADE_OF_THE_RUINED_KING,
    &BLOODTHIRSTER,
    &CHEMPUNK_CHAINSWORD,
    &COSMIC_DRIVE,
    &CRYPTBLOOM,
    &DEAD_MANS_PLATE,
    &DEATHS_DANCE,
    &ECLIPSE,
    &EDGE_OF_NIGHT,
    &ESSENCE_REAVER,
    &EXPERIMENTAL_HEXPLATE,
    &FROZEN_HEART,
    &GUARDIAN_ANGEL,
    &GUINSOOS_RAGEBLADE,
    &HEXTECH_ROCKETBELT,
    &HORIZON_FOCUS,
    &HUBRIS,
    &HULLBREAKER,
    &ICEBORN_GAUNTLET,
    &IMMORTAL_SHIELDBOW,
    &INFINITY_EDGE,
    &JAKSHO,
    &KAENIC_ROOKERN,
    &KRAKEN_SLAYER,
    &LIANDRYS_TORMENT,
    &LICH_BANE,
    &LORD_DOMINIKS_REGARDS,
    &LUDENS_COMPANION,
    &MALIGNANCE,
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
    &PROFANE_HYDRA,
    &RABADONS_DEATHCAP,
    &RANDUINS_OMEN,
    &RAPID_FIRECANNON,
    &RAVENOUS_HYDRA,
    &RIFTMAKER,
    &ROD_OF_AGES,
    &RUNAANS_HURRICANE,
    &RYLAIS_CRYSTAL_SCEPTER,
    &SERAPHS_EMBRACE,
    &SERPENTS_FANG,
    &SERYLDAS_GRUDGE,
    &SHADOWFLAME,
    &SPEAR_OF_SHOJIN,
    &STATIKK_SHIV,
    &STERAKS_GAGE,
    &STORMSURGE,
    &STRIDEBREAKER,
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
];

//todo
const TEMPLATE_UNIT_DEFAULT_BOOTS: [&Item; 6] = [
    &BERSERKERS_GREAVES,
    &BOOTS_OF_SWIFTNESS,
    &IONIAN_BOOTS_OF_LUCIDITY,
    &MERCURYS_TREADS,
    &PLATED_STEELCAPS,
    &SORCERERS_SHOES,
];

//todo
const TEMPLATE_UNIT_DEFAULT_SUPPORT_ITEMS: [&Item; 0] = [];

const TEMPLATE_UNIT_BASE_AS: f32 = 0;
impl Unit {
    pub const TEMPLATE_UNIT_PROPERTIES_REF: &UnitProperties = &UnitProperties {
        name: "Template_unit",            //todo
        as_limit: Unit::DEFAULT_AS_LIMIT, //todo
        as_ratio: TEMPLATE_UNIT_BASE_AS,  //todo, if not specified, same as base AS
        windup_percent: 0,
        windup_modifier: 1, //todo, get it from https://leagueoflegends.fandom.com/wiki/List_of_champions/Basic_attacks, 1 by default
        base_stats: UnitStats {
            hp: 0,
            mana: 0,
            base_ad: 0,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_coef: 0.,
            armor: 0,
            mr: 0,
            base_as: TEMPLATE_UNIT_BASE_AS,
            bonus_as: 0.,
            ability_haste: 0.,
            basic_haste: 0.,
            ultimate_haste: 0.,
            item_haste: 0.,
            crit_chance: 0.,
            crit_dmg: Unit::BASE_CRIT_DMG,
            ms_flat: 0,
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
            hp: 0,
            mana: 0,
            base_ad: 0,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_coef: 0.,
            armor: 0,
            mr: 0,
            base_as: 0.,
            bonus_as: 0,
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
        on_lvl_set: Some(template_unit_on_lvl_set),
        init_spells: Some(template_unit_init_spells),
        basic_attack: Unit::default_basic_attack, //todo
        q: Spell {
            cast: template_unit_q,
            cast_time: 0,
            base_cooldown_by_spell_lvl: [0, 0, 0, 0, 0, F32_TOL], //basic spells only uses the first 5 values (except for aphelios)
        },
        w: Spell {
            cast: template_unit_w,
            cast_time: 0,
            base_cooldown_by_spell_lvl: [0, 0, 0, 0, 0, F32_TOL], //basic spells only uses the first 5 values (except for aphelios)
        },
        e: Spell {
            cast: template_unit_e,
            cast_time: 0,
            base_cooldown_by_spell_lvl: [0, 0, 0, 0, 0, F32_TOL], //basic spells only uses the first 5 values (except for aphelios)
        },
        r: Spell {
            cast: template_unit_r,
            cast_time: 0,
            base_cooldown_by_spell_lvl: [0, 0, 0, F32_TOL, F32_TOL, F32_TOL], //ultimate only uses the first 3 values
        },
        fight_scenarios: &[(template_unit_fight_scenario, "todo")],
        unit_defaults: UnitDefaults {
            runes_pages: &TEMPLATE_UNIT_DEFAULT_RUNES_PAGE,
            skill_order: &TEMPLATE_UNIT_DEFAULT_SKILL_ORDER,
            legendary_items_pool: &TEMPLATE_UNIT_DEFAULT_LEGENDARY_ITEMS,
            boots_pool: &TEMPLATE_UNIT_DEFAULT_BOOTS,
            support_items_pool: &TEMPLATE_UNIT_DEFAULT_SUPPORT_ITEMS,
        },
    };
}
