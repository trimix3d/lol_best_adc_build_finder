mod build_optimizer;
mod builds_analyzer;
mod cli;
pub mod game_data;

use game_data::items_data::{items::*, *};
use game_data::units_data::*;

use runes_data::{RuneShard, RunesPage};

#[allow(dead_code)]
/// Test ground for validating champions implementations.
fn champion_test_ground() {
    //target dummy
    const TARGET_DUMMY_RUNES_PAGE: RunesPage = RunesPage {
        shard1: RuneShard::Left,
        shard2: RuneShard::Left,
        shard3: RuneShard::Left,
    };
    const TARGET_DUMMY_SKILL_ORDER: SkillOrder = SkillOrder::const_default(); //does nothing since dummy has no spell (except passing validity checks when creating the dummy)

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
            ap_coef: 0.,
            armor: 100., //in game default value is 0.
            mr: 100.,    //in game default value is 0.
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
        },
        //no growth stats so they remain constant (lvl doesn't matter)
        growth_stats: UnitStats::const_default(),
        on_lvl_set: None,
        init_spells: None,
        basic_attack: null_basic_attack,
        q: NULL_SPELL,
        w: NULL_SPELL,
        e: NULL_SPELL,
        r: NULL_SPELL,
        fight_scenarios: &[(null_simulate_fight, "null")],
        unit_defaults: UnitDefaults {
            runes_pages: &TARGET_DUMMY_RUNES_PAGE,
            skill_order: &TARGET_DUMMY_SKILL_ORDER,
            legendary_items_pool: &ALL_LEGENDARY_ITEMS,
            boots_pool: &ALL_BOOTS,
            support_items_pool: &ALL_SUPPORT_ITEMS,
        },
    };

    //creation of target dummy
    let target_dummy: Unit = Unit::from_defaults(&TARGET_DUMMY_PROPERTIES, 6, Build::default())
        .expect("failed to create target dummy");

    //creation of champion
    let mut champ: Unit = Unit::from_defaults(
        &Unit::ASHE_PROPERTIES,
        6,
        Build([
            &NULL_ITEM, &NULL_ITEM, &NULL_ITEM, &NULL_ITEM, &NULL_ITEM, &NULL_ITEM,
        ]),
    )
    .expect("failed to create unit");

    println!("{}", champ);
    println!("{}", champ.basic_attack(&target_dummy.stats));
    champ.walk(champ.basic_attack_cd);
}

fn main() {
    //champion_test_ground();
    cli::launch_interface();
}