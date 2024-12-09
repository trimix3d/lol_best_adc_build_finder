use crate::game_data::*;

use items_data::Item;
use runes_data::*;
use units_data::*;

use enumset::enum_set;

//champion parameters (constants):
//todo

fn template_init_abilities(champ: &mut Unit) {
    //todo
}

fn template_q(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    //todo
    //let q_lvl_idx: usize = usize::from(champ.q_lvl - 1); //to index ability ratios by lvl
    PartDmg(0., 0., 0.)
}

fn template_w(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    //todo
    //let w_lvl_idx: usize = usize::from(champ.w_lvl - 1); //to index ability ratios by lvl
    PartDmg(0., 0., 0.)
}

fn template_e(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    //todo
    //let e_lvl_idx: usize = usize::from(champ.e_lvl - 1); //to index ability ratios by lvl
    PartDmg(0., 0., 0.)
}

fn template_r(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    //todo
    //let r_lvl_idx: usize = usize::from(champ.r_lvl - 1); //to index ability ratios by lvl
    PartDmg(0., 0., 0.)
}

fn template_fight_scenario(champ: &mut Unit, target_stats: &UnitStats, fight_duration: f32) {
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

const TEMPLATE_BASE_AS: f32 = 0.658; //todo
impl Unit {
    pub const TEMPLATE_PROPERTIES: UnitProperties = UnitProperties {
        name: "Template champion",        //todo
        as_limit: Unit::DEFAULT_AS_LIMIT, //todo
        as_ratio: TEMPLATE_BASE_AS,       //if not specified, same as base AS //todo
        windup_percent: 0.2193,           //todo
        windup_modifier: 1.,              //"mod" next to attack windup, 1 by default //todo
        base_stats: UnitStats {
            hp: 610.,     //todo
            mana: 280.,   //todo
            base_ad: 59., //todo
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 26.,                //todo
            mr: 30.,                   //todo
            base_as: TEMPLATE_BASE_AS, //todo
            bonus_as: 0.,
            ability_haste: 0.,
            basic_haste: 0.,
            ultimate_haste: 0.,
            item_haste: 0.,
            crit_chance: 0.,
            crit_dmg: Unit::BASE_CRIT_DMG,
            ms_flat: 325., //todo
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
            hp: 101.,      //todo
            mana: 35.,     //todo
            base_ad: 2.95, //todo
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 4.6, //todo
            mr: 1.3,    //todo
            base_as: 0.,
            bonus_as: 0.0333, //todo
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
        basic_attack: units_data::default_basic_attack, //todo
        q: BasicAbility {
            cast: template_q,
            cast_time: 0., //todo
            //todo
            base_cooldown_by_ability_lvl: [0., 0., 0., 0., 0., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        w: BasicAbility {
            cast: template_w,
            cast_time: 0., //todo
            //todo
            base_cooldown_by_ability_lvl: [0., 0., 0., 0., 0., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        e: BasicAbility {
            cast: template_e,
            cast_time: 0., //todo
            //todo
            base_cooldown_by_ability_lvl: [0., 0., 0., 0., 0., F32_TOL], //basic abilities only uses the first 5 values (except for aphelios)
        },
        r: UltimateAbility {
            cast: template_r,
            cast_time: 0., //todo
            //todo
            base_cooldown_by_ability_lvl: [0., 0., 0.],
        },
        on_action_fns: OnActionFns {
            //todo
            on_lvl_set: None,
            on_fight_init: Some(template_init_abilities),
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
        fight_scenarios: &[(template_fight_scenario, "all out")], //todo
        defaults: UnitDefaults {
            runes_pages: RunesPage {
                keystone: &RuneKeystone::LETHAL_TEMPO, //todo
                //todo
                shard1: RuneShard::Middle,
                shard2: RuneShard::Left,
                shard3: RuneShard::Left,
            },
            skill_order: SkillOrder {
                //lvls:
                //  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18
                //todo
                q: [1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                w: [0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0],
                e: [0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1],
                r: [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0],
            },
            legendary_items_pool: &[
                //todo
                &Item::ABYSSAL_MASK,
                &Item::AXIOM_ARC,
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
                &Item::EXPERIMENTAL_HEXPLATE,
                &Item::FROZEN_HEART,
                &Item::GUARDIAN_ANGEL,
                &Item::GUINSOOS_RAGEBLADE,
                &Item::HEXTECH_ROCKETBELT,
                &Item::HORIZON_FOCUS,
                &Item::HUBRIS,
                &Item::HULLBREAKER,
                &Item::ICEBORN_GAUNTLET,
                &Item::IMMORTAL_SHIELDBOW,
                &Item::INFINITY_EDGE,
                &Item::JAKSHO,
                &Item::KAENIC_ROOKERN,
                &Item::KRAKEN_SLAYER,
                &Item::LIANDRYS_TORMENT,
                &Item::LICH_BANE,
                &Item::LORD_DOMINIKS_REGARDS,
                &Item::LUDENS_COMPANION,
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
                &Item::PROFANE_HYDRA,
                &Item::RABADONS_DEATHCAP,
                &Item::RANDUINS_OMEN,
                &Item::RAPID_FIRECANNON,
                &Item::RAVENOUS_HYDRA,
                &Item::RIFTMAKER,
                &Item::ROD_OF_AGES,
                &Item::RUNAANS_HURRICANE,
                &Item::RYLAIS_CRYSTAL_SCEPTER,
                &Item::SERAPHS_EMBRACE,
                &Item::SERPENTS_FANG,
                &Item::SERYLDAS_GRUDGE,
                &Item::SHADOWFLAME,
                &Item::SPEAR_OF_SHOJIN,
                &Item::STATIKK_SHIV,
                &Item::STERAKS_GAGE,
                &Item::STORMSURGE,
                &Item::STRIDEBREAKER,
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
                //todo
                &Item::BERSERKERS_GREAVES,
                &Item::BOOTS_OF_SWIFTNESS,
                &Item::IONIAN_BOOTS_OF_LUCIDITY,
                &Item::MERCURYS_TREADS,
                &Item::PLATED_STEELCAPS,
                &Item::SORCERERS_SHOES,
            ],
            supp_items_pool: &[
                //todo
            ],
        },
    };
}
