mod build_optimizer;
mod builds_analyzer;
mod cli;
mod game_data;

use game_data::units_data::*;

use items_data::*;
use runes_data::*;

/// Debug function for validating champions implementations.
#[allow(dead_code)]
fn champion_test_ground() {
    //creation of target dummy
    let dummy: Unit = Unit::new_target_dummy();

    //creation of champion
    let properties: &UnitProperties = &Unit::ASHE_PROPERTIES;
    let mut champ: Unit = Unit::new(
        properties,
        RunesPage {
            keystone: &RuneKeystone::FLEET_FOOTWORK,
            shard1: RuneShard::Middle,
            shard2: RuneShard::Left,
            shard3: RuneShard::Left,
        },
        properties.defaults.skill_order.clone(),
        6,
        Build([
            &Item::INFINITY_EDGE,
            &Item::ZHONYAS_HOURGLASS,
            &Item::NULL_ITEM,
            &Item::NULL_ITEM,
            &Item::NULL_ITEM,
            &Item::NULL_ITEM,
        ]),
    )
    .expect("Failed to create unit");
    println!("{}", champ);

    //champion actions
    champ.walk(champ.get_basic_attack_cd());
    println!("{}", champ.basic_attack(dummy.get_stats()));
}

fn main() {
    //champion_test_ground();
    cli::launch_interface();
}
