use super::build_optimizer::*;
use super::builds_analyzer::*;
use super::game_data::*;

use items_data::{items::*, *};
use units_data::*;

use constcat::concat;

use std::fmt::Debug;
use std::io;
use std::iter::Iterator;
use std::ops::RangeBounds;

use io::Write;

/// Number of builds to be printed by default when displaying results.
const DEFAULT_N_PRINTED_BUILDS: usize = 20;

pub fn launch_interface() {
    println!(
        "---------------------------------------------------\n\
         ---\\ LoL best ADC build finder - patch {:2}.{:2} \\-----\n\
         ----\\ Champions implemented: {:3}              \\----\n\
         -----\\ Items in database: {:3}                  \\---\n\
         ---------------------------------------------------",
        PATCH_NUMBER_MAJOR,
        PATCH_NUMBER_MINOR,
        Unit::ALL_CHAMPIONS.len(),
        ALL_LEGENDARY_ITEMS.len() + ALL_BOOTS.len() + ALL_SUPPORT_ITEMS.len()
    );
    println!(
        "At any time, you can type:\n\
         - help/? to show help info on the current context.\n\
         - back/b to go back.\n\
         - home to return to this page.\n\
         - exit to exit the program."
    );

    let champion_names: Vec<&str> = Unit::ALL_CHAMPIONS
        .iter()
        .map(|champ_properties| champ_properties.name)
        .collect();

    let mut greetings_msg: String = String::from("Available champions: ");
    greetings_msg.push_str(champion_names[0]);
    for name in champion_names[1..].iter() {
        greetings_msg.push_str(", ");
        greetings_msg.push_str(name);
    }

    loop {
        match get_user_matching_input(
            &greetings_msg,
            "Enter the champion for which you want to find the best builds",
            "Please enter a valid champion name (among those available)",
            "No help message available.",
            champion_names.iter().copied(),
            false, //safety of a later expect() depends on this argument to be false
        ) {
            Ok(index) => {
                if let Err(UserCommand::Exit) = handle_builds_generation(
                    Unit::ALL_CHAMPIONS
                        [index.expect("Expected an input from user, but received none")],
                ) {
                    break;
                }
            }
            Err(UserCommand::Back) => println!("Cannot go further back"),
            Err(UserCommand::Home) => (), //already home
            Err(UserCommand::Exit) => return,
        }
    }
}

#[derive(Debug)]
/// Represents user command that are possible to trigger anywhere in the cli
/// and that must be transmitted through different functions.
enum UserCommand {
    Back,
    Home,
    Exit,
}

/// Get the user input, returns it in a lowercase String.
/// doesn't catch user commands (go back, exit, etc) and returns the String directly
/// (can still returns `Err(UserCommand::Exit)`, but only when stdin is closed).
fn get_user_raw_input(input_line: &str) -> Result<String, UserCommand> {
    print!("{input_line}: ");
    io::stdout().flush().expect("Failed to flush stdout");

    let mut buffer: String = String::new();
    io::stdin()
        .read_line(&mut buffer)
        .expect("Failed to read user input from stdin");
    //exit if stdin is closed
    if buffer.is_empty() {
        return Err(UserCommand::Exit);
    }
    Ok(buffer.trim().to_lowercase())
}

/// Get the user input, returns it in a lowercase String.
/// Catches user commands (go back, exit, etc) and may return an Err with the specific variant to handle.
fn get_user_input(input_line: &str, help_msg: &str) -> Result<String, UserCommand> {
    loop {
        println!();
        let input: String = get_user_raw_input(input_line)?;
        match input.as_str() {
            "help" | "?" => println!("\n---[ HELP ]---\n{help_msg}"),
            "back" | "b" => return Err(UserCommand::Back),
            "home" => return Err(UserCommand::Home),
            "exit" => {
                if confirm_exit()? {
                    return Err(UserCommand::Exit);
                }
            }
            _ => return Ok(input),
        }
    }
}

/// Prompt the user to confirm the exit of the program and return the result in an Ok(bool).
/// Returns `Err(UserCommand::Exit)` if stdin is closed.
fn confirm_exit() -> Result<bool, UserCommand> {
    loop {
        let input: String = get_user_raw_input("Confirm exit? (y/n)")?;
        match input.as_str() {
            "yes" | "y" | "" => return Ok(true),
            "no" | "n" => return Ok(false),
            _ => {
                println!("'{input}' is not a recognized input, press enter or type 'yes' to confirm/'no' to deny");
            }
        }
    }
}

