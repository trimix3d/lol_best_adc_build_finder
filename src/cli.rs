use crate::RuneKeystone;

use super::builds_analyzer::*;
use super::champion_optimizer::*;
use super::game_data::*;

use items_data::*;
use runes_data::*;
use units_data::*;

use constcat::concat;
use enumset::{enum_set, EnumSet, EnumSetType};

use core::num::NonZeroUsize;
use core::ops::RangeBounds;
use std::io;

use io::Write;

///Character that represents a check mark.
pub(crate) const CHECK_MARK_CHAR: char = '●';
///Character that represents an unchecked mark.
pub(crate) const UNCHECKED_MARK_CHAR: char = ' ';
/// Number of builds to be printed by default when displaying results.
const DEFAULT_N_PRINTED_BUILDS: usize = 18;
/// Number of items used when automatically finding the best runes.
const N_ITEMS_WHEN_FINDING_BEST_RUNES: usize = 2;

const WELCOME_HELP_MSG: &str = "At any time, you can enter:\n\
                                -back/b: to go back to the previous menu.\n\
                                -help: to show help info on the current menu.\n\
                                -home: to return to the champion selection page (this page).\n\
                                -exit: to exit the program.";

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
    println!("{WELCOME_HELP_MSG}");

    let champ_names: Vec<&str> = Unit::ALL_CHAMPIONS
        .iter()
        .map(|champ_properties| champ_properties.name)
        .collect();

    let mut greetings_msg: String = String::from("Available champions: ");
    greetings_msg.push_str(champ_names[0]);
    for name in champ_names[1..].iter() {
        greetings_msg.push_str(", ");
        greetings_msg.push_str(name);
    }

    loop {
        match get_user_matching_input(
            &greetings_msg,
            "Enter the champion for which you want to find the best builds",
            "Please enter a valid champion name (among those available)",
            WELCOME_HELP_MSG,
            champ_names.iter().copied(),
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
            Err(UserCommand::Exit) => break,
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
            "help" | "?" => println!(
                "\n---[ HELP ]---\n{}",
                if help_msg.is_empty() {
                    "No help message available"
                } else {
                    help_msg
                }
            ),
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
        }
        match input.parse::<usize>() {
            Ok(number) => {
                if range.contains(&number) {
                    return Ok(Some(number));
                } else {
                    println!(
                        "{} is outside of range: {:?} to {:?}",
                        number,
                        range.start_bound(),
                        range.end_bound()
                    );
                }
            }
            Err(error) => println!("'{input}' is not a valid integer: {}", error),
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
        }
        match input.parse::<f32>() {
            Ok(number) => return Ok(Some(number)),
            Err(error) => println!("'{input}' is not a valid number: {}", error),
        }
    }
}

fn sanitize_item_name(name: &str) -> String {
    name.replace('_', " ") //replace underscores with spaces
        .replace(&['-', '\''][..], "") //remove - and '
        .to_lowercase()
}

#[derive(EnumSetType, Debug)]
enum ItemPoolType {
    Legendary,
    Boots,
    Support,
}

/// Prompts the user to enter an item name and returns the corresponding item.
fn get_user_item(
    greetings_msg: &str,
    input_line: &str,
    item_pool_types: EnumSet<ItemPoolType>,
) -> Result<&'static Item, UserCommand> {
    assert!(
        !item_pool_types.is_empty(),
        "Cannot choose an item from an empty pool"
    );

    let mut available_items: Vec<(&Item, String, String)> = Vec::new();
    if item_pool_types.contains(ItemPoolType::Legendary) {
        //ensure to match lowercase inputs with lowercase strings
        available_items.extend(items_data::ALL_LEGENDARY_ITEMS.iter().map(|&item| {
            (
                item,
                sanitize_item_name(item.full_name),
                sanitize_item_name(item.short_name),
            )
        }));
    }
    if item_pool_types.contains(ItemPoolType::Boots) {
        //ensure to match lowercase inputs with lowercase strings
        available_items.extend(items_data::ALL_BOOTS.iter().map(|&item| {
            (
                item,
                sanitize_item_name(item.full_name),
                sanitize_item_name(item.short_name),
            )
        }));
    }
    if item_pool_types.contains(ItemPoolType::Support) {
        //ensure to match lowercase inputs with lowercase strings
        available_items.extend(items_data::ALL_SUPPORT_ITEMS.iter().map(|&item| {
            (
                item,
                sanitize_item_name(item.full_name),
                sanitize_item_name(item.short_name),
            )
        }));
    }

    if !(greetings_msg.is_empty()) {
        println!("\n{greetings_msg}");
    }

    loop {
        let input: String = get_user_input(
            input_line,
            "Enter an item name (type 'list' to show available items)",
        )?;
        let sanitized_input: String = sanitize_item_name(&input);

        if sanitized_input.is_empty() {
            return Ok(&Item::NULL_ITEM);
        } else if let Some(item) =
            available_items
                .iter()
                .find(|(_item_ref, full_name, short_name)| {
                    *full_name == sanitized_input || *short_name == sanitized_input
                })
        {
            return Ok(item.0);
        } else if sanitized_input == "list" {
            //print list of items
            if item_pool_types.contains(ItemPoolType::Legendary) {
                println!("\nLegendary items in database:");
                for item in items_data::ALL_LEGENDARY_ITEMS {
                    println!("- {item:#}");
                }
            }

            if item_pool_types.contains(ItemPoolType::Boots) {
                println!("\nBoots in database:");
                for item in items_data::ALL_BOOTS {
                    println!("- {item:#}");
                }
            }

            if item_pool_types.contains(ItemPoolType::Support) {
                println!("\nSupport items in database:");
                for item in items_data::ALL_SUPPORT_ITEMS {
                    println!("- {item:#}");
                }
            }
        } else {
            println!("'{input}' is not a recognized item (type 'list' to show available items)");
        }
    }
}

