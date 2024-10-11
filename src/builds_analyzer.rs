use super::{
    champion_optimizer::{get_normalized_judgment_weights, BuildContainer},
    cli::{CHECK_MARK_CHAR, UNCHECKED_MARK_CHAR},
    game_data::{
        units_data::items_data::{BuildHash, ItemUtils},
        STARTING_GOLDS,
    },
};

use enumset::EnumSet;
use rustc_hash::{FxBuildHasher, FxHashMap};

use core::num::NonZeroUsize;

//todo: tier list and save it in file

/// Sort the provided pareto builds by their average score.
pub fn sort_builds_by_score(builds: &mut [BuildContainer], judgment_weights: (f32, f32, f32)) {
    //sanity check
    if builds.is_empty() {
        return;
    }

    let n_items: usize = builds[0].build.item_count(); //assumes all builds have the same length as the first of the list
    let max_golds: f32 = builds
        .iter()
        .map(|build| build.golds[n_items])
        .max_by(|a, b| a.partial_cmp(b).expect("Failed to compare floats"))
        .unwrap_or(STARTING_GOLDS);
    let normalized_judgment_weights: (f32, f32, f32) =
        get_normalized_judgment_weights(judgment_weights);

    //maybe using a hashmap is overkill to store average scores (but it allows to sanity check duplicates)
    let mut average_scores: FxHashMap<BuildHash, f32> =
        FxHashMap::with_capacity_and_hasher(builds.len(), FxBuildHasher);
    for container in builds.iter() {
        let old_score_maybe = average_scores.insert(
            container.build.get_hash(),
            container._get_avg_score_with_normalized_weights(
                n_items,
                max_golds,
                normalized_judgment_weights,
            ),
        );
        //sanity check
        assert!(
            old_score_maybe.is_none(),
            "Duplicate found in pareto builds"
        );
    }

    //sort in reverse order
    builds.sort_unstable_by(|container1, container2| {
        (average_scores.get(&container2.build.get_hash()).unwrap())
            .partial_cmp(average_scores.get(&container1.build.get_hash()).unwrap())
            .expect("Failed to compare floats")
    }); //getting keys will never panic as we previously inserted every build
}

/// Prints the provided pareto builds.
pub fn print_builds_scores(
    builds: &[BuildContainer],
    champ_name: &str,
    judgment_weights: (f32, f32, f32),
    n_to_print: NonZeroUsize,
    must_have_utils: EnumSet<ItemUtils>,
) {
    let filtered_builds = builds
        .iter()
        .filter(|container| (must_have_utils & !container.cum_utils).is_empty());
    let filtered_len = filtered_builds.clone().count();

    let n_to_print: usize = usize::min(n_to_print.get(), filtered_len);
    println!(
        "Showing the {n_to_print} best {champ_name} builds (out of {filtered_len}):\n\
         score | !h/s | surv | spec | build\n\
         ---------------------------------------------------"
    );

    //sanity check
    if filtered_len == 0 {
        println!("No builds to show!");
        return;
    }

    let n_items: usize = builds[0].build.item_count(); //assumes all builds have the same length as the first of the list
    let max_golds: f32 = builds
        .iter()
        .map(|build| build.golds[n_items])
        .max_by(|a, b| a.partial_cmp(b).expect("Failed to compare floats"))
        .unwrap_or(STARTING_GOLDS);
    let normalized_judgement_weights: (f32, f32, f32) =
        get_normalized_judgment_weights(judgment_weights);
    for container in filtered_builds.take(n_to_print) {
        print!(
            "{:5.0} | {:^4} | {:^4} | {:^4} | ",
            container._get_avg_score_with_normalized_weights(
                n_items,
                max_golds,
                normalized_judgement_weights
            ),
            if container.cum_utils.contains(ItemUtils::AntiHealShield) {
                CHECK_MARK_CHAR
            } else {
                UNCHECKED_MARK_CHAR
            },
            if container.cum_utils.contains(ItemUtils::Survivability) {
                CHECK_MARK_CHAR
            } else {
                UNCHECKED_MARK_CHAR
            },
            if container.cum_utils.contains(ItemUtils::Special) {
                CHECK_MARK_CHAR
            } else {
                UNCHECKED_MARK_CHAR
            },
        );
        for item_idx in 0..(n_items - 1) {
            print!(
                "{}-{} ({:.0}), ",
                item_idx + 1,
                container.build[item_idx],
                container._get_score_item_slot_with_normalized_weights(
                    item_idx + 1,
                    normalized_judgement_weights
                )
            );
        }
        println!(
            "{}-{} ({:.0})",
            n_items,
            container.build[n_items - 1],
            container._get_score_item_slot_with_normalized_weights(
                n_items,
                normalized_judgement_weights
            )
        );
    }
}