/// Matches the user input with the provided `match_strs` and returns the corresponding index.
/// This function will either:
///  - loop until the user provides a valid input and return `Ok(corresponding_index`).
///  - return Err with the specific variant if the user performs one of the persistent command.
///
/// This will return None only if `allow_no_input` is true and the user enters no input.
/// `invalid_msg` must fit into one line.
fn get_user_matching_input<'a>(
    greetings_msg: &str,
    input_line: &str,
    invalid_line: &str,
    help_msg: &str,
    match_strs: impl IntoIterator<Item = &'a str>,
    allow_no_input: bool,
) -> Result<Option<usize>, UserCommand> {
    if !(greetings_msg.is_empty()) {
        println!("\n{greetings_msg}");
    }

    //ensure to match lowercase inputs with lowercase strings
    let match_strings: Vec<String> = match_strs.into_iter().map(str::to_lowercase).collect();

    loop {
        let input: String = get_user_input(input_line, help_msg)?;

        if input.is_empty() {
            if allow_no_input {
                return Ok(None);
            } else {
                println!("{invalid_line}");
            }
        } else if let Some(index) = match_strings.iter().position(|s| *s == input) {
            return Ok(Some(index));
        } else {
            println!("'{input}' is not a recognized input, {invalid_line}");
        }
    }
}

/// Matches the user input with the provided choices and returns the corresponding number.
/// This function will either:
///  - loop until the user provides a valid input and return Ok(number).
///  - return Err with the specific variant if the user performs one of the persistent command.
///
/// The number returned is the index of the corresponding choice + 1,
/// This will return None only if `allow_no_input` is true and the user enters no input.
fn get_user_choice<'a>(
    greetings_msg: &str,
    input_line: &str,
    help_msg: &str,
    choices: impl IntoIterator<Item = &'a str>,
    allow_no_input: bool,
) -> Result<Option<usize>, UserCommand> {
    //create String that displays every choice to the user
    let mut choices_iter = choices.into_iter();
    let mut greetings_with_choices_msg: String = String::from(greetings_msg);

    //need to handle first choice slightly differently (depending if `greetings_msg` is empty or not)
    if !(greetings_msg.is_empty()) {
        greetings_with_choices_msg.push('\n');
    }
    let mut counter: usize = 1; //display choices numbers starting from 1
    greetings_with_choices_msg.push_str(&counter.to_string());
    greetings_with_choices_msg.push_str(") ");
    greetings_with_choices_msg.push_str(
        choices_iter
            .next()
            .expect("Choices given to user are empty"),
    );
    for choice_str in choices_iter {
        counter += 1;
        greetings_with_choices_msg.push('\n');
        greetings_with_choices_msg.push_str(&counter.to_string()); //use after increasing counter because we display choices numbers starting from 1
        greetings_with_choices_msg.push_str(") ");
        greetings_with_choices_msg.push_str(choice_str);
    }

    get_user_usize(
        &greetings_with_choices_msg,
        input_line,
        help_msg,
        1..=counter,
        allow_no_input,
    )
}

/// Prompts the user to enter a positive integer and returns it.
/// This will return None only if `allow_no_input` is true and the user enters no input.
fn get_user_usize(
    greetings_msg: &str,
    input_line: &str,
    help_msg: &str,
    range: impl RangeBounds<usize>,
    allow_no_input: bool,
) -> Result<Option<usize>, UserCommand> {
    if !(greetings_msg.is_empty()) {
        println!("\n{greetings_msg}");
    }

    loop {
        let input: String = get_user_input(input_line, help_msg)?;

        if input.is_empty() {
            if allow_no_input {
                return Ok(None);
            } else {
                println!("Please enter a valid integer");
            }
        } else if let Ok(number) = input.parse::<usize>() {
            if range.contains(&number) {
                return Ok(Some(number));
            } else {
                println!(
                    "{} is outside of range: ({:?}, {:?})",
                    number,
                    range.start_bound(),
                    range.end_bound()
                );
            }
        } else {
            println!("'{input}' is not a valid integer");
        }
    }
}