/// Handle the whole build generation with the user.
/// This function never returns `Err(UserCommand::back)` because cannot go further back.
fn handle_builds_generation(champ_properties: &'static UnitProperties) -> Result<(), UserCommand> {
    //create build generation settings
    let mut settings: BuildsGenerationSettings =
        BuildsGenerationSettings::default_by_champion(champ_properties);

    loop {
        //set build generation settings
        match confirm_builds_generation_settings(&mut settings, champ_properties) {
            Ok(()) => (),
            Err(UserCommand::Back) => return Ok(()),
            Err(command) => return Err(command),
        };

        //compute best builds
        println!();
        let mut pareto_builds: Vec<BuildContainer> = match find_best_builds(
            champ_properties,
            &settings,
            false,
        ) {
            Ok(pareto_builds) => pareto_builds,
            Err(error_msg) => {
                get_user_raw_input(&format!(
                        "\nFailed to generate builds: {error_msg} (press enter to return to settings screen)"
                    ))?;
                continue;
            }
        };
        sort_builds_by_score(&mut pareto_builds, settings.judgment_weights);

        //results screen
        let mut n_to_print: NonZeroUsize = NonZeroUsize::new(DEFAULT_N_PRINTED_BUILDS)
            .expect("Failed to create NonZeroUsize from DEFAULT_N_PRINTED_BUILDS");
        let mut must_have_utils: EnumSet<ItemUtils> = enum_set!();
        loop {
            println!();
            print_builds_scores(
                &pareto_builds,
                champ_properties.name,
                settings.judgment_weights,
                n_to_print,
                must_have_utils,
            );

            //prompt for what's next
            let choice: usize = match get_user_choice(
                "",
                "Select an action (press enter to return to champion selection)",
                "How to interpret the columns from left to right:\n \
                    - score: the overall score of the build\n \
                    - !h/s : if the build has anti heal/shield utility\n \
                    - surv : if the build has survivability utility (e.g. zhonyas stasis, edge of night spell shield, ...)\n \
                    - spec : if the build has special utility (e.g. RFC bonus range, black cleaver armor reduction, ...)\n \
                    - build: the build in item order (with the score at each item slot in brackets)",
                [
                    &format!("only show builds with anti heal/shield utility: {}", must_have_utils.contains(ItemUtils::AntiHealShield)),
                    &format!("only show builds with survivability utility: {}", must_have_utils.contains(ItemUtils::Survivability)),
                    &format!("only show builds with special utility: {}", must_have_utils.contains(ItemUtils::Special)),
                    "choose the number of builds to show",
                    "restart build generation with different settings",
                ],
                true,
            ) {
                Ok(Some(choice)) => choice,
                Ok(None) => return Ok(()),
                Err(UserCommand::Back) => break,
                Err(command) => return Err(command),
            };

            match choice {
                1 => must_have_utils ^= enum_set!(ItemUtils::AntiHealShield),
                2 => must_have_utils ^= enum_set!(ItemUtils::Survivability),
                3 => must_have_utils ^= enum_set!(ItemUtils::Special),
                4 => match get_user_usize(
                    "",
                    "Enter the number of builds to show",
                    "",
                    1..,   //safety of a later unwrap depends on this range to exclude 0
                    false, //safety of a later expect() depends on this argument to be false
                ) {
                    Ok(n) => {
                        n_to_print = NonZeroUsize::new(
                            n.expect("Expected an input from user, but received none"),
                        )
                        .unwrap(); //should never panic as we prevent choice from being 0 above
                    }
                    Err(UserCommand::Back) => (),
                    Err(command) => return Err(command),
                },
                5 => break,
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
    "\n\n4) percentage of physical damage received: ",
    PHYS_DMG_RECEIVED_PERCENT_HELP_MSG,
    "\n\n5) go to runes settings: change rune keystone and rune shards.",
    "\n\n6) go to items settings: manage items rules (such as when boots must be purchased, which items are allowed, etc.)",
    "\n\n7) judgment weights: 3 values, first for DPS, second for defense and third for mobility.\n\
    These vales are used to weight the relative importance of DPS, defense and mobility of the champion in a single score value given to a build.\n\
    The weights are relative to each other, i.e. DPS 3, defense 2, mobility 1 is the same as DPS 1, defense 0.66, mobility 0.33",
    "\n\n8) search threshold: ",
    SEARCH_THRESHOLD_HELP_MSG
);

/// Show the build generation settings, prompt the user for any change and returns the settings when done.
fn confirm_builds_generation_settings(
    settings: &mut BuildsGenerationSettings,
    champ_properties: &'static UnitProperties,
) -> Result<(), UserCommand> {
    loop {
        let choice: usize = match get_user_choice(
            format!(
                "Build generation for {} will be launched with these settings:",
                champ_properties.name
            )
            .as_str(),
            "Select a setting to change (press enter to confirm current settings)",
            BUILDS_GENERATION_SETTINGS_HELP_MSG,
            [
                format!("target: {}", settings.target_properties.name).as_str(),
                format!(
                    "fight scenario: {}",
                    champ_properties.fight_scenarios[settings.fight_scenario_number.get() - 1].1
                )
                .as_str(),
                format!(
                    "fight duration: {}s{}",
                    settings.fight_duration,
                    if settings.fight_duration < LOW_FIGHT_DURATION_VALUE_WARNING {
                        format!(
                        " (/!\\ set to a low value (<{}s), this can lead to inaccurate results)",
                        LOW_FIGHT_DURATION_VALUE_WARNING
                    )
                    } else {
                        "".to_string()
                    }
                )
                .as_str(),
                format!(
                    "percentage of physical damage received (by {}): {:.0}%",
                    champ_properties.name,
                    100. * settings.phys_dmg_received_percent,
                )
                .as_str(),
                format!(
                    "go to runes settings (current keystone: {}) -->",
                    settings.runes_page.keystone
                )
                .as_str(),
                "go to items settings -->",
                format!(
                    "judgment weights: DPS {}, defense {}, mobility {}",
                    settings.judgment_weights.0,
                    settings.judgment_weights.1,
                    settings.judgment_weights.2
                )
                .as_str(),
                format!(
                    "search threshold: {:.0}%{}",
                    100. * settings.search_threshold,
                    if settings.search_threshold < LOW_SEARCH_THRESHOLD_VALUE_WARNING {
                        format!(
                        " (/!\\ set to a low value (<{:.0}%), this can lead to inaccurate results)",
                        100. * LOW_SEARCH_THRESHOLD_VALUE_WARNING
                    )
                    } else if settings.search_threshold > HIGH_SEARCH_THRESHOLD_VALUE_WARNING {
                        format!(
                        " (/!\\ set to a high value (>{:.0}%), this can result in long computation time)",
                        100. * HIGH_SEARCH_THRESHOLD_VALUE_WARNING
                    )
                    } else {
                        "".to_string()
                    }
                ).as_str(),
                "reset all settings to default",
            ],
            true,
        )? {
            Some(choice) => choice,
            None => return Ok(()),
        };

        match choice {
            1 => {
                //target
                change_target(settings, champ_properties)?;
            }
            2 => {
                //fight_scenario
                change_fight_scenario_number(settings, champ_properties)?;
            }
            3 => {
                //fight_duration
                change_fight_duration(settings, champ_properties)?;
            }
            4 => {
                //phys_dmg_received_percent
                change_phys_dmg_received_percent(settings, champ_properties)?;
            }
            5 => {
                //change runes
                handle_runes_settings(settings, champ_properties)?;
            }
            6 => {
                //items settings
                handle_items_settings(settings, champ_properties)?;
            }
            7 => {
                //judgment_weights
                change_judgment_weights(settings, champ_properties)?;
            }
            8 => {
                //search_threshold
                change_search_threshold(settings, champ_properties)?;
            }
            9 => {
                //reset all settings to default
                *settings = BuildsGenerationSettings::default_by_champion(champ_properties);
                println!("\nAll settings have been reset to default.");
            }
            _ => unreachable!("Unhandled user input"),
        }
    }
}

const TARGET_HELP_MSG: &str =
    "The selected target resistances will be used to compute the champion's DPS.";

/// This function never returns `Err(UserCommand::back)`.
fn change_target(
    settings: &mut BuildsGenerationSettings,
    champ_properties: &UnitProperties,
) -> Result<(), UserCommand> {
    loop {
        let choice: usize = match get_user_choice(
            "Available targets:",
            "Select a target",
            TARGET_HELP_MSG,
            TARGET_OPTIONS.iter().map(|properties| properties.name),
            false,
        ) {
            Ok(Some(choice)) => choice,
            Ok(None) => return Ok(()), //should never get here because `allow_no_input` is false
            Err(UserCommand::Back) => return Ok(()),
            Err(command) => return Err(command),
        };

        let old_target: &UnitProperties = settings.target_properties; //backup before checking validity
        settings.target_properties = TARGET_OPTIONS[choice - 1];

        if let Err(error_msg) = settings.check_settings(champ_properties) {
            println!("Failed to set target: {error_msg}");
            settings.target_properties = old_target; //restore valid value
        } else {
            return Ok(());
        }
    }
}

const FIGHT_SCENARIO_HELP_MSG: &str = "Each generated build will go through a fight simulation according to the selected scenario to evaluate its performance.\n\
        Therefore, the builds found will perform best for the selected scenario.";

/// This function never returns `Err(UserCommand::back)`.
fn change_fight_scenario_number(
    settings: &mut BuildsGenerationSettings,
    champ_properties: &UnitProperties,
) -> Result<(), UserCommand> {
    loop {
        let number: usize = match get_user_choice(
            &format!(
                "Available fight scenarios for {} are:",
                champ_properties.name
            ),
            &format!("Select a fight scenario for {}", champ_properties.name),
            FIGHT_SCENARIO_HELP_MSG,
            champ_properties
                .fight_scenarios
                .iter()
                .map(|scenario| scenario.1),
            false,
        ) {
            Ok(Some(choice)) => choice,
            Ok(None) => return Ok(()), //should never get here because `allow_no_input` is false
            Err(UserCommand::Back) => return Ok(()),
            Err(command) => return Err(command),
        };

        let old_fight_scenario_number: NonZeroUsize = settings.fight_scenario_number; //backup before checking validity
        settings.fight_scenario_number =
            NonZeroUsize::new(number).expect("Fight scenario number must be non-zero");

        if let Err(error_msg) = settings.check_settings(champ_properties) {
            println!("Failed to set fight scenario: {error_msg}");
            settings.fight_scenario_number = old_fight_scenario_number; //restore valid value
        } else {
            return Ok(());
        }
    }
}

const FIGHT_DURATION_HELP_MSG: &str =
    "Every build will be evaluated based on a fight simulation of the selected duration (in seconds).";

/// This function never returns `Err(UserCommand::back)`.
fn change_fight_duration(
    settings: &mut BuildsGenerationSettings,
    champ_properties: &UnitProperties,
) -> Result<(), UserCommand> {
    loop {
        let number: f32 =
            match get_user_f32("", "Enter a fight duration", FIGHT_DURATION_HELP_MSG, false) {
                Ok(Some(number)) => number,
                Ok(None) => return Ok(()), //should never get here because `allow_no_input` is false
                Err(UserCommand::Back) => return Ok(()),
                Err(command) => return Err(command),
            };

        let old_fight_duration: f32 = settings.fight_duration; //backup before checking validity
        settings.fight_duration = number;

        if let Err(error_msg) = settings.check_settings(champ_properties) {
            println!("Failed to set fight duration: {error_msg}");
            settings.fight_duration = old_fight_duration; //restore valid value
        } else {
            return Ok(());
        }
    }
}

const PHYS_DMG_RECEIVED_PERCENT_HELP_MSG: &str =
    "The selected percentage of physical dmg received will be considered when evaluating the defensive value of different builds.\n\
     The percentage of magic dmg received is deducted from this (assuming no true dmg received).";

/// This function never returns `Err(UserCommand::back)`.
fn change_phys_dmg_received_percent(
    settings: &mut BuildsGenerationSettings,
    champ_properties: &UnitProperties,
) -> Result<(), UserCommand> {
    loop {
        let number: f32 = match get_user_f32(
            "",
            format!(
                "Enter the percentage of physical dmg received by {}",
                champ_properties.name
            )
            .as_str(),
            PHYS_DMG_RECEIVED_PERCENT_HELP_MSG,
            false,
        ) {
            Ok(Some(number)) => number,
            Ok(None) => return Ok(()), //should never get here because `allow_no_input` is false
            Err(UserCommand::Back) => return Ok(()),
            Err(command) => return Err(command),
        };

        let old_phys_dmg_received_percent: f32 = settings.phys_dmg_received_percent; //backup before checking validity
        settings.phys_dmg_received_percent = number / 100.;

        if let Err(error_msg) = settings.check_settings(champ_properties) {
            println!("Failed to set percentage of physical dmg received: {error_msg}");
            settings.phys_dmg_received_percent = old_phys_dmg_received_percent; //restore valid value
        } else {
            return Ok(());
        }
    }
}

fn handle_runes_settings(
    settings: &mut BuildsGenerationSettings,
    champ_properties: &'static UnitProperties,
) -> Result<(), UserCommand> {
    loop {
        let choice: usize = match get_user_choice(
            "Runes settings:\n\
            (/!\\ runes shard = runes bonus stats, not the runes slots under the keystone)",
            "Select a setting to change (press enter to confirm current runes)",
            "",
            [
                format!("rune shard 1: {:?}", settings.runes_page.shard1).as_str(),
                format!("rune shard 2: {:?}", settings.runes_page.shard2).as_str(),
                format!("rune shard 3: {:?}", settings.runes_page.shard3).as_str(),
                format!("rune keystone: {:#}", settings.runes_page.keystone).as_str(),
                "automatically find the best runes keystones",
                "reset to default runes page",
            ],
            true,
        ) {
            Ok(Some(choice)) => choice,
            Ok(None) => return Ok(()),
            Err(UserCommand::Back) => return Ok(()),
            Err(command) => return Err(command),
        };

        match choice {
            1 => {
                //set rune shard 1
                change_rune_shard(settings, champ_properties, 1)?;
            }
            2 => {
                //set rune shard 2
                change_rune_shard(settings, champ_properties, 2)?;
            }
            3 => {
                //set rune shard 3
                change_rune_shard(settings, champ_properties, 3)?;
            }
            4 => {
                //change rune keystone
                change_rune_keystone(settings, champ_properties)?;
            }
            5 => {
                //find best runes keystones
                let best_keystones: Vec<(&RuneKeystone, f32)> = match find_best_runes_keystones(
                    champ_properties,
                    settings,
                    N_ITEMS_WHEN_FINDING_BEST_RUNES,
                ) {
                    Ok(best_keystones) => best_keystones,
                    Err(error_msg) => {
                        get_user_raw_input(&format!(
                            "\nFailed to find best runes keystones: {error_msg} (press enter to return to runes settings screen)"
                        ))?;
                        continue;
                    }
                };
                settings.runes_page.keystone = best_keystones[0].0; //should never go out of bounds since `runes_data::ALL_RUNES_KEYSTONES` should never be empty

                //print best runes keystone screen
                println!("\nBest runes keystone in order:");
                println!(
                    " - {:#} (score: {:.0}) - has replaced the previous setting",
                    best_keystones[0].0, best_keystones[0].1
                );
                for k in best_keystones[1..].iter() {
                    println!(" - {:#} (score: {:.0})", k.0, k.1);
                }

                get_user_raw_input("press enter to return to runes settings screen")?;
            }
            6 => {
                //reset to default runes page
                settings.runes_page = champ_properties.defaults.runes_pages;
                println!("\nRunes page has been reset to default.");
            }
            _ => unreachable!("Unhandled user input"),
        }
    }
}

fn change_rune_keystone(
    settings: &mut BuildsGenerationSettings,
    champ_properties: &UnitProperties,
) -> Result<(), UserCommand> {
    let keystone_choices: Vec<&RuneKeystone> = [
        &runes_data::ALL_RUNES_KEYSTONES[..],
        &[&RuneKeystone::EMPTY_RUNE_KEYSTONE][..],
    ]
    .concat();

    loop {
        let choice: usize = match get_user_choice(
            "Available rune keystones:",
            "Select a rune keystone",
            "",
            keystone_choices.iter().map(|keystone| keystone.full_name),
            false,
        ) {
            Ok(Some(choice)) => choice,
            Ok(None) => return Ok(()), //should never get here because `allow_no_input` is false
            Err(UserCommand::Back) => return Ok(()),
            Err(command) => return Err(command),
        };

        let old_runes: RunesPage = settings.runes_page; //backup before checking validity
        settings.runes_page.keystone = keystone_choices[choice - 1];

        if let Err(error_msg) = settings.check_settings(champ_properties) {
            println!("Failed to set rune shard: {error_msg}");
            settings.runes_page = old_runes; //restore valid value
        } else {
            return Ok(());
        }
    }
}

fn change_rune_shard(
    settings: &mut BuildsGenerationSettings,
    champ_properties: &UnitProperties,
    shard_number: usize,
) -> Result<(), UserCommand> {
    loop {
        let choice: RuneShard = match get_user_choice(
            "Available rune shards:",
            "Select a rune shard",
            "",
            ["Left", "Middle", "Right"],
            false,
        ) {
            Ok(Some(1)) => RuneShard::Left,
            Ok(Some(2)) => RuneShard::Middle,
            Ok(Some(3)) => RuneShard::Right,
            Ok(Some(_)) => unreachable!("Unhandled shard number"),
            Ok(None) => return Ok(()), //should never get here because `allow_no_input` is false
            Err(UserCommand::Back) => return Ok(()),
            Err(command) => return Err(command),
        };

        let old_runes: RunesPage = settings.runes_page; //backup before checking validity
        match shard_number {
            1 => settings.runes_page.shard1 = choice,
            2 => settings.runes_page.shard2 = choice,
            3 => settings.runes_page.shard3 = choice,
            _ => unreachable!("Unhandled shard number"),
        }

        if let Err(error_msg) = settings.check_settings(champ_properties) {
            println!("Failed to set rune shard: {error_msg}");
            settings.runes_page = old_runes; //restore valid value
        } else {
            return Ok(());
        }
    }
}

const ITEMS_POOLS_SETTINGS_HELP_MSG: &str = concat!(
    "Meaning of these settings:\n\
    1) number of items per build: ",
    N_ITEMS_HELP_MSG,
    "\n\n2) mandatory items: ",
    MANDATORY_ITEMS_HELP_MSG,
    "\n\n3) boots slot: ",
    BOOTS_SLOT_HELP_MSG,
    "\n\n4) support item slot: ",
    SUPPORT_ITEM_SLOT_HELP_MSG,
    "\n\n5) change allowed legendary items",
    "\n\n6) change allowed boots",
    "\n\n7) change allowed support items",
    "\n\n8) allow manaflow items in first slot: ",
    ALLOW_MANAFLOW_FIRST_ITEM_HELP_MSG,
);

fn handle_items_settings(
    settings: &mut BuildsGenerationSettings,
    champ_properties: &UnitProperties,
) -> Result<(), UserCommand> {
    loop {
        let choice: usize = match get_user_choice(
            "Item settings:",
            "Select a setting to change (press enter to confirm current settings)",
            ITEMS_POOLS_SETTINGS_HELP_MSG,
            [
                format!("number of items per build: {}", settings.n_items).as_str(),
                format!("mandatory items: {}", settings.mandatory_items).as_str(),
                format!("boots slot: {}", settings.boots_slot).as_str(),
                format!("support item slot: {}", settings.support_item_slot).as_str(),
                "change allowed legendary items -->",
                "change allowed boots -->",
                "change allowed support items -->",
                format!(
                    "allow manaflow items in first slot: {}",
                    settings.allow_manaflow_first_item
                )
                .as_str(),
                "reset to default items settings",
            ],
            true,
        ) {
            Ok(Some(choice)) => choice,
            Ok(None) => return Ok(()),
            Err(UserCommand::Back) => return Ok(()),
            Err(command) => return Err(command),
        };

        match choice {
            1 => {
                //n_items
                change_n_items(settings, champ_properties)?;
            }
            2 => {
                //mandatory_items
                change_mandatory_items(settings, champ_properties)?;
            }
            3 => {
                //boots_slot
                change_boots_slot(settings, champ_properties)?;
            }
            4 => {
                //support_item_slot
                change_support_item_slot(settings, champ_properties)?;
            }
            5 => {
                //change legendary items pool
                change_items_pool(ItemPoolType::Legendary, &mut settings.legendary_items_pool)?;
            }
            6 => {
                //change boots pool
                change_items_pool(ItemPoolType::Boots, &mut settings.boots_pool)?;
            }
            7 => {
                //change support items pool
                change_items_pool(ItemPoolType::Support, &mut settings.support_items_pool)?;
            }
            8 => {
                //flip allow_manaflow_first_item
                settings.allow_manaflow_first_item = !settings.allow_manaflow_first_item;
            }
            9 => {
                //reset to default items settings
                let default: BuildsGenerationSettings =
                    BuildsGenerationSettings::default_by_champion(champ_properties);

                settings.n_items = default.n_items;
                settings.mandatory_items = default.mandatory_items;
                settings.boots_slot = default.boots_slot;
                settings.support_item_slot = default.support_item_slot;
                settings.legendary_items_pool = default.legendary_items_pool;
                settings.boots_pool = default.boots_pool;
                settings.support_items_pool = default.support_items_pool;

                println!("\nItem settings have been reset to default.");
            }
            _ => unreachable!("Unhandled user input"),
        }
    }
}

const N_ITEMS_HELP_MSG: &str = "Generated builds will have the selected number of items.";

/// This function never returns `Err(UserCommand::back)`.
fn change_n_items(
    settings: &mut BuildsGenerationSettings,
    champ_properties: &UnitProperties,
) -> Result<(), UserCommand> {
    loop {
        let n_items: usize = match get_user_usize(
            "",
            "Enter a number of item per build",
            N_ITEMS_HELP_MSG,
            1..=MAX_UNIT_ITEMS,
            false,
        ) {
            Ok(Some(n_items)) => n_items,
            Ok(None) => return Ok(()), //should never get here because `allow_no_input` is false
            Err(UserCommand::Back) => return Ok(()),
            Err(command) => return Err(command),
        };

        let old_n_items: usize = settings.n_items; //backup before checking validity
        settings.n_items = n_items;

        if let Err(error_msg) = settings.check_settings(champ_properties) {
            println!("Failed to set number of items per build: {error_msg}");
            settings.n_items = old_n_items; //restore valid value
        } else {
            return Ok(());
        }
    }
}

const MANDATORY_ITEMS_HELP_MSG: &str =
    "Every generated build will have the selected items at the specified slots.";

/// This function never returns `Err(UserCommand::back)`.
fn change_mandatory_items(
    settings: &mut BuildsGenerationSettings,
    champ_properties: &UnitProperties,
) -> Result<(), UserCommand> {
    //get item index first
    loop {
        let item_slot: usize = match get_user_usize(
            "",
            format!(
                "Current mandatory items are: {}\n\
                Enter an item slot where you want to impose an item (press enter to confirm current items)",
                settings.mandatory_items
            )
            .as_str(),
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
            let item: &Item = match get_user_item(
                "",
                &format!("Enter an item to impose at slot {item_slot} (press enter for none)"),
                EnumSet::all(),
            ) {
                Ok(item) => item,
                Err(UserCommand::Back) => break,
                Err(command) => return Err(command),
            };

            let old_item: &Item = settings.mandatory_items[item_idx]; //backup before checking validity
            settings.mandatory_items[item_idx] = item;

            if let Err(error_msg) = settings.check_settings(champ_properties) {
                println!("Failed to set mandatory items: {error_msg}");
                settings.mandatory_items[item_idx] = old_item; //restore valid value
            } else {
                break;
            }
        }
    }
}

const BOOTS_SLOT_HELP_MSG: &str = "Every generated build will have boots at the selected slot.\n\
If set to 'Any', you let the optimizer decide which slot is best (it may even not pick any depending on your other settings).\n\
If set to 'None', boots are disallowed.";

fn get_item_slot(help_msg: &str) -> Result<ItemSlot, UserCommand> {
    loop {
        let input: String = get_user_input("Enter an item slot or 'any' or 'none'", help_msg)?;

        if input.is_empty() {
            println!("Please enter a valid item slot");
        }
        match input.as_str() {
            "any" => return Ok(ItemSlot::Any),
            "none" => return Ok(ItemSlot::None),
            input => match input.parse::<usize>() {
                Ok(number) => {
                    if (1..=MAX_UNIT_ITEMS).contains(&number) {
                        return Ok(ItemSlot::Slot(number));
                    } else {
                        println!(
                            "Item slot must be between 1 and {MAX_UNIT_ITEMS} (got {})",
                            number,
                        );
                    }
                }
                Err(error) => println!("'{input}' is not a valid integer: {}", error),
            },
        }
    }
}

/// This function never returns `Err(UserCommand::back)`.
fn change_boots_slot(
    settings: &mut BuildsGenerationSettings,
    champ_properties: &UnitProperties,
) -> Result<(), UserCommand> {
    loop {
        let boots_slot: ItemSlot = match get_item_slot(BOOTS_SLOT_HELP_MSG) {
            Ok(boots_slot) => boots_slot,
            Err(UserCommand::Back) => return Ok(()),
            Err(command) => return Err(command),
        };

        let old_boots_slot: ItemSlot = settings.boots_slot; //backup before checking validity
        settings.boots_slot = boots_slot;

        if let Err(error_msg) = settings.check_settings(champ_properties) {
            println!("Failed to set boots slot: {error_msg}");
            settings.boots_slot = old_boots_slot; //restore valid value
        } else {
            return Ok(());
        }
    }
}

const SUPPORT_ITEM_SLOT_HELP_MSG: &str = "Every generated build will have a support item at the selected slot.\n\
If set to 'Any', you let the optimizer decide which slot is best (it may even not pick any depending on your other settings).\n\
If set to 'None', support items are disallowed.";

/// This function never returns `Err(UserCommand::back)`.
fn change_support_item_slot(
    settings: &mut BuildsGenerationSettings,
    champ_properties: &UnitProperties,
) -> Result<(), UserCommand> {
    loop {
        let support_item_slot: ItemSlot = match get_item_slot(SUPPORT_ITEM_SLOT_HELP_MSG) {
            Ok(support_item_slot) => support_item_slot,
            Err(UserCommand::Back) => return Ok(()),
            Err(command) => return Err(command),
        };

        let old_support_item_slot: ItemSlot = settings.support_item_slot; //backup before checking validity
        settings.support_item_slot = support_item_slot;

        if let Err(error_msg) = settings.check_settings(champ_properties) {
            println!("Failed to set support item slot: {error_msg}");
            settings.support_item_slot = old_support_item_slot; //restore valid value
        } else {
            return Ok(());
        }
    }
}

fn change_items_pool(
    item_pool_types: ItemPoolType,
    pool: &mut Vec<&Item>,
) -> Result<(), UserCommand> {
    let reference_pool: &[&Item] = match item_pool_types {
        ItemPoolType::Legendary => &items_data::ALL_LEGENDARY_ITEMS,
        ItemPoolType::Boots => &items_data::ALL_BOOTS,
        ItemPoolType::Support => &items_data::ALL_SUPPORT_ITEMS,
    };

    loop {
        //print list of items with allowed/disallowed checkbox
        match item_pool_types {
            ItemPoolType::Legendary => {
                println!("\nAllowed legendary items:");
            }
            ItemPoolType::Boots => {
                println!("\nAllowed boots:");
            }
            ItemPoolType::Support => {
                println!("\nAllowed support items:");
            }
        }
        for item in reference_pool {
            println!(
                "[{}] {item:#}",
                if pool.contains(item) {
                    CHECK_MARK_CHAR
                } else {
                    UNCHECKED_MARK_CHAR
                }
            );
        }

        //get item
        let item: &Item = match get_user_item(
            "",
            "Enter an item to switch its allowance status (press enter to confirm current settings)",
            enum_set!(item_pool_types)
        ) {
            Ok(item) => item,
            Err(UserCommand::Back) => return Ok(()),
            Err(command) => return Err(command),
        };
        if *item == Item::NULL_ITEM {
            return Ok(());
        }

        //switch allowance status
        if let Some(index) = pool.iter().position(|&x| *x == *item) {
            pool.swap_remove(index);
        } else {
            pool.push(item);
        }
    }
}

const ALLOW_MANAFLOW_FIRST_ITEM_HELP_MSG: &str =
    "If manaflow items are allowed as first item. Only effective if there are manaflow items allowed in items pools.\n\
    This setting is overridden by mandatory items.";

const SEARCH_THRESHOLD_HELP_MSG: &str =
    "Controls the percentage of builds explored among the possibilities during the generation process.\n\
     Higher value -> a higher number of badly performing builds are explored, may find better scaling builds but increases the computation time and memory usage.\n\
     Lower value -> a lower number of badly performing builds are explored, may find worse scaling builds but decreases the computation time and memory usage.\n\
     A search treshold percentage between 15-25% is generally sufficient to find most of the relevant builds.";

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
fn change_judgment_weights(
    settings: &mut BuildsGenerationSettings,
    champ_properties: &UnitProperties,
) -> Result<(), UserCommand> {
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

        if let Err(error_msg) = settings.check_settings(champ_properties) {
            println!("Failed to set judgment weights: {error_msg}");
            settings.judgment_weights = old_judgment_weights; //restore valid value
        } else {
            return Ok(());
        }
    }
}

/// This function never returns `Err(UserCommand::back)`.
fn change_search_threshold(
    settings: &mut BuildsGenerationSettings,
    champ_properties: &UnitProperties,
) -> Result<(), UserCommand> {
    loop {
        let number: f32 = match get_user_f32(
            "",
            "Enter the search threshold percentage",
            SEARCH_THRESHOLD_HELP_MSG,
            false,
        ) {
            Ok(Some(number)) => number,
            Ok(None) => return Ok(()), //should never get here because `allow_no_input` is false
            Err(UserCommand::Back) => return Ok(()),
            Err(command) => return Err(command),
        };

        let old_search_threshold: f32 = settings.search_threshold; //backup before checking validity
        settings.search_threshold = number / 100.;

        if let Err(error_msg) = settings.check_settings(champ_properties) {
            println!("Failed to set search threshold: {error_msg}");
            settings.search_threshold = old_search_threshold; //restore valid value
        } else {
            return Ok(());
        }
    }
}
