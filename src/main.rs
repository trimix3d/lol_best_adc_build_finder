mod build_optimizer;
mod builds_analyzer;
mod cli;
mod game_data;

use game_data::units_data::*;

use items_data::*;
use runes_data::*;

#[allow(dead_code)]
/// Test ground for validating champions implementations.
fn champion_test_ground() {
    //creation of target dummy
    let dummy: Unit = Unit::new_dummy().expect("Failed to create target dummy");

    //creation of champion
    let properties: &UnitProperties = &Unit::ASHE_PROPERTIES;
    let mut champ: Unit = Unit::new(
        properties,
        RunesPage {
            keystone: &RuneKeystone::PRESS_THE_ATTACK,
            shard1: RuneShard::Middle,
            shard2: RuneShard::Left,
            shard3: RuneShard::Left,
        },
        properties.defaults.skill_order.clone(),
        6,
        Build([
            &Item::NULL_ITEM,
            &Item::NULL_ITEM,
            &Item::NULL_ITEM,
            &Item::NULL_ITEM,
            &Item::NULL_ITEM,
            &Item::NULL_ITEM,
        ]),
    )
    .expect("Failed to create unit");

    //champion actions
    println!("{}", champ);
    champ.walk(champ.get_basic_attack_cd());
    println!("{}", champ.basic_attack(dummy.get_stats()));
}

fn main() {
    //champion_test_ground();
    cli::launch_interface();
}