/// Prompts the user to enter a (float) number and returns it.
/// This will return None only if `allow_no_input` is true and the user enters no input.
fn get_user_f32(
    greetings_msg: &str,
    input_line: &str,
    help_msg: &str,
    allow_no_input: bool,
) -> Result<Option<f32>, UserCommand> {
    if !(greetings_msg.is_empty()) {
        println!("\n{greetings_msg}");
    }

    loop {
        let input: String = get_user_input(input_line, help_msg)?;

        if input.is_empty() {
            if allow_no_input {
                return Ok(None);
            } else {
                println!("Please enter a valid number");
            }
        } else if let Ok(number) = input.parse::<f32>() {
            return Ok(Some(number));
        } else {
            println!("'{input}' is not a valid number");
        }
    }
}

fn sanitize_item_name(name: &str) -> String {
    name.replace('_', " ") //replace underscores with spaces
        .replace(&['-', '\''][..], "") //remove - and '
        .to_lowercase()
}

/// Prompts the user to enter an item name and returns the corresponding item.
fn get_user_item(input_line: &str) -> Result<&'static Item, UserCommand> {
    //ensure to match lowercase inputs with lowercase strings
    let item_names: Vec<(String, String)> = ALL_ITEMS
        .iter()
        .map(|item| {
            (
                sanitize_item_name(item.full_name),
                sanitize_item_name(item.short_name),
            )
        })
        .collect();

    loop {
        let input: String = get_user_input(
            input_line,
            "Enter an item name (type 'list' to show available items in database)",
        )?;
        let sanitized_input: String = sanitize_item_name(&input);

        if sanitized_input.is_empty() {
            return Ok(&NULL_ITEM);
        } else if let Some(index) = item_names.iter().position(|(full_name, short_name)| {
            *full_name == sanitized_input || *short_name == sanitized_input
        }) {
            return Ok(ALL_ITEMS[index]);
        } else if sanitized_input == "list" {
            //print list of items
            println!("\nLegendary items in database:");
            for item in ALL_LEGENDARY_ITEMS {
                println!("- {item:#}");
            }
            println!("\nBoots in database:");
            for item in ALL_BOOTS {
                println!("- {item:#}");
            }
            println!("\nSupport items in database:");
            for item in ALL_SUPPORT_ITEMS {
                println!("- {item:#}");
            }
        } else {
            println!("'{input}' is not a recognized item (type 'list' to show available items in database)");
        }
    }
}

/// Handle the whole build generation with the user.
/// This function never returns `Err(UserCommand::back)` because cannot go further back.
fn handle_builds_generation(champ_properties: &'static UnitProperties) -> Result<(), UserCommand> {
    //create champion
    let mut champ: Unit =
        Unit::from_defaults(champ_properties, 6, Build::default()).expect("Failed to create unit");

    //create build generation settings
    let mut settings: BuildsGenerationSettings =
        BuildsGenerationSettings::default_by_champion(champ.properties);

    loop {
        //set build generation settings
        match confirm_builds_generation_settings(&mut settings, &mut champ) {
            Ok(()) => (),
            Err(UserCommand::Back) => return Ok(()),
            Err(command) => return Err(command),
        };

        //start computation
        println!();
        let mut pareto_builds = match find_best_builds(&mut champ, &settings) {
            Ok(pareto_builds) => pareto_builds,
            Err(error_msg) => {
                get_user_raw_input(&format!(
                        "\nFailed to generate builds: {error_msg} (press enter to return to settings screen) "
                    ))?;
                continue;
            }
        };

        //print results
        sort_builds_by_score(&mut pareto_builds, settings.judgment_weights);
        println!();
        print_builds_scores(
            &pareto_builds,
            DEFAULT_N_PRINTED_BUILDS,
            settings.judgment_weights,
        );

        //prompt for next thing to do
        loop {
            let choice: usize = match get_user_choice(
                    "",
                    "Select an action (press enter to return to champion selection)",
                    "How to interpret the columns from left to right:\n \
                    - score: the overall score of the build (according to the judgment weights)\n \
                    - !h/s : if the build has anti-heal or anti-shield utility\n \
                    - surv : if the build has survivability utility (e.g. zhonyas stasis, edge of night spell shield, ...)\n \
                    - other: if the build has other utility (e.g. RFC bonus range, black cleaver armor reduction, ...)\n \
                    - build: the build in item order (with the score at each item slot in brackets)",
                    ["show more builds", "restart generation with different settings"],
                    true,
                ) {
                    Ok(Some(choice)) => choice,
                    Ok(None) => return Ok(()),
                    Err(UserCommand::Back) => break,
                    Err(command) => return Err(command)
                };
            match choice {
                1 => {
                    match get_user_usize(
                        "",
                        "Enter the number of builds to show",
                        "No help message available.",
                        1..,
                        false, //safety of a later expect() depends on this argument to be false
                    ) {
                        Ok(n) => print_builds_scores(
                            &pareto_builds,
                            n.expect("Expected an input from user, but received none"),
                            settings.judgment_weights,
                        ),
                        Err(UserCommand::Back) => (),
                        Err(command) => return Err(command),
                    }
                }
                2 => break,
                _ => unreachable!("Unhandled user input"),
            }
        }
    }
}

