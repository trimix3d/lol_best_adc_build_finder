use crate::game_data::STARTING_GOLDS;

use super::{
    build_optimizer::{get_normalized_judgment_weights, BuildContainer},
    game_data::items_data::{BuildHash, ItemUtils},
};

use rustc_hash::{FxBuildHasher, FxHashMap};

/// Sort the provided pareto builds by their average score.
pub fn sort_builds_by_score(builds_ref: &mut [BuildContainer], judgment_weights: (f32, f32, f32)) {
    let n_items: usize = builds_ref[0].build.item_count(); //assumes all builds have the same length
    let max_golds: f32 = builds_ref
        .iter()
        .map(|build| build.golds[n_items])
        .max_by(|a, b| a.partial_cmp(b).expect("failed to compare floats"))
        .unwrap_or(STARTING_GOLDS);
    let normalized_judgement_weights: (f32, f32, f32) =
        get_normalized_judgment_weights(judgment_weights);

    let mut average_scores: FxHashMap<BuildHash, f32> =
        FxHashMap::with_capacity_and_hasher(builds_ref.len(), FxBuildHasher);
    for container in builds_ref.iter() {
        let old = average_scores.insert(
            container.build.get_hash(),
            container.get_avg_score_with_normalized_weights(
                n_items,
                max_golds,
                normalized_judgement_weights,
            ),
        );
        //sanity check
        assert!(old.is_none(), "duplicate found in pareto builds");
    }

    //sort in reverse order
    builds_ref.sort_unstable_by(|container1, container2| {
        (average_scores.get(&container2.build.get_hash()).unwrap())
            .partial_cmp(average_scores.get(&container1.build.get_hash()).unwrap())
            .expect("failed to compare floats")
    }); //getting keys will never panic as we previously inserted every build
}

/// Prints the provided pareto builds.
pub fn print_builds_scores(
    builds_ref: &[BuildContainer],
    n_to_print: usize,
    judgment_weights: (f32, f32, f32),
) {
    let n_items: usize = builds_ref[0].build.item_count(); //assumes all builds have the same length
    let max_golds: f32 = builds_ref
        .iter()
        .map(|build| build.golds[n_items])
        .max_by(|a, b| a.partial_cmp(b).expect("failed to compare floats"))
        .unwrap_or(STARTING_GOLDS);
    let normalized_judgement_weights: (f32, f32, f32) =
        get_normalized_judgment_weights(judgment_weights);

    //print builds
    let n_to_print: usize = usize::min(n_to_print, builds_ref.len());
    println!(
        "showing the {n_to_print} best builds (out of {}):\n\
         score | !h/s | surv | other | build\n\
         ---------------------------------------------------",
        builds_ref.len()
    );
    for container in &builds_ref[0..n_to_print] {
        print!(
            "{:5.0} | {:^4} | {:^4} | {:^5} | ",
            container.get_avg_score_with_normalized_weights(
                n_items,
                max_golds,
                normalized_judgement_weights
            ),
            u8::from(
                container
                    .cumulated_utils
                    .contains(ItemUtils::AntiHealShield)
            ),
            u8::from(container.cumulated_utils.contains(ItemUtils::Survivability)),
            u8::from(container.cumulated_utils.contains(ItemUtils::Other))
        );
        for item_idx in 0..(n_items - 1) {
            print!(
                "{}-{} ({:.0}), ",
                item_idx + 1,
                container.build[item_idx],
                container
                    .get_score_with_normalized_weights(item_idx + 1, normalized_judgement_weights)
            );
        }
        println!(
            "{}-{} ({:.0})",
            n_items,
            container.build[n_items - 1],
            container.get_score_with_normalized_weights(n_items, normalized_judgement_weights)
        );
    }
}
