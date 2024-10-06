mod build_optimizer;
mod builds_analyzer;
mod cli;
mod game_data;

use game_data::units_data::*;

use items_data::{items::*, *};
use runes_data::RunesPage;

#[allow(dead_code)]
/// Test ground for validating champions implementations.
fn champion_test_ground() {
    //target dummy properties
    const TARGET_DUMMY_BASE_AS: f32 = 0.658; //in game default value is 0.658
    const TARGET_DUMMY_PROPERTIES: UnitProperties = UnitProperties {
        name: "target_dummy",
        as_limit: Unit::DEFAULT_AS_LIMIT,
        as_ratio: TARGET_DUMMY_BASE_AS,
        windup_percent: 0.5,
        windup_modifier: 1.,
        base_stats: UnitStats {
            hp: 1000., //in game default value is 1000.
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 0., //in game default value is 0.
            mr: 0.,    //in game default value is 0.
            base_as: TARGET_DUMMY_BASE_AS,
            bonus_as: 0.,
            ability_haste: 0.,
            basic_haste: 0.,
            ultimate_haste: 0.,
            item_haste: 0.,
            crit_chance: 0.,
            crit_dmg: Unit::BASE_CRIT_DMG,
            ms_flat: 370., //in game default value is 370.
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
        //no growth stats so they remain constant (lvl doesn't matter)
        growth_stats: UnitStats::const_default(),
        basic_attack: null_basic_attack,
        q: NULL_BASIC_ABILITY,
        w: NULL_BASIC_ABILITY,
        e: NULL_BASIC_ABILITY,
        r: NULL_ULTIMATE_ABILITY,
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: None,
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
        fight_scenarios: &[(null_simulate_fight, "null")],
        unit_defaults: UnitDefaults {
            runes_pages: RunesPage::const_default(),
            skill_order: SkillOrder::const_default(), //does nothing since dummy has no ability
            legendary_items_pool: &ALL_LEGENDARY_ITEMS,
            boots_pool: &ALL_BOOTS,
            support_items_pool: &ALL_SUPPORT_ITEMS,
        },
    };

    //creation of target dummy
    let dummy: Unit = Unit::from_defaults(&TARGET_DUMMY_PROPERTIES, 6, Build::default())
        .expect("Failed to create target dummy");

    //creation of champion
    let mut champ: Unit = Unit::from_defaults(
        &Unit::LUCIAN_PROPERTIES,
        6,
        Build([
            &RUNAANS_HURRICANE,
            &KRAKEN_SLAYER,
            &INFINITY_EDGE,
            &PHANTOM_DANCER,
            &NAVORI_FLICKERBLADE,
            &NULL_ITEM,
        ]),
    )
    .expect("Failed to create unit");

    //champion actions
    println!("{}", champ);
    champ.walk(champ.basic_attack_cd);
    println!("{} - t: {}", champ.basic_attack(&dummy.stats), champ.time,);
    champ.walk(champ.basic_attack_cd);
    println!("{} - t: {}", champ.basic_attack(&dummy.stats), champ.time,);
    champ.walk(champ.basic_attack_cd);
    println!("{} - t: {}", champ.basic_attack(&dummy.stats), champ.time,);
    champ.walk(champ.basic_attack_cd);
    println!("{} - t: {}", champ.basic_attack(&dummy.stats), champ.time,);
    champ.walk(champ.basic_attack_cd);
    println!("{} - t: {}", champ.basic_attack(&dummy.stats), champ.time,);
    champ.walk(champ.basic_attack_cd);
    println!("{} - t: {}", champ.basic_attack(&dummy.stats), champ.time,);
    champ.walk(champ.basic_attack_cd);
    println!("{} - t: {}", champ.basic_attack(&dummy.stats), champ.time,);
}

fn main() {
    //todo: remove all "!!!"
    //champion_test_ground();
    cli::launch_interface();
}