const BUILDS_GENERATION_SETTINGS_HELP_MSG: &str = concat!(
    "Meaning of these settings:\n\
    1) target: ",
    TARGET_HELP_MSG,
    "\n\n2) fight scenario: ",
    FIGHT_SCENARIO_HELP_MSG,
    "\n\n3) fight duration: ",
    FIGHT_DURATION_HELP_MSG,
    "\n\n4) percentage of AD damage taken: ",
    AD_TAKEN_PERCENT_HELP_MSG,
    "\n\n5) judgment weights: 3 values, one for DPS, one for defense and one for mobility.\n\
    The DPS weight is used to measure the importance of the champion's DPS in the score given to a build.\n\
    The defense weight is used to measure the importance of the champion's defensive stats, heals and hields in the score given to a build.\n\
    The mobility weight is used to measure the importance of the champion's mobility in the score given to a build.\n\
    The absolute value of these weight is not relevant, what's important is their value relative to each other.\n\
    i.e. if the DPS weight has 2x the value of the defense weight, increasing the DPS will be 2x more important than increasing the defense in the eyes of the optimizer",
    "\n\n6) number of items per build: ",
    N_ITEMS_HELP_MSG,
    "\n\n7) boots slot: ",
    BOOTS_SLOT_HELP_MSG,
    "\n\n8) allow boots if slot is not specified: ",
    ALLOW_BOOTS_IF_NO_SLOT_HELP_MSG,
    "\n\n9) support item slot: ",
    SUPPORT_ITEM_SLOT_HELP_MSG,
    "\n\n10) allow manaflow items in first slot: ",
    ALLOW_MANAFLOW_FIRST_ITEM_HELP_MSG,
    "\n\n11) mandatory items: ",
    MANDATORY_ITEMS_HELP_MSG,
    "\n\n12) search threshold: ",
    SEARCH_THRESHOLD_HELP_MSG
);

