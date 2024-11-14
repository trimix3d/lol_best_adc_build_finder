mod builds_analyzer;
mod champion_optimizer;
mod cli;
mod game_data;

use game_data::units_data::*;

use items_data::*;
use runes_data::*;

/// Sorts the slice and compares adjacent elements to find if there are duplicates.
/// Return a reference to the first duplicate found, if any.
/// The slice given to this function will be modified, if you don't want to modify the given slice, pass a clone.
fn find_dupes_in_slice<T: Ord>(slice: &mut [T]) -> Option<&T> {
    slice.sort_unstable();
    for window in slice.windows(2) {
        if window[0] == window[1] {
            return Some(&window[0]);
        }
    }
    None
}

/// Debug function for validating champions implementations.
#[allow(dead_code)]
fn champion_test_ground() {
    //creation of target dummy
    let dummy: Unit = Unit::new_target_dummy();

    //creation of champion
    let properties: &UnitProperties = &Unit::EZREAL_PROPERTIES;
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
            &Item::STATIKK_SHIV,
            &Item::NULL_ITEM,
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