/// Show the build generation settings, prompt the user for any change and returns the settings when done.
fn confirm_builds_generation_settings(
    settings_ref: &mut BuildsGenerationSettings,
    champ: &mut Unit,
) -> Result<(), UserCommand> {
    loop {
        let choices_strings: [String; 13] = [
            format!("target: {}", settings_ref.target_properties.name),
            format!("fight scenario: {}", champ.fight_scenario.1),
            format!(
                "fight duration: {}s{}",
                settings_ref.fight_duration,
                if settings_ref.fight_duration < LOW_FIGHT_DURATION_VALUE_WARNING {
                    format!(
                        " (/!\\ set to a low value (<{}s), this can lead to inaccurate results)",
                        LOW_FIGHT_DURATION_VALUE_WARNING
                    )
                } else {
                    "".to_string()
                }
            ),
            format!(
                "percentage of AD damage taken: {:.0}%",
                100. * settings_ref.ad_taken_percent,
            ),
            format!(
                "judgment weights: DPS {}, defense {}, mobility {}",
                settings_ref.judgment_weights.0,
                settings_ref.judgment_weights.1,
                settings_ref.judgment_weights.2
            ),
            format!("number of items per build: {}", settings_ref.n_items),
            format!(
                "boots slot: {}",
                if settings_ref.boots_slot == 0 {
                    "not specified".to_string()
                } else {
                    settings_ref.boots_slot.to_string()
                }
            ),
            format!(
                "allow boots if slot is not specified: {}",
                settings_ref.allow_boots_if_no_slot
            ),
            format!(
                "support item slot: {}",
                if settings_ref.support_item_slot == 0 {
                    "none".to_string()
                } else {
                    settings_ref.support_item_slot.to_string()
                }
            ),
            format!(
                "allow manaflow items in first slot: {}",
                settings_ref.allow_manaflow_first_item
            ),
            format!("mandatory items: {}", settings_ref.mandatory_items),
            format!(
                "search threshold: {:.0}%{}",
                100. * settings_ref.search_threshold,
                if settings_ref.search_threshold < LOW_SEARCH_THRESHOLD_VALUE_WARNING {
                    format!(
                        " (/!\\ set to a low value (<{:.0}%), this can lead to inaccurate results)",
                        100. * LOW_SEARCH_THRESHOLD_VALUE_WARNING
                    )
                } else if settings_ref.search_threshold > HIGH_SEARCH_THRESHOLD_VALUE_WARNING {
                    format!(
                        " (/!\\ set to a high value (>{:.0}%), this can result in long computation time)",
                        100. * HIGH_SEARCH_THRESHOLD_VALUE_WARNING
                    )
                } else {
                    "".to_string()
                }
            ),
            "reset to default settings".to_string(),
        ];

        let choice: usize = match get_user_choice(
            format!(
                "Build generation for {} will be launched with these settings:",
                champ.properties.name
            )
            .as_str(),
            "Select a setting to change (press enter to confirm current settings)",
            BUILDS_GENERATION_SETTINGS_HELP_MSG,
            choices_strings.iter().map(String::as_str),
            true,
        )? {
            Some(choice) => choice,
            None => return Ok(()),
        };

        match choice {
            1 => {
                //target
                change_target(settings_ref)?;
            }
            2 => {
                //fight_scenario
                change_fight_scenario(champ)?;
            }
            3 => {
                //fight_duration
                change_fight_duration(settings_ref)?;
            }
            4 => {
                //ad_taken_percent
                change_ad_taken_percent(settings_ref)?;
            }
            5 => {
                //judgment_weights
                change_judgment_weights(settings_ref)?;
            }
            6 => {
                //n_items
                change_n_items(settings_ref)?;
            }
            7 => {
                //boots_slot
                change_boots_slot(settings_ref)?;
            }
            8 => {
                settings_ref.allow_boots_if_no_slot = !settings_ref.allow_boots_if_no_slot;
            }
            9 => {
                //support_item_slot
                change_support_item_slot(settings_ref)?;
            }
            //todo: change_items_pools feature, incorporate allow_manaflow_first_item & allow_boots_if_no_slot inside
            10 => {
                settings_ref.allow_manaflow_first_item = !settings_ref.allow_manaflow_first_item;
            }
            11 => {
                //mandatory_items
                change_mandatory_items(settings_ref)?;
            }
            12 => {
                //search_threshold
                change_search_threshold(settings_ref)?;
            }
            13 => {
                //reset to default settings
                *settings_ref = BuildsGenerationSettings::default_by_champion(champ.properties);
            }
            _ => unreachable!("Unhandled user input"),
        }
    }
}

const TARGET_HELP_MSG: &str = "The selected target will be used to compute the champion's DPS.";

/// This function never returns `Err(UserCommand::back)`.
fn change_target(settings: &mut BuildsGenerationSettings) -> Result<(), UserCommand> {
    loop {
        let choice: usize = match get_user_choice(
            "Available targets:",
            "Select a target",
            TARGET_HELP_MSG,
            TARGET_OPTIONS.iter().map(|properties| properties.name),
            false,
        ) {
            Ok(Some(choice)) => choice,
            Ok(None) => return Ok(()), //should never get here because `allow_no_input` is false but just in case
            Err(UserCommand::Back) => return Ok(()),
            Err(command) => return Err(command),
        };

        let old_target: &UnitProperties = settings.target_properties; //backup before checking validity
        settings.target_properties = TARGET_OPTIONS[choice - 1];

        if let Err(error_msg) = settings.check_settings() {
            println!("Failed to set target: {error_msg}");
            settings.target_properties = old_target; //restore valid value
        } else {
            return Ok(());
        }
    }
}

const FIGHT_SCENARIO_HELP_MSG: &str = "Each generated build will go through a fight simulation according to the selected scenario to evaluate its performance.\n\
        Hence, the builds found will perform best for the selected scenario.";

/// This function never returns `Err(UserCommand::back)`.
fn change_fight_scenario(champ: &mut Unit) -> Result<(), UserCommand> {
    match get_user_choice(
        &format!(
            "Available fight scenarios for {} are:",
            champ.properties.name
        ),
        &format!("Select a fight scenario for {}", champ.properties.name),
        FIGHT_SCENARIO_HELP_MSG,
        champ
            .properties
            .fight_scenarios
            .iter()
            .map(|scenario| scenario.1),
        false,
    ) {
        Ok(Some(choice)) => {
            champ.fight_scenario = champ.properties.fight_scenarios[choice - 1];
            Ok(())
        }
        Ok(None) => Ok(()), //should never get here because `allow_no_input` is false but just in case
        Err(UserCommand::Back) => Ok(()),
        Err(command) => Err(command),
    }
}

const FIGHT_DURATION_HELP_MSG: &str =
    "Every build will be evaluated based on a fight simulation of the selected duration (in seconds).";

/// This function never returns `Err(UserCommand::back)`.
fn change_fight_duration(settings: &mut BuildsGenerationSettings) -> Result<(), UserCommand> {
    loop {
        let number: f32 =
            match get_user_f32("", "Enter a fight duration", FIGHT_DURATION_HELP_MSG, false) {
                Ok(Some(number)) => number,
                Ok(None) => return Ok(()), //should never get here because `allow_no_input` is false but just in case
                Err(UserCommand::Back) => return Ok(()),
                Err(command) => return Err(command),
            };

        let old_fight_duration: f32 = settings.fight_duration; //backup before checking validity
        settings.fight_duration = number;

        if let Err(error_msg) = settings.check_settings() {
            println!("Failed to set fight duration: {error_msg}");
            settings.fight_duration = old_fight_duration; //restore valid value
        } else {
            return Ok(());
        }
    }
}

const AD_TAKEN_PERCENT_HELP_MSG: &str =
    "When evaluating the defensive value of different builds, the selected percentage of AD dmg taken will be considered.\n\
     The percentage of AP dmg taken is deducted from this (assuming no true dmg taken).";

/// This function never returns `Err(UserCommand::back)`.
fn change_ad_taken_percent(settings: &mut BuildsGenerationSettings) -> Result<(), UserCommand> {
    loop {
        let number: f32 = match get_user_f32(
            "",
            "Enter the percentage of AD dmg taken by the champion",
            AD_TAKEN_PERCENT_HELP_MSG,
            false,
        ) {
            Ok(Some(number)) => number,
            Ok(None) => return Ok(()), //should never get here because `allow_no_input` is false but just in case
            Err(UserCommand::Back) => return Ok(()),
            Err(command) => return Err(command),
        };

        let old_ad_taken_percent: f32 = settings.ad_taken_percent; //backup before checking validity
        settings.ad_taken_percent = number / 100.;

        if let Err(error_msg) = settings.check_settings() {
            println!("Failed to set percentage of AD dmg taken: {error_msg}");
            settings.ad_taken_percent = old_ad_taken_percent; //restore valid value
        } else {
            return Ok(());
        }
    }
}

#[allow(clippy::type_complexity)]
fn get_user_judgment_weights() -> Result<(Option<f32>, Option<f32>, Option<f32>), UserCommand> {
    //get dps weight
    let dps_weight: Option<f32> = get_user_f32("",
        "Enter the DPS weight (press enter to keep the previous value)",
        "The DPS weight is used to measure the importance of the champion's DPS when calculating the gold value of a build.\n\
        The absolute value of the weight is not relevant, what is important is its value relative to other weights.",
        true)?;

    //get defense weight
    let defense_weight: Option<f32> = get_user_f32("",
        "Enter the defense weight (press enter to keep the previous value)",
        "The defense weight is used to measure the importance of the champion's defensive stats, heals and hields when calculating the gold value of a build.\n\
        The absolute value of the weight is not relevant, what is important is its value relative to other weights.",
        true)?;

    //get ms weight
    let ms_weight: Option<f32> = get_user_f32("",
        "Enter the mobility weight (press enter to keep the previous value)",
        "The mobility weight is used to measure the importance of the champion's mobility when calculating the gold value of a build.\n\
        The absolute value of the weight is not relevant, what is important is its value relative to other weights.",
        true)?;

    Ok((dps_weight, defense_weight, ms_weight))
}

/// This function never returns `Err(UserCommand::back)`.
fn change_judgment_weights(settings: &mut BuildsGenerationSettings) -> Result<(), UserCommand> {
    loop {
        let input_weights: (Option<f32>, Option<f32>, Option<f32>) =
            match get_user_judgment_weights() {
                Ok(input_weights) => input_weights,
                Err(UserCommand::Back) => return Ok(()),
                Err(command) => return Err(command),
            };

        let old_judgment_weights: (f32, f32, f32) = settings.judgment_weights; //backup before checking validity
        if let Some(dps_weight) = input_weights.0 {
            settings.judgment_weights.0 = dps_weight;
        }
        if let Some(defense_weight) = input_weights.1 {
            settings.judgment_weights.1 = defense_weight;
        }
        if let Some(ms_weight) = input_weights.2 {
            settings.judgment_weights.2 = ms_weight;
        }

        if let Err(error_msg) = settings.check_settings() {
            println!("Failed to set judgment weights: {error_msg}");
            settings.judgment_weights = old_judgment_weights; //restore valid value
        } else {
            return Ok(());
        }
    }
}

const N_ITEMS_HELP_MSG: &str = "Generated builds will have the selected number of items.";

/// This function never returns `Err(UserCommand::back)`.
fn change_n_items(settings: &mut BuildsGenerationSettings) -> Result<(), UserCommand> {
    loop {
        let n_items: usize = match get_user_usize(
            "",
            "Enter a number of item per build",
            N_ITEMS_HELP_MSG,
            1..=MAX_UNIT_ITEMS,
            false,
        ) {
            Ok(Some(n_items)) => n_items,
            Ok(None) => return Ok(()), //should never get here because `allow_no_input` is false but just in case
            Err(UserCommand::Back) => return Ok(()),
            Err(command) => return Err(command),
        };

        let old_n_items: usize = settings.n_items; //backup before checking validity
        settings.n_items = n_items;

        if let Err(error_msg) = settings.check_settings() {
            println!("Failed to set number of items per build: {error_msg}");
            settings.n_items = old_n_items; //restore valid value
        } else {
            return Ok(());
        }
    }
}

const BOOTS_SLOT_HELP_MSG: &str = "Every generated build will have boots at the selected slot.\n\
If set to 0, the slot is not specified and boots are considered like any other regular item (thus not guaranteed to be in the generated builds depending on your settings).";

/// This function never returns `Err(UserCommand::back)`.
fn change_boots_slot(settings: &mut BuildsGenerationSettings) -> Result<(), UserCommand> {
    loop {
        let boots_slot: usize = match get_user_usize(
            "",
            "Enter a boots slot (or 0 if not specified)",
            BOOTS_SLOT_HELP_MSG,
            0..=MAX_UNIT_ITEMS,
            false,
        ) {
            Ok(Some(boots_slot)) => boots_slot,
            Ok(None) => return Ok(()), //should never get here because `allow_no_input` is false but just in case
            Err(UserCommand::Back) => return Ok(()),
            Err(command) => return Err(command),
        };

        let old_boots_slot: usize = settings.boots_slot; //backup before checking validity
        settings.boots_slot = boots_slot;

        if let Err(error_msg) = settings.check_settings() {
            println!("Failed to set boots slot: {error_msg}");
            settings.boots_slot = old_boots_slot; //restore valid value
        } else {
            return Ok(());
        }
    }
}

const SUPPORT_ITEM_SLOT_HELP_MSG: &str =
    "Every generated build will have a support item at the selected slot (or no support item if 0).";

/// This function never returns `Err(UserCommand::back)`.
fn change_support_item_slot(settings: &mut BuildsGenerationSettings) -> Result<(), UserCommand> {
    loop {
        let support_item_slot: usize = match get_user_usize(
            "",
            "Enter a support item slot (or 0 if none)",
            SUPPORT_ITEM_SLOT_HELP_MSG,
            0..=MAX_UNIT_ITEMS,
            false,
        ) {
            Ok(Some(support_item_slot)) => support_item_slot,
            Ok(None) => return Ok(()), //should never get here because `allow_no_input` is false but just in case
            Err(UserCommand::Back) => return Ok(()),
            Err(command) => return Err(command),
        };

        let old_support_item_slot: usize = settings.support_item_slot; //backup before checking validity
        settings.support_item_slot = support_item_slot;

        if let Err(error_msg) = settings.check_settings() {
            println!("Failed to set support item slot: {error_msg}");
            settings.support_item_slot = old_support_item_slot; //restore valid value
        } else {
            return Ok(());
        }
    }
}

const ALLOW_BOOTS_IF_NO_SLOT_HELP_MSG: &str =
    "If boots are allowed in the build when the boots slot is not specified (set to 0)";

const ALLOW_MANAFLOW_FIRST_ITEM_HELP_MSG: &str =
    "If manaflow items are allowed in first slot (overrides items pools if set to false)";

const MANDATORY_ITEMS_HELP_MSG: &str =
    "Every generated build will have the selected items at the specified slots.";

/// This function never returns `Err(UserCommand::back)`.
fn change_mandatory_items(settings: &mut BuildsGenerationSettings) -> Result<(), UserCommand> {
    //get item index first
    loop {
        let greeting_msg: String =
            format!("Current mandatory items are: {}", settings.mandatory_items);

        let item_slot:usize = match get_user_usize(
            &greeting_msg,
            "Enter an item slot where you want to impose an item (press enter to confirm current items)",
            MANDATORY_ITEMS_HELP_MSG,
            1..=MAX_UNIT_ITEMS,
            true,
        ) {
            Ok(Some(item_slot)) => item_slot,
            Ok(None) => return Ok(()),
            Err(UserCommand::Back) => return Ok(()),
            Err(command) => return Err(command),
        };
        let item_idx: usize = item_slot - 1;

        //get item and put it in mandatory items if valid
        loop {
            let item: &Item = match get_user_item(&format!(
                "Enter an item to impose at slot {item_slot} (press enter for none)"
            )) {
                Ok(item) => item,
                Err(UserCommand::Back) => break,
                Err(command) => return Err(command),
            };

            let old_item: &Item = settings.mandatory_items[item_idx]; //backup before checking validity
            settings.mandatory_items[item_idx] = item;

            if let Err(error_msg) = settings.check_settings() {
                println!("Failed to set mandatory items: {error_msg}");
                settings.mandatory_items[item_idx] = old_item; //restore valid value
            } else {
                break;
            }
        }
    }
}

const SEARCH_THRESHOLD_HELP_MSG: &str =
    "Controls the percentage of builds to explore among the possibilities during the generation process.\n\
     Higher value -> a higher number of badly performing builds are explored, may find better scaling builds but increases the computation time.\n\
     Lower value -> a lower number of badly performing builds are explored, may find worse scaling builds but decreases the computation time.\n\
     A search treshold percentage between 15-25% is generally sufficient to find most of the relevant builds.";

/// This function never returns `Err(UserCommand::back)`.
fn change_search_threshold(settings: &mut BuildsGenerationSettings) -> Result<(), UserCommand> {
    loop {
        let number: f32 = match get_user_f32(
            "",
            "Enter the search threshold percentage",
            SEARCH_THRESHOLD_HELP_MSG,
            false,
        ) {
            Ok(Some(number)) => number,
            Ok(None) => return Ok(()), //should never get here because `allow_no_input` is false but just in case
            Err(UserCommand::Back) => return Ok(()),
            Err(command) => return Err(command),
        };

        let old_search_threshold: f32 = settings.search_threshold; //backup before checking validity
        settings.search_threshold = number / 100.;

        if let Err(error_msg) = settings.check_settings() {
            println!("Failed to set search threshold: {error_msg}");
            settings.search_threshold = old_search_threshold; //restore valid value
        } else {
            return Ok(());
        }
    }
}
