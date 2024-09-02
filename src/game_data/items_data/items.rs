use super::{Item, ItemGroups, ItemId, ItemUtils};
use crate::game_data::*;

use buffs_data::{BuffId, BuffStackId, BuffValueId, TemporaryBuff};
use units_data::{capped_ms, DmgSource, RawDmg, Unit, UnitStats, MAX_UNIT_LVL};

use enumset::{enum_set, EnumSet};

//items parameters:
/// Average number of targets affected by the unmake passive.
const ABYSSAL_MASK_UNMAKE_AVG_TARGETS_IN_RANGE: f32 = 1.;
/// Percentage of dmg that is done in the passive range and profit from mr reduction.
const ABYSSAL_MASK_UNMAKE_PERCENT_OF_DMG_IN_RANGE: f32 = 0.80;
///x*x, where x is the % of hp under which it crits
const SHADOWFLAME_CINDERBLOOM_COEF: f32 = 0.35 * 0.35;
/// Percentage of target missing hp to account for the average dmg calculation.
const KRAKEN_SLAYER_BRING_IT_DOWN_AVG_TARGET_MISSING_HP_PERCENT: f32 = 0.30;
/// Actual duration of Malignance hatefog curse on the ennemy
const MALIGNANCE_HATEFOG_CURSE_TIME: f32 = 0.8;
/// Number of bolts fired by Runaan's hurricane wind's fury on average (adding to the primary basic attack).
pub const RUNAANS_HURRICANE_WINDS_FURY_AVG_BOLTS: f32 = 0.25;
/// Number of targets hit by profane hydra cleave aoe on average
const PROFANE_HYDRA_CLEAVE_AVG_TARGETS: f32 =
    basic_attack_aoe_effect_avg_additionnal_targets!(PROFANE_HYDRA_CLEAVE_RANGE);
/// Number of targets hit by ravenous hydra cleave aoe on average
const RAVENOUS_HYDRA_CLEAVE_AVG_TARGETS: f32 =
    basic_attack_aoe_effect_avg_additionnal_targets!(RAVENOUS_HYDRA_CLEAVE_RANGE);
/// Number of targets hit by stridebreaker cleave aoe on average
const STRIDEBREAKER_CLEAVE_AVG_TARGETS: f32 =
    basic_attack_aoe_effect_avg_additionnal_targets!(STRIDEBREAKER_CLEAVE_RANGE);
/// Number of targets hit by titanic hydra cleave aoe on average
const TITANIC_HYDRA_CLEAVE_AVG_TARGETS: f32 = 0.5;
/// % of mana considered during the activation of the shield (1. = 100%)
const SERAPHS_EMBRACE_LIFELINE_MANA_PERCENT: f32 = 0.5;
/// % of missing HP used to calculate the heal from Sundered sky lightshield strike
const SUNDERED_SKY_LIGHTSHIELD_STRIKE_MISSING_HP: f32 = 0.33;

//spellblade (generic functions for spellblade items)
//some lich bane spellblade functions are separate (because it modifies attack speed)
const SPELLBLADE_COOLDOWN: f32 = 1.5;
const SPELLBLADE_DELAY: f32 = 10.; //stack duration
fn spellblade_init(champ: &mut Unit) {
    champ.buffs_stacks[BuffStackId::SpellbladeEmpowered] = 0;
    champ.buffs_values[BuffValueId::SpellbladeLastEmpowerTime] = -(SPELLBLADE_DELAY + F32_TOL); //to allow for effect at time = 0.
    champ.buffs_values[BuffValueId::SpellbladeLastConsumeTime] = -(SPELLBLADE_COOLDOWN + F32_TOL);
    //to allow for effect at time = 0.
}

fn spellblade_on_spell_cast(champ: &mut Unit) {
    //if already empowered, update timer
    if champ.buffs_stacks[BuffStackId::SpellbladeEmpowered] == 1 {
        champ.buffs_values[BuffValueId::SpellbladeLastEmpowerTime] = champ.time;
    }
    //if not empowered (previous condition), empower next basic attack if not on cooldown
    else if champ.time - champ.buffs_values[BuffValueId::SpellbladeLastConsumeTime]
        > SPELLBLADE_COOLDOWN * haste_formula(champ.stats.item_haste)
    {
        champ.buffs_stacks[BuffStackId::SpellbladeEmpowered] = 1;
        champ.buffs_values[BuffValueId::SpellbladeLastEmpowerTime] = champ.time;
    }
}

/*
fn template_item_spellblade_on_basic_attack_hit(champ: &mut Unit, _target_stats: &UnitStats) -> RawDmg {
    //do nothing if not empowered
    if champ.buffs_stacks[BuffStackId::SpellbladeEmpowered] != 1 {
        return (0., 0., 0.);
    }
    //if empowered (previous condition) but last spell cast from too long ago, reset spellblade
    else if champ.time - champ.buffs_values[BuffValueId::SpellbladeLastEmpowerTime]
        >= SPELLBLADE_DELAY
    {
        champ.buffs_stacks[BuffStackId::SpellbladeEmpowered] = 0;
        return (0., 0., 0.);
    }
    //if empowered and last spell cast is recent enough (previous condition), reset and trigger spellblade
    champ.buffs_stacks[BuffStackId::SpellbladeEmpowered] = 0;
    champ.buffs_values[BuffValueId::SpellbladeLastConsumeTime] = champ.time;
    (
        0,
        0,
        0,
    )
}
*/

//
// --- ITEMS LISTING --- //
//
// from https://leagueoflegends.fandom.com/wiki/List_of_items
//

/*
//Template item
fn template_item_init(champ: &mut Unit) {
    champ.buffs_values[BuffValueId::TemplateItemEffectStat] = 0.;
    champ.add_temporary_buff(&TEMPLATE_BUFF, champ.stats.item_haste);
}

fn template_effect_enable(champ: &mut Unit, availability_coef: f32) {
    if champ.buffs_values[BuffValueId::TemplateItemEffectStat] == 0. {
        let some_stat_buff: f32 = availability_coef * some_value;
        champ.stats.some_stat += some_stat_buff;
        champ.buffs_values[BuffValueId::TemplateItemEffectStat] = some_stat_buff;
    }
}

fn template_effect_disable(champ: &mut Unit) {
    champ.stats.some_stat -= champ.buffs_values[BuffValueId::TemplateItemEffectStat];
    champ.buffs_values[BuffValueId::TemplateItemEffectStat] = 0.;
}

const TEMPLATE_BUFF: TemporaryBuff = TemporaryBuff {
    id: BuffId::TemplateItemEffect,
    add_stack: template_effect_enable,
    remove_every_stack: template_effect_disable,
    duration: some_duration,
    cooldown: some_cooldown,
};

pub const TEMPLATE_ITEM: Item = Item {
    id: ItemId::,
    full_name: "Template_item",
    short_name: "Template_item",
    cost: ,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
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
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};
*/

/*
//template energized item
fn template_energized_item_init(champ: &mut Unit) {
    champ.buffs_values[BuffValueId::TemplateEnergizedItemEnergizedPassiveLastTriggerDistance] =
        -(ENERGIZED_ATTACKS_TRAVEL_REQUIRED + F32_TOL); // to allow for effect at time == 0
}

fn template_energized_item_energized_passive(champ: &mut Unit, _target_stats: &UnitStats) -> RawDmg {
    //if not enough energy, add basic attack energy stacks
    if champ.sim_results.units_travelled
        - champ.buffs_values[BuffValueId::TemplateEnergizedItemEnergizedPassiveLastTriggerDistance]
        < ENERGIZED_ATTACKS_TRAVEL_REQUIRED
    {
        champ.buffs_values[BuffValueId::TemplateEnergizedItemEnergizedPassiveLastTriggerDistance] -=
            ENERGIZED_ATTACKS_TRAVEL_REQUIRED * 6. / 100.; //basic attacks generate 6 energy stacks
        return (0., 0., 0.);
    }
    //if enough energy (previous condition), trigger energized attack
    champ.buffs_values[BuffValueId::TemplateEnergizedItemEnergizedPassiveLastTriggerDistance] =
        champ.sim_results.units_travelled;
    (0, 0, 0)
}
*/

//Null item
/// For performance reason, we use a `NULL_ITEM` constant to represent empty items slots instead of an Option.
///
/// This is to avoid checking an Option everytime when working with items, since the majority of items aren't null.
pub const NULL_ITEM: Item = Item {
    id: ItemId::NullItem,
    full_name: "Null_item",
    short_name: "",
    cost: 0.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
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
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//todo: support items?

//Abyssal mask
pub const ABYSSAL_MASK: Item = Item {
    id: ItemId::AbyssalMask,
    full_name: "Abyssal_mask",
    short_name: "Abyssal_mask",
    cost: 2500.,
    item_groups: enum_set!(ItemGroups::Blight),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 300.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 50. + ABYSSAL_MASK_UNMAKE_AVG_TARGETS_IN_RANGE * 10.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 10.,
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
        mr_red_percent: ABYSSAL_MASK_UNMAKE_PERCENT_OF_DMG_IN_RANGE * 0.30,
        life_steal: 0.,
        omnivamp: 0.,
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Archangel staff not implemented (Seraph's embrace takes its place)

//Ardent censer (useless?)

//Axiom arc
pub const AXIOM_ARC: Item = Item {
    id: ItemId::AxiomArc,
    full_name: "Axiom_arc",
    short_name: "Axiom_arc",
    cost: 3000.,
    item_groups: enum_set!(),
    utils: enum_set!(ItemUtils::Other), //flux ultimate cd reduction passive
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 55.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 25.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.,
        lethality: 18.,
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
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Banshee's veil
pub const BANSHEES_VEIL: Item = Item {
    id: ItemId::BansheesVeil,
    full_name: "Banshees_veil",
    short_name: "Banshees",
    cost: 3100.,
    item_groups: enum_set!(ItemGroups::Annul),
    utils: enum_set!(ItemUtils::Survivability), //annul spellshield
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 120.,
        ap_coef: 0.,
        armor: 0.,
        mr: 50.,
        base_as: 0.,
        bonus_as: 0.,
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
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Black cleaver
fn black_cleaver_init(champ: &mut Unit) {
    champ.buffs_stacks[BuffStackId::BlackCleaverCarveStacks] = 0;
    champ.buffs_values[BuffValueId::BlackCleaverCarveArmorRedPercent] = 0.;
    champ.buffs_values[BuffValueId::BlackCleaverFervorMsFlat] = 0.;
}

const BLACK_CLEAVER_CARVE_P_ARMOR_RED_PER_STACK: f32 = 0.06;
fn black_cleaver_carve_add_stack(champ: &mut Unit, _availability_coef: f32) {
    if champ.buffs_stacks[BuffStackId::BlackCleaverCarveStacks] < 5 {
        champ.buffs_stacks[BuffStackId::BlackCleaverCarveStacks] += 1;
        champ.stats.armor_red_percent += BLACK_CLEAVER_CARVE_P_ARMOR_RED_PER_STACK;
        champ.buffs_values[BuffValueId::BlackCleaverCarveArmorRedPercent] +=
            BLACK_CLEAVER_CARVE_P_ARMOR_RED_PER_STACK;
    }
}

fn black_cleaver_carve_remove_every_stack(champ: &mut Unit) {
    champ.stats.armor_red_percent -=
        champ.buffs_values[BuffValueId::BlackCleaverCarveArmorRedPercent];
    champ.buffs_values[BuffValueId::BlackCleaverCarveArmorRedPercent] = 0.;
    champ.buffs_stacks[BuffStackId::BlackCleaverCarveStacks] = 0;
}

const BLACK_CLEAVER_CARVE: TemporaryBuff = TemporaryBuff {
    id: BuffId::BlackCleaverCarve,
    add_stack: black_cleaver_carve_add_stack,
    remove_every_stack: black_cleaver_carve_remove_every_stack,
    duration: 6.,
    cooldown: 0.,
};

fn black_cleaver_fervor_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.buffs_values[BuffValueId::BlackCleaverFervorMsFlat] == 0. {
        let flat_ms_buff: f32 = 20.;
        champ.stats.ms_flat += flat_ms_buff;
        champ.buffs_values[BuffValueId::BlackCleaverFervorMsFlat] = flat_ms_buff;
    }
}

fn black_cleaver_fervor_disable(champ: &mut Unit) {
    champ.stats.ms_flat -= champ.buffs_values[BuffValueId::BlackCleaverFervorMsFlat];
    champ.buffs_values[BuffValueId::BlackCleaverFervorMsFlat] = 0.;
}

const BLACK_CLEAVER_FERVOR: TemporaryBuff = TemporaryBuff {
    id: BuffId::BlackCleaverFervor,
    add_stack: black_cleaver_fervor_enable,
    remove_every_stack: black_cleaver_fervor_disable,
    duration: 2.,
    cooldown: 0.,
};

fn black_cleaver_on_ad_hit(champ: &mut Unit) {
    champ.add_temporary_buff(&BLACK_CLEAVER_CARVE, champ.stats.item_haste);
    champ.add_temporary_buff(&BLACK_CLEAVER_FERVOR, champ.stats.item_haste);
}

pub const BLACK_CLEAVER: Item = Item {
    id: ItemId::BlackCleaver,
    full_name: "Black_cleaver",
    short_name: "Black_cleaver",
    cost: 3000.,
    item_groups: enum_set!(ItemGroups::Fatality),
    utils: enum_set!(ItemUtils::Other), //carve armor reduction passive
    stats: UnitStats {
        hp: 400.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 55.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 20.,
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
    },
    init_item: Some(black_cleaver_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: Some(black_cleaver_on_ad_hit),
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Blackfire torch
const BLACKFIRE_TORCH_BALEFUL_BLAZE_DOT_DURATION: f32 = 3.;
fn blackfire_torch_init(champ: &mut Unit) {
    champ.buffs_values[BuffValueId::BlackfireTorchBalefulBlazeLastApplicationTime] =
        -(BLACKFIRE_TORCH_BALEFUL_BLAZE_DOT_DURATION + F32_TOL); //to allow for effect at time == 0
}

fn blackfire_torch_baleful_blaze(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    n_targets: f32,
) -> RawDmg {
    let dot_time: f32 = f32::min(
        BLACKFIRE_TORCH_BALEFUL_BLAZE_DOT_DURATION,
        champ.time - champ.buffs_values[BuffValueId::BlackfireTorchBalefulBlazeLastApplicationTime],
    ); //account for DoT overlap with the previous spell hit
    champ.buffs_values[BuffValueId::BlackfireTorchBalefulBlazeLastApplicationTime] = champ.time;
    (
        0.,
        n_targets * dot_time * (7.5 + 0.015 * champ.stats.ap()) * (1. / 0.5),
        0.,
    )
}

pub const BLACKFIRE_TORCH: Item = Item {
    id: ItemId::BlackfireTorch,
    full_name: "Blackfire_torch",
    short_name: "Blackfire_torch",
    cost: 2800.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 600.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 90.,
        ap_coef: 0.04, //assumes 1 ennemy is affected by passive
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 25.,
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
    },
    init_item: Some(blackfire_torch_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: Some(blackfire_torch_baleful_blaze),
    on_ultimate_spell_hit: Some(blackfire_torch_baleful_blaze),
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Blade of the ruined king
fn blade_of_the_ruined_king_mists_edge(_champion: &mut Unit, target_stats: &UnitStats) -> RawDmg {
    //when the champion attacks, target HP decreases, hence mists edge dmg also decreases over time
    //since we do not keep track of the target current HP, we need to find an average hp percent value to work with
    //this value depends on a number of factors (target resistances and champ basic attack damages, ...)
    //based on some analysis and by taking the worst case scenario (to ensure the item is good when the optimizer picks it)
    //average mists edge dmg would be roughly equivalent as if the target had constantly 40% hp
    (0.40 * 0.06 * target_stats.hp, 0., 0.) //value for ranged champions
}

pub const BLADE_OF_THE_RUINED_KING: Item = Item {
    id: ItemId::BladeOfTheRuinedKing,
    full_name: "Blade_of_the_ruined_king",
    short_name: "BRK",
    cost: 3200.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 50.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.25,
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
        life_steal: 0.10,
        omnivamp: 0.,
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: Some(blade_of_the_ruined_king_mists_edge),
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Bloodthirster
const BLOODTHIRSTER_ICHORSHIELD_MAX_SHIELD_BY_LVL: [f32; MAX_UNIT_LVL] = [
    50.,  //lvl 1 , wiki says 50.
    71.,  //lvl 2 , wiki says 50.
    91.,  //lvl 3 , wiki says 50.
    112., //lvl 4 , wiki says 50.
    132., //lvl 5 , wiki says 50.
    153., //lvl 6 , wiki says 50.
    174., //lvl 7 , wiki says 50.
    194., //lvl 8 , wiki says 50.
    215., //lvl 9 , wiki says 85.
    235., //lvl 10, wiki says 120.
    256., //lvl 11, wiki says 155.
    276., //lvl 12, wiki says 190.
    297., //lvl 13, wiki says 225.
    318., //lvl 14, wiki says 260.
    338., //lvl 15, wiki says 295.
    359., //lvl 16, wiki says 330.
    379., //lvl 17, wiki says 365.
    400., //lvl 18, wiki says 400.
]; //values in wiki are wrong, these actual values are from in game (patch 14.10)
fn bloodthirster_init(champ: &mut Unit) {
    //ichorshield passive
    champ.sim_results.heals_shields +=
        BLOODTHIRSTER_ICHORSHIELD_MAX_SHIELD_BY_LVL[usize::from(champ.lvl.get() - 1)];
}

pub const BLOODTHIRSTER: Item = Item {
    id: ItemId::Bloodthirster,
    full_name: "Bloodthirster",
    short_name: "BT",
    cost: 3400.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 80.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
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
        life_steal: 0.18,
        omnivamp: 0.,
    },
    init_item: Some(bloodthirster_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Chempunk chainsword
pub const CHEMPUNK_CHAINSWORD: Item = Item {
    id: ItemId::ChempunkChainsword,
    full_name: "Chempunk_chainsword",
    short_name: "Chempunk_chainsword",
    cost: 2800.,
    item_groups: enum_set!(),
    utils: enum_set!(ItemUtils::AntiHealShield),
    stats: UnitStats {
        hp: 250.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 55.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 15.,
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
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Cosmic drive
fn cosmic_drive_init(champ: &mut Unit) {
    champ.buffs_values[BuffValueId::CosmicDriveSpellDanceMsFlat] = 0.;
}

const COSMIC_DRIVE_SPELLDANCE_MS_FLAT_BY_LVL: [f32; MAX_UNIT_LVL] = [
    40., //lvl 1
    40., //lvl 2
    40., //lvl 3
    40., //lvl 4
    40., //lvl 5
    40., //lvl 6
    40., //lvl 7
    40., //lvl 8
    42., //lvl 9
    44., //lvl 10
    46., //lvl 11
    48., //lvl 12
    50., //lvl 13
    52., //lvl 14
    54., //lvl 15
    56., //lvl 16
    58., //lvl 17
    60., //lvl 18
];

fn cosmic_drive_spelldance_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.buffs_values[BuffValueId::CosmicDriveSpellDanceMsFlat] == 0. {
        let flat_ms_buff: f32 =
            COSMIC_DRIVE_SPELLDANCE_MS_FLAT_BY_LVL[usize::from(champ.lvl.get() - 1)];
        champ.stats.ms_flat += flat_ms_buff;
        champ.buffs_values[BuffValueId::CosmicDriveSpellDanceMsFlat] = flat_ms_buff;
    }
}

fn cosmic_drive_spelldance_disable(champ: &mut Unit) {
    champ.stats.ms_flat -= champ.buffs_values[BuffValueId::CosmicDriveSpellDanceMsFlat];
    champ.buffs_values[BuffValueId::CosmicDriveSpellDanceMsFlat] = 0.;
}

const COSMIC_DRIVE_SPELLDANCE: TemporaryBuff = TemporaryBuff {
    id: BuffId::CosmicDriveSpellDance,
    add_stack: cosmic_drive_spelldance_enable,
    remove_every_stack: cosmic_drive_spelldance_disable,
    duration: 2.,
    cooldown: 0.,
};

fn cosmic_drive_spelldance_on_spell_hit(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    _n_targets: f32,
) -> RawDmg {
    champ.add_temporary_buff(&COSMIC_DRIVE_SPELLDANCE, champ.stats.item_haste);
    (0., 0., 0.)
}

pub const COSMIC_DRIVE: Item = Item {
    id: ItemId::CosmicDrive,
    full_name: "Cosmic_drive",
    short_name: "Cosmic_drive",
    cost: 3000.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 350.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 80.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 25.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.05,
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
    init_item: Some(cosmic_drive_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: Some(cosmic_drive_spelldance_on_spell_hit),
    on_ultimate_spell_hit: Some(cosmic_drive_spelldance_on_spell_hit),
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Cryptbloom
pub const CRYPTBLOOM: Item = Item {
    id: ItemId::Cryptbloom,
    full_name: "Cryptbloom",
    short_name: "Cryptbloom",
    cost: 2850.,
    item_groups: enum_set!(ItemGroups::Blight),
    utils: enum_set!(ItemUtils::Other), //life from death healing passive
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 70.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 15.,
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
        magic_pen_percent: 0.30,
        armor_red_flat: 0.,
        armor_red_percent: 0.,
        mr_red_flat: 0.,
        mr_red_percent: 0.,
        life_steal: 0.,
        omnivamp: 0.,
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Dawncore (useless?) (need to add mana regen & heals_shields power (with Unit::heal_shield_on_ally fn?))

//Dead man's plate
const DEAD_MANS_PLATE_SHIPWRECKER_STACKS_PER_SEC: f32 = 7. / 0.25;
fn dead_mans_plate_init(champ: &mut Unit) {
    let ms: f32 = capped_ms(
        (champ.lvl_stats.ms_flat + champ.items_stats.ms_flat)
            * (1. + (champ.lvl_stats.ms_percent + champ.items_stats.ms_percent)),
    ); //can't use champ.ms() as it uses champ.stats that can be modified by other items init functions
    champ.buffs_values[BuffValueId::DeadMansPlateShipwreckerLastHitdistance] =
        -ms * 100. / DEAD_MANS_PLATE_SHIPWRECKER_STACKS_PER_SEC; //to allow for effect at time == 0
}

fn dead_mans_plate_shipwrecker(champ: &mut Unit, _target_stats: &UnitStats) -> RawDmg {
    let time_moving: f32 = (champ.sim_results.units_travelled
        - champ.buffs_values[BuffValueId::DeadMansPlateShipwreckerLastHitdistance])
        / champ.stats.ms();

    let stacks: f32 = f32::min(
        100.,
        DEAD_MANS_PLATE_SHIPWRECKER_STACKS_PER_SEC * time_moving,
    ); //bound stacks to 100.

    champ.buffs_values[BuffValueId::DeadMansPlateShipwreckerLastHitdistance] =
        champ.sim_results.units_travelled;
    ((stacks / 100.) * (40. + champ.stats.base_ad), 0., 0.)
}

pub const DEAD_MANS_PLATE: Item = Item {
    id: ItemId::DeadMansPlate,
    full_name: "Dead_mans_plate",
    short_name: "Dead_mans",
    cost: 2900.,
    item_groups: enum_set!(ItemGroups::Momentum),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 300.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 45.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.05,
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
    init_item: Some(dead_mans_plate_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: Some(dead_mans_plate_shipwrecker),
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Death's dance
pub const DEATHS_DANCE: Item = Item {
    id: ItemId::DeathsDance,
    full_name: "Deaths_dance",
    short_name: "Deaths_dance",
    cost: 3200.,
    item_groups: enum_set!(),
    utils: enum_set!(), //ignore pain passive not big enough utility
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 60.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 50.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 15.,
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
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Echoes of helia (useless?) (need to add Unit::heal_shield_on_ally fct & on_shield attributes to items)

//Eclipse
const ECLIPSE_EVER_RISING_MOON_COOLDOWN: f32 = 6.;
const ECLIPSE_EVER_RISING_MOON_DELAY: f32 = 2.; //stack duration

//const ECLIPSE_EVER_RISING_MOON_MAX_STACKS: u8 = 2;
fn eclipse_init(champ: &mut Unit) {
    champ.buffs_stacks[BuffStackId::EclipseEverRisingMoonStacks] = 0;
    champ.buffs_values[BuffValueId::EclipseEverRisingMoonLastStackTime] =
        -(ECLIPSE_EVER_RISING_MOON_DELAY + F32_TOL); //to allow for effect at time = 0.
    champ.buffs_values[BuffValueId::EclipseEverRisingMoonLastTriggerTime] =
        -(ECLIPSE_EVER_RISING_MOON_COOLDOWN + F32_TOL); //to allow for effect at time = 0.
}

fn eclipse_ever_rising_moon(champ: &mut Unit, target_stats: &UnitStats) -> RawDmg {
    //do nothing if on cooldown
    if champ.time - champ.buffs_values[BuffValueId::EclipseEverRisingMoonLastTriggerTime]
        <= ECLIPSE_EVER_RISING_MOON_COOLDOWN * haste_formula(champ.stats.item_haste)
    {
        return (0., 0., 0.);
    }
    //if last hit from too long ago, reset stacks and add 1
    else if champ.time - champ.buffs_values[BuffValueId::EclipseEverRisingMoonLastStackTime]
        >= ECLIPSE_EVER_RISING_MOON_DELAY
    {
        champ.buffs_stacks[BuffStackId::EclipseEverRisingMoonStacks] = 1;
        champ.buffs_values[BuffValueId::EclipseEverRisingMoonLastStackTime] = champ.time;
        return (0., 0., 0.);
    }

    //if last hit is recent enough (previous condition) but not fully stacked, add 1 stack (useless since max 2 stacks)
    //else if champ.buffs_stacks[BuffStackId::EclipseEverRisingMoonStacks]
    //    < ECLIPSE_EVER_RISING_MOON_MAX_STACKS - 1
    //{
    //    champ.buffs_stacks[BuffStackId::EclipseEverRisingMoonStacks] += 1;
    //    champ.buffs_values[BuffValueId::EclipseEverRisingMoonLastStackTime] = champ.time;
    //    return (0., 0., 0.);
    //}

    //if last hit is recent enough and fully stacked (previous condition), reset stacks and trigger ever rising moon
    champ.buffs_stacks[BuffStackId::EclipseEverRisingMoonStacks] = 0;
    champ.buffs_values[BuffValueId::EclipseEverRisingMoonLastTriggerTime] = champ.time;
    champ.sim_results.heals_shields += 80. + 0.2 * champ.stats.bonus_ad; //value for ranged champions
    (0.04 * target_stats.hp, 0., 0.)
}

pub const ECLIPSE: Item = Item {
    id: ItemId::Eclipse,
    full_name: "Eclipse",
    short_name: "Eclipse",
    cost: 2800.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 70.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 15.,
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
    },
    init_item: Some(eclipse_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: Some(eclipse_ever_rising_moon),
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Edge of night
pub const EDGE_OF_NIGHT: Item = Item {
    id: ItemId::EdgeOfNight,
    full_name: "Edge_of_night",
    short_name: "Edge_of_night",
    cost: 2800.,
    item_groups: enum_set!(ItemGroups::Annul),
    utils: enum_set!(ItemUtils::Survivability), //annul spellshield
    stats: UnitStats {
        hp: 250.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 50.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.,
        lethality: 15.,
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
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Essence reaver
pub const ESSENCE_REAVER: Item = Item {
    id: ItemId::EssenceReaver,
    full_name: "Essence_reaver",
    short_name: "ER",
    cost: 3100.,
    item_groups: enum_set!(),
    utils: enum_set!(ItemUtils::Other), //essence drain passive mana refund
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 70.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 25.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.25,
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
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Experimental hexplate
fn experimental_hexplate_init(champ: &mut Unit) {
    champ.buffs_values[BuffValueId::ExperimentalHexplateOverdriveBonusAS] = 0.;
    champ.buffs_values[BuffValueId::ExperimentalHexplateOverdriveMsPercent] = 0.;
}

fn experimental_hexplate_enable(champ: &mut Unit, availability_coef: f32) {
    if champ.buffs_values[BuffValueId::ExperimentalHexplateOverdriveBonusAS] == 0. {
        let bonus_as_buff: f32 = 0.30 * availability_coef;
        let percent_ms_buff: f32 = 0.15 * availability_coef;
        champ.stats.bonus_as += bonus_as_buff;
        champ.stats.ms_percent += percent_ms_buff;
        champ.buffs_values[BuffValueId::ExperimentalHexplateOverdriveBonusAS] = bonus_as_buff;
        champ.buffs_values[BuffValueId::ExperimentalHexplateOverdriveMsPercent] = percent_ms_buff;
    }
}

fn experimental_hexplate_disable(champ: &mut Unit) {
    champ.stats.bonus_as -= champ.buffs_values[BuffValueId::ExperimentalHexplateOverdriveBonusAS];
    champ.stats.ms_percent -=
        champ.buffs_values[BuffValueId::ExperimentalHexplateOverdriveMsPercent];
    champ.buffs_values[BuffValueId::ExperimentalHexplateOverdriveBonusAS] = 0.;
    champ.buffs_values[BuffValueId::ExperimentalHexplateOverdriveMsPercent] = 0.;
}

const EXPERIMENTAL_HEXPLATE_OVERDRIVE: TemporaryBuff = TemporaryBuff {
    id: BuffId::ExperimentalHexplateOverdrive,
    add_stack: experimental_hexplate_enable,
    remove_every_stack: experimental_hexplate_disable,
    duration: 8.,
    cooldown: 30.,
};

fn experimental_hexplate_overdrive_on_r_cast(champ: &mut Unit) {
    champ.add_temporary_buff(&EXPERIMENTAL_HEXPLATE_OVERDRIVE, champ.stats.item_haste);
}

pub const EXPERIMENTAL_HEXPLATE: Item = Item {
    id: ItemId::ExperimentalHexplate,
    full_name: "Experimental_hexplate",
    short_name: "Hexplate",
    cost: 3000.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 300.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 55.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.25,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 30., //hexcharged passive
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
    },
    init_item: Some(experimental_hexplate_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: Some(experimental_hexplate_overdrive_on_r_cast),
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Fimbulwinter (useless?)

//Force of nature (useless?)

//Frozen heart
pub const FROZEN_HEART: Item = Item {
    id: ItemId::FrozenHeart,
    full_name: "Frozen_heart",
    short_name: "Frozen_heart",
    cost: 2500.,
    item_groups: enum_set!(),
    utils: enum_set!(), //rock solid dmg mitigation passive not big enough
    stats: UnitStats {
        hp: 0.,
        mana: 400.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 65.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 20.,
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
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Guardian angel
pub const GUARDIAN_ANGEL: Item = Item {
    id: ItemId::GuardianAngel,
    full_name: "Guardian_angel",
    short_name: "GA",
    cost: 3200.,
    item_groups: enum_set!(),
    utils: enum_set!(ItemUtils::Survivability), //rebirth passive
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 55.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 45.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
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
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Guinsoo's rageblade
fn guinsoos_rageblade_init(champ: &mut Unit) {
    champ.buffs_stacks[BuffStackId::GuinsoosRagebladeSeethingStrikeStacks] = 0;
    champ.buffs_stacks[BuffStackId::GuinsoosRagebladePhantomStacks] = 0;
}

fn guinsoos_rageblade_wrath(_champ: &mut Unit, _target_stats: &UnitStats) -> RawDmg {
    (0., 30., 0.)
}

const GUINSOOS_RAGEBLADE_SEETHING_STRIKE_MAX_STACKS: u8 = 4;
const GUINSOOS_RAGEBLADE_SEETHING_STRIKE_BONUS_AS_PER_STACK: f32 = 0.08;
fn guinsoos_rageblade_seething_strike_add_stack(champ: &mut Unit, _availability_coef: f32) {
    if champ.buffs_stacks[BuffStackId::GuinsoosRagebladeSeethingStrikeStacks]
        < GUINSOOS_RAGEBLADE_SEETHING_STRIKE_MAX_STACKS
    {
        champ.buffs_stacks[BuffStackId::GuinsoosRagebladeSeethingStrikeStacks] += 1;
        champ.stats.bonus_as += GUINSOOS_RAGEBLADE_SEETHING_STRIKE_BONUS_AS_PER_STACK;
    }
}

fn guinsoos_rageblade_seething_strike_remove_every_stack(champ: &mut Unit) {
    champ.stats.bonus_as -=
        f32::from(champ.buffs_stacks[BuffStackId::GuinsoosRagebladeSeethingStrikeStacks])
            * GUINSOOS_RAGEBLADE_SEETHING_STRIKE_BONUS_AS_PER_STACK;
    champ.buffs_stacks[BuffStackId::GuinsoosRagebladeSeethingStrikeStacks] = 0;
    champ.buffs_stacks[BuffStackId::GuinsoosRagebladePhantomStacks] = 0;
}

const GUINSOOS_RAGEBLADE_SEETHING_STRIKE: TemporaryBuff = TemporaryBuff {
    id: BuffId::GuinsoosRagebladeSeethingStrike,
    add_stack: guinsoos_rageblade_seething_strike_add_stack,
    remove_every_stack: guinsoos_rageblade_seething_strike_remove_every_stack,
    duration: 3.,
    cooldown: 0.,
};

fn guinsoos_rageblade_seething_strike_on_basic_attack_hit(
    champ: &mut Unit,
    target_stats: &UnitStats,
) -> RawDmg {
    //seething strike buff (and stacks) must be applied first, phantom stacks second
    champ.add_temporary_buff(&GUINSOOS_RAGEBLADE_SEETHING_STRIKE, champ.stats.item_haste);

    //if seething strike is not fully stacked, do nothing more
    if champ.buffs_stacks[BuffStackId::GuinsoosRagebladeSeethingStrikeStacks]
        < GUINSOOS_RAGEBLADE_SEETHING_STRIKE_MAX_STACKS
    {
        return (0., 0., 0.);
    }
    //if seething strike is fully stacked (previous condition) but phantom stacks are not fully stacked, add 1 phantom stack
    else if champ.buffs_stacks[BuffStackId::GuinsoosRagebladePhantomStacks] < 2 {
        champ.buffs_stacks[BuffStackId::GuinsoosRagebladePhantomStacks] += 1;
        return (0., 0., 0.);
    }
    //if seething strike is fully stacked and phantom stacks are fully stacked (previous conditions), reset and return phantom hit dmg
    champ.buffs_stacks[BuffStackId::GuinsoosRagebladePhantomStacks] = 0;
    champ.get_items_on_basic_attack_hit_static(target_stats)
}

pub const GUINSOOS_RAGEBLADE: Item = Item {
    id: ItemId::GuinsoosRageblade,
    full_name: "Guinsoos_rageblade",
    short_name: "Guinsoos",
    cost: 3000.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 35.,
        ap_flat: 35.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.25,
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
    },
    init_item: Some(guinsoos_rageblade_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: Some(guinsoos_rageblade_wrath),
    on_basic_attack_hit_dynamic: Some(guinsoos_rageblade_seething_strike_on_basic_attack_hit),
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Heartsteel (useless?)

//Hextech Rocketbelt
fn hextech_rocketbelt_supersonic(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let availability_coef: f32 =
        effect_availability_formula(40. * haste_formula(champ.stats.item_haste));
    champ.sim_results.units_travelled += availability_coef * 275.; //maximum dash distance
    let ap_dmg: f32 = availability_coef * (100. + 0.1 * champ.stats.ap());
    champ.dmg_on_target(
        target_stats,
        (0., ap_dmg, 0.),
        (1, 1),
        DmgSource::BasicSpell,
        false,
        1.,
    )
}

pub const HEXTECH_ROCKETBELT: Item = Item {
    id: ItemId::HextechRocketbelt,
    full_name: "Hextech_rocketbelt",
    short_name: "Rocketbelt",
    cost: 2600.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 400.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 70.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 15.,
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
    },
    init_item: None,
    active: Some(hextech_rocketbelt_supersonic),
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Hollow radiance (useless?)

//Horizon focus, hypershot passive is implemented as a spell coef
fn horizon_focus_hypershot_spell_coef(_champ: &mut Unit) -> f32 {
    0.05 //less than the real value since we don't know if the spell hits at a far enough distance
}

pub const HORIZON_FOCUS: Item = Item {
    id: ItemId::HorizonFocus,
    full_name: "Horizon_focus",
    short_name: "Horizon_focus",
    cost: 2700.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 90.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 20.,
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
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: Some(horizon_focus_hypershot_spell_coef),
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Hubris, ego passive not implemented (too situationnal)
pub const HUBRIS: Item = Item {
    id: ItemId::Hubris,
    full_name: "Hubris",
    short_name: "Hubris",
    cost: 3000.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 60.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 15.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.,
        lethality: 18.,
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
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Hullbreaker, doesn't take into account skipper bonus dmg on structures
const HULLBREAKER_SKIPPER_DELAY: f32 = 10.; //stack duration
fn hullbreaker_init(champ: &mut Unit) {
    champ.buffs_stacks[BuffStackId::HullbreakerSkipperStacks] = 0;
    champ.buffs_values[BuffValueId::HullbreakerSkipperLastStackTime] =
        -(HULLBREAKER_SKIPPER_DELAY + F32_TOL); //to allow for effect at time = 0.
}

fn hullbreaker_skipper(champ: &mut Unit, _target_stats: &UnitStats) -> RawDmg {
    //if last hit from too long ago, reset stacks and add 1
    if champ.time - champ.buffs_values[BuffValueId::HullbreakerSkipperLastStackTime]
        >= HULLBREAKER_SKIPPER_DELAY
    {
        champ.buffs_stacks[BuffStackId::HullbreakerSkipperStacks] = 1;
        champ.buffs_values[BuffValueId::HullbreakerSkipperLastStackTime] = champ.time;
        return (0., 0., 0.);
    }
    //if last hit is recent enough (previous condition) but not fully stacked, add 1 stack
    else if champ.buffs_stacks[BuffStackId::HullbreakerSkipperStacks] < 4 {
        champ.buffs_stacks[BuffStackId::HullbreakerSkipperStacks] += 1;
        champ.buffs_values[BuffValueId::HullbreakerSkipperLastStackTime] = champ.time;
        return (0., 0., 0.);
    }
    //if fully stacked, (previous conditions) reset stacks and return skipper dmg
    champ.buffs_stacks[BuffStackId::HullbreakerSkipperStacks] = 0;
    (0.7 * champ.stats.base_ad + 0.035 * champ.stats.hp, 0., 0.) //value for ranged champions
}

pub const HULLBREAKER: Item = Item {
    id: ItemId::Hullbreaker,
    full_name: "Hullbreaker",
    short_name: "Hullbreaker",
    cost: 3000.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 350.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 65.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.05,
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
    init_item: Some(hullbreaker_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: Some(hullbreaker_skipper),
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Iceborn gauntlet
//no init function needed since we use generic spellblade variables
fn iceborn_gauntlet_spellblade_on_basic_attack_hit(
    champ: &mut Unit,
    _target_stats: &UnitStats,
) -> RawDmg {
    //do nothing if not empowered
    if champ.buffs_stacks[BuffStackId::SpellbladeEmpowered] != 1 {
        return (0., 0., 0.);
    }
    //if empowered (previous condition) but last spell cast from too long ago, reset spellblade
    else if champ.time - champ.buffs_values[BuffValueId::SpellbladeLastEmpowerTime]
        >= SPELLBLADE_DELAY
    {
        champ.buffs_stacks[BuffStackId::SpellbladeEmpowered] = 0;
        return (0., 0., 0.);
    }
    //if empowered and last spell cast is recent enough (previous condition), reset and trigger spellblade
    champ.buffs_stacks[BuffStackId::SpellbladeEmpowered] = 0;
    champ.buffs_values[BuffValueId::SpellbladeLastConsumeTime] = champ.time;
    (champ.stats.base_ad, 0., 0.)
}

pub const ICEBORN_GAUNTLET: Item = Item {
    id: ItemId::IcebornGauntlet,
    full_name: "Iceborn_gauntlet",
    short_name: "Iceborn_gauntlet",
    cost: 2600.,
    item_groups: enum_set!(ItemGroups::Spellblade),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 300.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 50.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 15.,
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
    },
    init_item: Some(spellblade_init),
    active: None,
    on_basic_spell_cast: Some(spellblade_on_spell_cast),
    on_ultimate_cast: Some(spellblade_on_spell_cast),
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: Some(iceborn_gauntlet_spellblade_on_basic_attack_hit),
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Immortal shieldbow
const IMMORTAL_SHIELDBOW_LIFELINE_SHIELD_BY_LVL: [f32; MAX_UNIT_LVL] = [
    320., //lvl 1
    320., //lvl 2
    320., //lvl 3
    320., //lvl 4
    320., //lvl 5
    320., //lvl 6
    320., //lvl 7
    320., //lvl 8
    360., //lvl 9
    400., //lvl 10
    440., //lvl 11
    480., //lvl 12
    520., //lvl 13
    560., //lvl 14
    600., //lvl 15
    640., //lvl 16
    680., //lvl 17
    720., //lvl 18
];
fn immortal_shieldbow_init(champ: &mut Unit) {
    //lifeline passive
    champ.sim_results.heals_shields += IMMORTAL_SHIELDBOW_LIFELINE_SHIELD_BY_LVL
        [usize::from(champ.lvl.get() - 1)]
        * effect_availability_formula(
            90. * haste_formula(champ.lvl_stats.item_haste + champ.items_stats.item_haste),
        );
}

pub const IMMORTAL_SHIELDBOW: Item = Item {
    id: ItemId::ImmortalShieldbow,
    full_name: "Immortal_shieldbow",
    short_name: "Shieldbow",
    cost: 3000.,
    item_groups: enum_set!(ItemGroups::Lifeline),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 55.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.25,
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
    },
    init_item: Some(immortal_shieldbow_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Imperial mandate (useless?)

//Infinity edge
pub const INFINITY_EDGE: Item = Item {
    id: ItemId::InfinityEdge,
    full_name: "Infinity_edge",
    short_name: "IE",
    cost: 3400.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 80.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.25,
        crit_dmg: 0.40,
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
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Jak'sho, voidborn resilience passive not implemented since i find it takes too much time to kick in for a dps
pub const JAKSHO: Item = Item {
    id: ItemId::Jaksho,
    full_name: "Jaksho",
    short_name: "Jaksho",
    cost: 3200.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 300.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 50.,
        mr: 50.,
        base_as: 0.,
        bonus_as: 0.,
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
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Kaenic Rookern
fn kaenic_rookern_init(champ: &mut Unit) {
    //magebane passive
    champ.sim_results.heals_shields += 0.18 * (champ.lvl_stats.hp + champ.items_stats.hp);
}

pub const KAENIC_ROOKERN: Item = Item {
    id: ItemId::KaenicRookern,
    full_name: "Kaenic_rookern",
    short_name: "Kaenic_rookern",
    cost: 2900.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        //todo: missing 150% base hp regeneration stat
        hp: 400.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 80.,
        base_as: 0.,
        bonus_as: 0.,
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
    },
    init_item: Some(kaenic_rookern_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Knight's vow (useles?)

//Kraken slayer
const KRAKEN_SLAYER_BRING_IT_DOWN_DELAY: f32 = 3.; //stack duration
fn kraken_slayer_init(champ: &mut Unit) {
    champ.buffs_stacks[BuffStackId::KrakenSlayerBringItDownStacks] = 0;
    champ.buffs_values[BuffValueId::KrakenSlayerBringItDownLastStackTime] =
        -(KRAKEN_SLAYER_BRING_IT_DOWN_DELAY + F32_TOL); //to allow for effect at time == 0
}

const KRAKEN_SLAYER_BRING_IT_DOWN_AD_DMG_BY_LVL: [f32; MAX_UNIT_LVL] = [
    140., //lvl 1
    140., //lvl 2
    140., //lvl 3
    140., //lvl 4
    140., //lvl 5
    140., //lvl 6
    140., //lvl 7
    140., //lvl 8
    157., //lvl 9
    174., //lvl 10
    191., //lvl 11
    208., //lvl 12
    225., //lvl 13
    242., //lvl 14
    259., //lvl 15
    276., //lvl 16
    293., //lvl 17
    310., //lvl 18
];
fn kraken_slayer_bring_it_down(champ: &mut Unit, _target_stats: &UnitStats) -> RawDmg {
    //if last hit from too long ago, reset stacks and add 1
    if champ.time - champ.buffs_values[BuffValueId::KrakenSlayerBringItDownLastStackTime]
        >= KRAKEN_SLAYER_BRING_IT_DOWN_DELAY
    {
        champ.buffs_stacks[BuffStackId::KrakenSlayerBringItDownStacks] = 1;
        champ.buffs_values[BuffValueId::KrakenSlayerBringItDownLastStackTime] = champ.time;
        return (0., 0., 0.);
    }
    //if last hit is recent enough (previous condition) but not fully stacked, add 1 stack
    else if champ.buffs_stacks[BuffStackId::KrakenSlayerBringItDownStacks] < 2 {
        champ.buffs_stacks[BuffStackId::KrakenSlayerBringItDownStacks] += 1;
        champ.buffs_values[BuffValueId::KrakenSlayerBringItDownLastStackTime] = champ.time;
        return (0., 0., 0.);
    }
    //if fully stacked (previous conditions), reset stacks, update coef and return bring it down dmg
    champ.buffs_stacks[BuffStackId::KrakenSlayerBringItDownStacks] = 0;
    let ad_dmg: f32 = (1. + 0.5 * KRAKEN_SLAYER_BRING_IT_DOWN_AVG_TARGET_MISSING_HP_PERCENT)
        * KRAKEN_SLAYER_BRING_IT_DOWN_AD_DMG_BY_LVL[usize::from(champ.lvl.get() - 1)];
    (0.80 * ad_dmg, 0., 0.) //value for ranged champions
}

pub const KRAKEN_SLAYER: Item = Item {
    id: ItemId::KrakenSlayer,
    full_name: "Kraken_slayer",
    short_name: "Kraken",
    cost: 3100.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 50.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.40,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.05,
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
    init_item: Some(kraken_slayer_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: Some(kraken_slayer_bring_it_down),
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Lidandry's torment
const LIANDRYS_TORMENT_TORMENT_DOT_DURATION: f32 = 3.;
const LIANDRYS_TORMENT_SUFFERING_DELAY: f32 = 3.; //duration after which passive deactivates
fn liandrys_torment_init(champ: &mut Unit) {
    champ.buffs_values[BuffValueId::LiandrysTormentTormentLastApplicationTime] =
        -(LIANDRYS_TORMENT_TORMENT_DOT_DURATION + F32_TOL); //to allow for effect at time == 0
    champ.buffs_values[BuffValueId::LiandrysTormentSufferingCombatStartTime] = 0.;
    champ.buffs_values[BuffValueId::LiandrysTormentSufferingLastHitTime] =
        -(LIANDRYS_TORMENT_SUFFERING_DELAY + F32_TOL); //to allow for effect at time == 0
}

fn liandrys_torment_torment(champ: &mut Unit, target_stats: &UnitStats, n_targets: f32) -> RawDmg {
    let dot_time: f32 = f32::min(
        LIANDRYS_TORMENT_TORMENT_DOT_DURATION,
        champ.time - champ.buffs_values[BuffValueId::LiandrysTormentTormentLastApplicationTime],
    ); //account for DoT overlap with the previous spell hit
    champ.buffs_values[BuffValueId::LiandrysTormentTormentLastApplicationTime] = champ.time;
    (
        0.,
        n_targets * dot_time * (0.01 / 0.5) * target_stats.hp,
        0.,
    )
}

fn liandrys_torment_suffering(champ: &mut Unit, _target_stats: &UnitStats) -> f32 {
    //if last hit from too long ago, reset combat
    if champ.time - champ.buffs_values[BuffValueId::LiandrysTormentSufferingLastHitTime]
        >= LIANDRYS_TORMENT_SUFFERING_DELAY
    {
        champ.buffs_values[BuffValueId::LiandrysTormentSufferingCombatStartTime] = champ.time;
        champ.buffs_values[BuffValueId::LiandrysTormentSufferingLastHitTime] = champ.time;
        return 0.;
    }
    //if last hit is recent enough (previous condition), return dmg coef based on the last combat start time
    champ.buffs_values[BuffValueId::LiandrysTormentSufferingLastHitTime] = champ.time;
    f32::min(
        0.06,
        0.02 * f32::round(
            champ.time - champ.buffs_values[BuffValueId::LiandrysTormentSufferingCombatStartTime],
        ),
    ) //as of patch 14.06, using round is the correct way to get the value
}

pub const LIANDRYS_TORMENT: Item = Item {
    id: ItemId::LiandrysTorment,
    full_name: "Liandrys_torment",
    short_name: "Liandrys",
    cost: 3000.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 300.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 90.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
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
    },
    init_item: Some(liandrys_torment_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: Some(liandrys_torment_torment),
    on_ultimate_spell_hit: Some(liandrys_torment_torment),
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: Some(liandrys_torment_suffering),
};

//Lich bane
const LICH_BANE_SPELLBLADE_BONUS_AS: f32 = 0.5;
fn lich_bane_spellblade_on_spell_cast(champ: &mut Unit) {
    //if already empowered, update timer
    if champ.buffs_stacks[BuffStackId::SpellbladeEmpowered] == 1 {
        champ.buffs_values[BuffValueId::SpellbladeLastEmpowerTime] = champ.time;
    }
    //if not empowered (previous condition), empower next basic attack if not on cooldown
    else if champ.time - champ.buffs_values[BuffValueId::SpellbladeLastConsumeTime]
        > SPELLBLADE_COOLDOWN * haste_formula(champ.stats.item_haste)
    {
        champ.buffs_stacks[BuffStackId::SpellbladeEmpowered] = 1;
        champ.stats.bonus_as += LICH_BANE_SPELLBLADE_BONUS_AS;
        champ.buffs_values[BuffValueId::SpellbladeLastEmpowerTime] = champ.time;
    }
}

fn lich_bane_spellblade_on_basic_attack_hit(champ: &mut Unit, _target_stats: &UnitStats) -> RawDmg {
    //do nothing if not empowered
    if champ.buffs_stacks[BuffStackId::SpellbladeEmpowered] != 1 {
        return (0., 0., 0.);
    }
    //if empowered (previous condition) but last spell cast from too long ago, reset spellblade
    else if champ.time - champ.buffs_values[BuffValueId::SpellbladeLastEmpowerTime]
        >= SPELLBLADE_DELAY
    {
        champ.buffs_stacks[BuffStackId::SpellbladeEmpowered] = 0;
        champ.stats.bonus_as -= LICH_BANE_SPELLBLADE_BONUS_AS;
        return (0., 0., 0.);
    }
    //if empowered and last spell cast is recent enough (previous condition), reset and trigger spellblade
    champ.buffs_stacks[BuffStackId::SpellbladeEmpowered] = 0;
    champ.stats.bonus_as -= LICH_BANE_SPELLBLADE_BONUS_AS;
    champ.buffs_values[BuffValueId::SpellbladeLastConsumeTime] = champ.time;
    (0., 0.75 * champ.stats.base_ad + 0.45 * champ.stats.ap(), 0.)
}

pub const LICH_BANE: Item = Item {
    id: ItemId::LichBane,
    full_name: "Lich_bane",
    short_name: "Lich_bane",
    cost: 3100.,
    item_groups: enum_set!(ItemGroups::Spellblade),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 100.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 15.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.08,
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
    init_item: Some(spellblade_init),
    active: None,
    on_basic_spell_cast: Some(lich_bane_spellblade_on_spell_cast),
    on_ultimate_cast: Some(lich_bane_spellblade_on_spell_cast),
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: Some(lich_bane_spellblade_on_basic_attack_hit),
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Locket of the iron solari (useless?)

//Lord dominik's regards
pub const LORD_DOMINIKS_REGARDS: Item = Item {
    id: ItemId::LordDominiksRegards,
    full_name: "Lord_dominiks_regards",
    short_name: "Dominiks",
    cost: 3000.,
    item_groups: enum_set!(ItemGroups::Fatality),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 45.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.25,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.,
        lethality: 0.,
        armor_pen_percent: 0.35,
        magic_pen_flat: 0.,
        magic_pen_percent: 0.,
        armor_red_flat: 0.,
        armor_red_percent: 0.,
        mr_red_flat: 0.,
        mr_red_percent: 0.,
        life_steal: 0.,
        omnivamp: 0.,
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Luden's companion
const LUDENS_COMPANION_FIRE_LOADED_STACKS: f32 = 6.; //f32 because it is directly used in f32 operations
const LUDENS_COMPANION_FIRE_STACKS_CHARGE_TIME: f32 = 12.; //f32 because it is directly used in f32 operations
fn ludens_companion_init(champ: &mut Unit) {
    champ.buffs_values[BuffValueId::LudensCompanionFireLastConsumeTime] =
        -(LUDENS_COMPANION_FIRE_STACKS_CHARGE_TIME + F32_TOL);
    //to allow for max stacks at time==0
}

fn ludens_companion_fire(champ: &mut Unit, _target_stats: &UnitStats, n_targets: f32) -> RawDmg {
    //if stacks not loaded, do nothing (previous condition), consume them and return fire dmg
    if champ.time - champ.buffs_values[BuffValueId::LudensCompanionFireLastConsumeTime]
        <= LUDENS_COMPANION_FIRE_STACKS_CHARGE_TIME
    {
        return (0., 0., 0.);
    }

    //if stacks loaded (previous condition), consume stacks
    champ.buffs_values[BuffValueId::LudensCompanionFireLastConsumeTime] = champ.time;
    let dmg: f32 = if n_targets >= LUDENS_COMPANION_FIRE_LOADED_STACKS {
        LUDENS_COMPANION_FIRE_LOADED_STACKS * (60. + 0.04 * champ.stats.ap())
    } else {
        n_targets * (60. + 0.04 * champ.stats.ap())
            + (LUDENS_COMPANION_FIRE_LOADED_STACKS - n_targets) * (30. + 0.02 * champ.stats.ap())
    };
    (0., dmg, 0.)
}

pub const LUDENS_COMPANION: Item = Item {
    id: ItemId::LudensCompanion,
    full_name: "Ludens_companion",
    short_name: "Ludens",
    cost: 2900.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 600.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 95.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 25.,
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
    },
    init_item: Some(ludens_companion_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: Some(ludens_companion_fire),
    on_ultimate_spell_hit: Some(ludens_companion_fire),
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Malignance
fn malignance_init(champ: &mut Unit) {
    champ.buffs_values[BuffValueId::MalignanceHatefogCurseMrRedFlat] = 0.;
}

const MALIGNANCE_HATEFOG_CURSE_F_MR_RED: f32 = 10.;
fn malignance_hatefog_curse_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.buffs_values[BuffValueId::MalignanceHatefogCurseMrRedFlat] == 0. {
        champ.stats.mr_red_flat += MALIGNANCE_HATEFOG_CURSE_F_MR_RED;
        champ.buffs_values[BuffValueId::MalignanceHatefogCurseMrRedFlat] =
            MALIGNANCE_HATEFOG_CURSE_F_MR_RED;
    }
}

fn malignance_hatefog_curse_disable(champ: &mut Unit) {
    champ.stats.mr_red_flat -= champ.buffs_values[BuffValueId::MalignanceHatefogCurseMrRedFlat];
    champ.buffs_values[BuffValueId::MalignanceHatefogCurseMrRedFlat] = 0.;
}

const MALIGNANCE_HATEFOG_CURSE: TemporaryBuff = TemporaryBuff {
    id: BuffId::MalignanceHatefogCurse,
    add_stack: malignance_hatefog_curse_enable,
    remove_every_stack: malignance_hatefog_curse_disable,
    duration: MALIGNANCE_HATEFOG_CURSE_TIME,
    cooldown: 3.,
};

fn malignance_hatefog(champ: &mut Unit, _target_stats: &UnitStats, n_targets: f32) -> RawDmg {
    //if on cooldown, do nothing
    if !champ.add_temporary_buff(&MALIGNANCE_HATEFOG_CURSE, champ.stats.item_haste) {
        return (0., 0., 0.);
    }
    //if not on cooldown (previous condition), return dmg
    (
        0.,
        n_targets * (MALIGNANCE_HATEFOG_CURSE_TIME / 0.25) * (15. + 0.0125 * champ.stats.ap()),
        0.,
    )
}

pub const MALIGNANCE: Item = Item {
    id: ItemId::Malignance,
    full_name: "Malignance",
    short_name: "Malignance",
    cost: 2700.,
    item_groups: enum_set!(),
    utils: enum_set!(ItemUtils::Other), //hatefog magic resistance reduction
    stats: UnitStats {
        hp: 0.,
        mana: 600.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 80.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 25.,
        basic_haste: 0.,
        ultimate_haste: 20.,
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
    },
    init_item: Some(malignance_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: Some(malignance_hatefog),
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Manamune not implemented (Muramana takes its place)

//Maw of malmortius
fn maw_of_malmortius_init(champ: &mut Unit) {
    //lifeline passive
    champ.sim_results.heals_shields += (150.
        + 1.6875 * (champ.lvl_stats.bonus_ad + champ.items_stats.bonus_ad))
        * effect_availability_formula(
            90. * haste_formula(champ.lvl_stats.item_haste + champ.items_stats.item_haste),
        );
    //value for ranged champions
}

pub const MAW_OF_MALMORTIUS: Item = Item {
    id: ItemId::MawOfMalmortius,
    full_name: "Maw_of_malmortius",
    short_name: "Malmortius",
    cost: 3100.,
    item_groups: enum_set!(ItemGroups::Lifeline),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 70.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 40.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 15.,
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
    },
    init_item: Some(maw_of_malmortius_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Mejai's soulstealer not implemented (too situationnal)

//Mercirial scimitar
pub const MERCURIAL_SCIMITAR: Item = Item {
    id: ItemId::MercurialScimitar,
    full_name: "Mercurial_scimitar",
    short_name: "Mercurial",
    cost: 3300.,
    item_groups: enum_set!(ItemGroups::Quicksilver),
    utils: enum_set!(ItemUtils::Survivability),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 40.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 50.,
        base_as: 0.,
        bonus_as: 0.,
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
        life_steal: 0.10,
        omnivamp: 0.,
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Mikael's blessing (useless?)

//Moonstone renewer (useless?)

//Morellonomicon
pub const MORELLONOMICON: Item = Item {
    id: ItemId::Morellonomicon,
    full_name: "Morellonomicon",
    short_name: "Morello",
    cost: 2200.,
    item_groups: enum_set!(),
    utils: enum_set!(ItemUtils::AntiHealShield),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 90.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 15.,
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
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Mortal reminder
pub const MORTAL_REMINDER: Item = Item {
    id: ItemId::MortalReminder,
    full_name: "Mortal_reminder",
    short_name: "Mortal_reminder",
    cost: 3000.,
    item_groups: enum_set!(ItemGroups::Fatality),
    utils: enum_set!(ItemUtils::AntiHealShield),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 35.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.25,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.,
        lethality: 0.,
        armor_pen_percent: 0.35,
        magic_pen_flat: 0.,
        magic_pen_percent: 0.,
        armor_red_flat: 0.,
        armor_red_percent: 0.,
        mr_red_flat: 0.,
        mr_red_percent: 0.,
        life_steal: 0.,
        omnivamp: 0.,
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Muramana
fn muramana_init(champ: &mut Unit) {
    champ.buffs_values[BuffValueId::MuramanaShockLastSpellHitTime] = -F32_TOL; //to allow for effect at time == 0

    //awe passive
    champ.stats.bonus_ad += 0.025 * (champ.lvl_stats.mana + champ.items_stats.mana);
}

fn muramana_shock_on_spell_hit(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    n_targets: f32,
) -> RawDmg {
    //set shock last spell hit, to prevent potential on basic attack hit effects triggered by this ability to apply shock twice
    champ.buffs_values[BuffValueId::MuramanaShockLastSpellHitTime] = champ.time;
    (
        n_targets * (0.027 * champ.stats.mana + 0.06 * champ.stats.ad()),
        0.,
        0.,
    ) //value for ranged champions
}

fn muramana_shock_on_basic_attack_hit(champ: &mut Unit, _target_stats: &UnitStats) -> RawDmg {
    //it is okay to have this condition in static on hit (exception)
    //if same instance of dmg as muramana_shock_on_spell_hit, do nothing (to prevent basic attack that trigger on hit to apply muramana passive twice)
    if champ.time == champ.buffs_values[BuffValueId::MuramanaShockLastSpellHitTime] {
        return (0., 0., 0.);
    }
    //if not the same instance, return dmg (no need to update shock last spell hit time since spells effects are called first)
    (0.015 * champ.stats.mana, 0., 0.)
}

pub const MURAMANA: Item = Item {
    id: ItemId::Muramana,
    full_name: "Muramana",
    short_name: "Muramana",
    cost: 2900.,
    item_groups: enum_set!(ItemGroups::Manaflow),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 860.,
        base_ad: 0.,
        bonus_ad: 35.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 15.,
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
    },
    init_item: Some(muramana_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: Some(muramana_shock_on_spell_hit),
    on_ultimate_spell_hit: Some(muramana_shock_on_spell_hit),
    on_basic_attack_hit_static: Some(muramana_shock_on_basic_attack_hit),
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Nashor's tooth
fn nashors_tooth_icathian_bite(champ: &mut Unit, _target_stats: &UnitStats) -> RawDmg {
    (0., 15. + 0.2 * champ.stats.ap(), 0.)
}

pub const NASHORS_TOOTH: Item = Item {
    id: ItemId::NashorsTooth,
    full_name: "Nashors_tooth",
    short_name: "Nashors",
    cost: 3000.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 90.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.50,
        ability_haste: 15.,
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
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: Some(nashors_tooth_icathian_bite),
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Navori flickerblade
const NAVORI_FLICKERBLADE_TRANSCENDENCE_CD_REFUND_PERCENT: f32 = 0.15;
fn navori_flickerblade_transcendence(champ: &mut Unit, _target_stats: &UnitStats) -> RawDmg {
    champ.q_cd *= 1. - NAVORI_FLICKERBLADE_TRANSCENDENCE_CD_REFUND_PERCENT;
    champ.w_cd *= 1. - NAVORI_FLICKERBLADE_TRANSCENDENCE_CD_REFUND_PERCENT;
    champ.e_cd *= 1. - NAVORI_FLICKERBLADE_TRANSCENDENCE_CD_REFUND_PERCENT;
    (0., 0., 0.)
}

pub const NAVORI_FLICKERBLADE: Item = Item {
    id: ItemId::NavoriFlickerblade,
    full_name: "Navori_flickerblade",
    short_name: "Navori",
    cost: 2600.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.40,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.25,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.07,
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
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: Some(navori_flickerblade_transcendence),
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Opportunity, extration passive not implemented (too situationnal)
fn opportunity_init(champ: &mut Unit) {
    champ.buffs_values[BuffValueId::OpportunityPreparationLethality] = 0.;

    //preparation passive
    champ.add_temporary_buff(
        &OPPORTUNITY_PREPARATION,
        champ.lvl_stats.item_haste + champ.items_stats.item_haste,
    );
}

const OPPORTUNITY_PREPARATION_LETHALITY_BY_LVL: [f32; MAX_UNIT_LVL] = [
    3.,  //lvl 1
    3.,  //lvl 2
    3.,  //lvl 3
    3.,  //lvl 4
    3.,  //lvl 5
    3.,  //lvl 6
    3.,  //lvl 7
    3.3, //lvl 8
    3.6, //lvl 9
    3.9, //lvl 10
    4.2, //lvl 11
    4.5, //lvl 12
    4.8, //lvl 13
    5.1, //lvl 14
    5.4, //lvl 15
    5.7, //lvl 16
    6.0, //lvl 17
    6.3, //lvl 18
]; //assumes ranged value
fn opportunity_preparation_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.buffs_values[BuffValueId::OpportunityPreparationLethality] == 0. {
        let lethality_buff: f32 =
            OPPORTUNITY_PREPARATION_LETHALITY_BY_LVL[usize::from(champ.lvl.get() - 1)];
        champ.stats.lethality += lethality_buff;
        champ.buffs_values[BuffValueId::OpportunityPreparationLethality] = lethality_buff;
    }
}

fn opportunity_preparation_disable(champ: &mut Unit) {
    champ.stats.lethality -= champ.buffs_values[BuffValueId::OpportunityPreparationLethality];
    champ.buffs_values[BuffValueId::OpportunityPreparationLethality] = 0.;
}

const OPPORTUNITY_PREPARATION: TemporaryBuff = TemporaryBuff {
    id: BuffId::OpportunityPreparation,
    add_stack: opportunity_preparation_enable,
    remove_every_stack: opportunity_preparation_disable,
    duration: 3.,
    cooldown: 0., //cooldown too small to be relevant (as of patch 14.08)
};

pub const OPPORTUNITY: Item = Item {
    id: ItemId::Opportunity,
    full_name: "Opportunity",
    short_name: "Opportunity",
    cost: 2700.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 55.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.05,
        lethality: 18.,
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
    init_item: Some(opportunity_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Overlord's bloodmail
fn overlords_bloodmail_init(champ: &mut Unit) {
    //tyranny passive
    champ.stats.bonus_ad += 0.02 * champ.items_stats.hp;

    //retribution passive not implemented (too situationnal)
}

pub const OVERLORDS_BLOODMAIL: Item = Item {
    id: ItemId::OverlordsBloodmail,
    full_name: "Overlords_bloodmail",
    short_name: "Overlords_bloodmail",
    cost: 3300.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 500.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 40.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
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
    },
    init_item: Some(overlords_bloodmail_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Phantom dancer
pub const PHANTOM_DANCER: Item = Item {
    id: ItemId::PhantomDancer,
    full_name: "Phantom_dancer",
    short_name: "PD",
    cost: 2600.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.60,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.25,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.12,
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
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Profane hydra
fn _profane_hydra_heretical_cleave(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    //we do not reduce the dmg value because the cd is short enough (10 sec, as of patch 14.06)
    champ.dmg_on_target(
        target_stats,
        (champ.stats.ad(), 0., 0.),
        (1, 1),
        DmgSource::Other,
        false,
        1.,
    ) //assumes the target is not under 50% hp (worst case scenario)
}

fn profane_hydra_cleave(champ: &mut Unit, _target_stats: &UnitStats) -> RawDmg {
    (
        PROFANE_HYDRA_CLEAVE_AVG_TARGETS * 0.25 * champ.stats.ad(),
        0.,
        0.,
    ) //value for ranged champions
}

pub const PROFANE_HYDRA_CLEAVE_RANGE: f32 = 450.; //used to determine how much targets are hit by cleave
pub const PROFANE_HYDRA: Item = Item {
    id: ItemId::ProfaneHydra,
    full_name: "Profane_hydra",
    short_name: "Profane_hydra",
    cost: 3300.,
    item_groups: enum_set!(ItemGroups::Hydra),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 60.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 20.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.,
        lethality: 18.,
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
    init_item: None,
    active: None, //Some(profane_hydra_heretical_cleave), //active not used for ranged champions
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: Some(profane_hydra_cleave),
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Rabadon's deathcap
pub const RABADONS_DEATHCAP: Item = Item {
    id: ItemId::RabadonsDeathcap,
    full_name: "Rabadons_deathcap",
    short_name: "Rabadons",
    cost: 3600.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 140.,
        ap_coef: 0.35,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
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
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Randuin's omen
pub const RANDUINS_OMEN: Item = Item {
    id: ItemId::RanduinsOmen,
    full_name: "Randuins_omen",
    short_name: "Randuins",
    cost: 2700.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 350.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 75.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
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
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Rapid firecannon
fn rapid_firecannon_init(champ: &mut Unit) {
    champ.buffs_values[BuffValueId::RapidFirecannonSharpshooterLastTriggerDistance] =
        -(ENERGIZED_ATTACKS_TRAVEL_REQUIRED + F32_TOL); // to allow for effect at time == 0
}

fn rapid_firecannon_sharpshooter(champ: &mut Unit, _target_stats: &UnitStats) -> RawDmg {
    //if not enough energy, add basic attack energy stacks
    if champ.sim_results.units_travelled
        - champ.buffs_values[BuffValueId::RapidFirecannonSharpshooterLastTriggerDistance]
        < ENERGIZED_ATTACKS_TRAVEL_REQUIRED
    {
        champ.buffs_values[BuffValueId::RapidFirecannonSharpshooterLastTriggerDistance] -=
            ENERGIZED_ATTACKS_TRAVEL_REQUIRED * 6. / 100.; //basic attacks generate 6 energy stacks
        return (0., 0., 0.);
    }
    //if enough energy (previous condition), trigger energized attack
    champ.buffs_values[BuffValueId::RapidFirecannonSharpshooterLastTriggerDistance] =
        champ.sim_results.units_travelled;
    (0., 60., 0.)
}

pub const RAPID_FIRECANNON: Item = Item {
    id: ItemId::RapidFirecannon,
    full_name: "Rapid_firecannon",
    short_name: "RFC",
    cost: 2600.,
    item_groups: enum_set!(),
    utils: enum_set!(ItemUtils::Other), //sharpshooter bonus range
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.35,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.25,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.07,
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
    init_item: Some(rapid_firecannon_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: Some(rapid_firecannon_sharpshooter),
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Ravenous hydra
fn _ravenous_hydra_ravenous_crescent(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    //we do not reduce the dmg value because the cd is short enough (10 sec, as of patch 14.06)
    champ.dmg_on_target(
        target_stats,
        (champ.stats.ad(), 0., 0.),
        (1, 1),
        DmgSource::Other,
        false,
        1.,
    )
}

fn ravenous_hydra_cleave(champ: &mut Unit, _target_stats: &UnitStats) -> RawDmg {
    (
        RAVENOUS_HYDRA_CLEAVE_AVG_TARGETS * 0.20 * champ.stats.ad(),
        0.,
        0.,
    ) //value for ranged champions
}

pub const RAVENOUS_HYDRA_CLEAVE_RANGE: f32 = 350.; //used to determine how much targets are hit by cleave
pub const RAVENOUS_HYDRA: Item = Item {
    id: ItemId::RavenousHydra,
    full_name: "Ravenous_hydra",
    short_name: "Ravenous_hydra",
    cost: 3300.,
    item_groups: enum_set!(ItemGroups::Hydra),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 70.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 20.,
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
        life_steal: 0.12,
        omnivamp: 0.,
    },
    init_item: None,
    active: None, //Some(ravenous_hydra_ravenous_crescent), //active not used for ranged champions
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: Some(ravenous_hydra_cleave),
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Redemption (useless?)

//Riftmaker
const RIFTMAKER_VOID_CORRUPTION_NOT_IN_COMBAT_TIME_VALUE: f32 = -1.; //special value to indicate that the unit is not in combat, MUST BE NEGATIVE to not interfere with an actual combat start time value
fn riftmaker_init(champ: &mut Unit) {
    champ.buffs_values[BuffValueId::RiftmakerVoidCorruptionOmnivamp] = 0.;
    champ.buffs_values[BuffValueId::RiftmakerVoidCorruptionCoef] = 0.;
    champ.buffs_values[BuffValueId::RiftmakerVoidCorruptionCombatStartTime] =
        RIFTMAKER_VOID_CORRUPTION_NOT_IN_COMBAT_TIME_VALUE;

    //void infusion passive
    champ.stats.ap_flat += 0.02 * champ.items_stats.hp;
}

const RIFTMAKER_VOID_CORRUPTION_MAX_COEF: f32 = 0.10;
const RIFTMAKER_VOID_CORRUPTION_OMNIVAMP: f32 = 0.06; //value for ranged champions
fn riftmaker_void_corruption_refresh(champ: &mut Unit, _availability_coef: f32) {
    //test if it's the first refresh of the buff, reset combat start time if so
    if champ.buffs_values[BuffValueId::RiftmakerVoidCorruptionCombatStartTime]
        == RIFTMAKER_VOID_CORRUPTION_NOT_IN_COMBAT_TIME_VALUE
    {
        champ.buffs_values[BuffValueId::RiftmakerVoidCorruptionCombatStartTime] = champ.time;
        champ.buffs_values[BuffValueId::RiftmakerVoidCorruptionCoef] = 0.;
        return;
    }
    //if not the first refresh (previous condition), update coef
    champ.buffs_values[BuffValueId::RiftmakerVoidCorruptionCoef] = f32::min(
        RIFTMAKER_VOID_CORRUPTION_MAX_COEF,
        0.02 * f32::trunc(
            champ.time - champ.buffs_values[BuffValueId::RiftmakerVoidCorruptionCombatStartTime],
        ),
    ); //as of patch 14.06, using trunc is the correct way to get the value

    //gain omnivamp if fully stacked
    if (champ.buffs_values[BuffValueId::RiftmakerVoidCorruptionOmnivamp] == 0.)
        && champ.buffs_values[BuffValueId::RiftmakerVoidCorruptionCoef]
            == RIFTMAKER_VOID_CORRUPTION_MAX_COEF
    {
        champ.stats.omnivamp += RIFTMAKER_VOID_CORRUPTION_OMNIVAMP;
        champ.buffs_values[BuffValueId::RiftmakerVoidCorruptionOmnivamp] =
            RIFTMAKER_VOID_CORRUPTION_OMNIVAMP;
    }
}

fn riftmaker_void_corruption_disable(champ: &mut Unit) {
    champ.stats.omnivamp -= champ.buffs_values[BuffValueId::RiftmakerVoidCorruptionOmnivamp];
    champ.buffs_values[BuffValueId::RiftmakerVoidCorruptionOmnivamp] = 0.;
    champ.buffs_values[BuffValueId::RiftmakerVoidCorruptionCoef] = 0.; //useless since we init it when refreshing buff for the first time but we do it for debug consistency
    champ.buffs_values[BuffValueId::RiftmakerVoidCorruptionCombatStartTime] =
        RIFTMAKER_VOID_CORRUPTION_NOT_IN_COMBAT_TIME_VALUE;
}

const RIFTMAKER_VOID_CORRUPTION: TemporaryBuff = TemporaryBuff {
    id: BuffId::RiftmakerVoidCorruption,
    add_stack: riftmaker_void_corruption_refresh,
    remove_every_stack: riftmaker_void_corruption_disable,
    duration: 4.,
    cooldown: 0.,
};

fn riftmaker_void_corruption(champ: &mut Unit, _target_stats: &UnitStats) -> f32 {
    champ.add_temporary_buff(&RIFTMAKER_VOID_CORRUPTION, champ.stats.item_haste);
    champ.buffs_values[BuffValueId::RiftmakerVoidCorruptionCoef]
}

pub const RIFTMAKER: Item = Item {
    id: ItemId::Riftmaker,
    full_name: "Riftmaker",
    short_name: "Riftmaker",
    cost: 3100.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 350.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 80.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 15.,
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
    },
    init_item: Some(riftmaker_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: Some(riftmaker_void_corruption),
};

//Rod of ages, timeless and eternity passives too complicated to implement
const ROD_OF_AGES_TIMELESS_COEF: f32 = 0.50; //proportion of the timeless additionnal stats we consider permanent for the item (since timeless passive is not implemented)
pub const ROD_OF_AGES: Item = Item {
    id: ItemId::RodOfAges,
    full_name: "Rod_of_ages",
    short_name: "RoA",
    cost: 2600.,
    item_groups: enum_set!(ItemGroups::Eternity),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 400. + ROD_OF_AGES_TIMELESS_COEF * 200.,
        mana: 400. + ROD_OF_AGES_TIMELESS_COEF * 200.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 50. + ROD_OF_AGES_TIMELESS_COEF * 40.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
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
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Runaan's hurricane
fn runaans_hurricane_winds_fury(champ: &mut Unit, target_stats: &UnitStats) -> RawDmg {
    let (
        on_basic_attack_hit_static_ad_dmg,
        on_basic_attack_hit_static_ap_dmg,
        on_basic_attack_hit_static_true_dmg,
    ) = champ.get_items_on_basic_attack_hit_static(target_stats);
    (
        RUNAANS_HURRICANE_WINDS_FURY_AVG_BOLTS
            * (0.55 * champ.stats.ad() * champ.stats.crit_coef()
                + on_basic_attack_hit_static_ad_dmg),
        RUNAANS_HURRICANE_WINDS_FURY_AVG_BOLTS * on_basic_attack_hit_static_ap_dmg,
        RUNAANS_HURRICANE_WINDS_FURY_AVG_BOLTS * on_basic_attack_hit_static_true_dmg,
    )
}

pub const RUNAANS_HURRICANE: Item = Item {
    id: ItemId::RunaansHurricane,
    full_name: "Runaans_hurricane",
    short_name: "Runaans",
    cost: 2600.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.40,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.25,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.07,
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
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: Some(runaans_hurricane_winds_fury),
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Rylais crystal scepter
pub const RYLAIS_CRYSTAL_SCEPTER: Item = Item {
    id: ItemId::RylaisCrystalScepter,
    full_name: "Rylais_crystal_scepter",
    short_name: "Rylais",
    cost: 2600.,
    item_groups: enum_set!(),
    utils: enum_set!(), //rimefrost slow not big enough
    stats: UnitStats {
        hp: 400.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 75.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
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
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Seraph's_embrace
fn seraphs_embrace_init(champ: &mut Unit) {
    //awe passive
    champ.stats.ap_flat += 0.02 * champ.items_stats.mana; //only take bonus mana into account

    //lifeline passive
    champ.sim_results.heals_shields += (250.
        + SERAPHS_EMBRACE_LIFELINE_MANA_PERCENT
            * 0.2
            * (champ.lvl_stats.mana + champ.items_stats.mana))
        * effect_availability_formula(
            90. * haste_formula(champ.lvl_stats.item_haste + champ.items_stats.item_haste),
        );
    //shield depends on current mana
}

pub const SERAPHS_EMBRACE: Item = Item {
    id: ItemId::SeraphsEmbrace,
    full_name: "Seraphs_embrace",
    short_name: "Seraphs",
    cost: 2900.,
    item_groups: enum_set!(ItemGroups::Lifeline | ItemGroups::Manaflow),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 1000.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 80.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 25.,
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
    },
    init_item: Some(seraphs_embrace_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Serpent's fang
pub const SERPENTS_FANG: Item = Item {
    id: ItemId::SerpentsFang,
    full_name: "Serpents_fang",
    short_name: "Serpents_fang",
    cost: 2500.,
    item_groups: enum_set!(),
    utils: enum_set!(ItemUtils::AntiHealShield),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 55.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.,
        lethality: 15.,
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
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Serylda's grudge
fn seryldas_grudge_init(champ: &mut Unit) {
    champ.stats.armor_pen_percent +=
        0.0011 * (champ.lvl_stats.lethality + champ.items_stats.lethality);
}

pub const SERYLDAS_GRUDGE: Item = Item {
    id: ItemId::SeryldasGrudge,
    full_name: "Seryldas_grudge",
    short_name: "Seryldas",
    cost: 3200.,
    item_groups: enum_set!(ItemGroups::Fatality),
    utils: enum_set!(), //bitter cold passive slow not big enough
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 45.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 15.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.,
        lethality: 15.,
        armor_pen_percent: 0.25,
        magic_pen_flat: 0.,
        magic_pen_percent: 0.,
        armor_red_flat: 0.,
        armor_red_percent: 0.,
        mr_red_flat: 0.,
        mr_red_percent: 0.,
        life_steal: 0.,
        omnivamp: 0.,
    },
    init_item: Some(seryldas_grudge_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Shadowflame
fn shadowflame_cinderbloom(champ: &mut Unit) -> f32 {
    SHADOWFLAME_CINDERBLOOM_COEF * (0.2 * (1. + champ.stats.crit_dmg - Unit::BASE_CRIT_DMG))
    //crit dmg above BASE_CRIT_DMG only affects only the bonus dmg of shadowflame not the entire dmg instance
}

pub const SHADOWFLAME: Item = Item {
    id: ItemId::Shadowflame,
    full_name: "Shadowflame",
    short_name: "Shadowflame",
    cost: 3200.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 120.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
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
        magic_pen_flat: 12.,
        magic_pen_percent: 0.,
        armor_red_flat: 0.,
        armor_red_percent: 0.,
        mr_red_flat: 0.,
        mr_red_percent: 0.,
        life_steal: 0.,
        omnivamp: 0.,
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: Some(shadowflame_cinderbloom),
    tot_dmg_coef: None,
};

//Shurelya's Battlesong (useless?)

//Solstice sleigh (useless?)

//Spear of shojin
const SPEAR_OF_SHOJIN_FOCUSED_WILL_DELAY: f32 = 6.; //stack duration
fn spear_of_shojin_init(champ: &mut Unit) {
    champ.buffs_stacks[BuffStackId::SpearOfShojinFocusedWillStacks] = 0;
    champ.buffs_values[BuffValueId::SpearOfShojinFocusedWillLastHitTime] =
        -(SPEAR_OF_SHOJIN_FOCUSED_WILL_DELAY + F32_TOL); //to allow for effect at time==0
}

const SPEAR_OF_SHOJIN_FOCUSED_WILL_SPELL_COEF_PER_STACK: f32 = 0.03;
fn spear_of_shojin_focused_will(champ: &mut Unit) -> f32 {
    //if last hit from too long ago, refresh duration and reset stacks
    if champ.time - champ.buffs_values[BuffValueId::SpearOfShojinFocusedWillLastHitTime]
        > SPEAR_OF_SHOJIN_FOCUSED_WILL_DELAY
    {
        champ.buffs_values[BuffValueId::SpearOfShojinFocusedWillLastHitTime] = champ.time;
        champ.buffs_stacks[BuffStackId::SpearOfShojinFocusedWillStacks] = 0; //first instance has 0 stack
        return 0.;
    }
    //if last hit is recent enough (previous condition), refresh duration and return coef (add 1 stack if not fully stacked)
    else if champ.buffs_stacks[BuffStackId::SpearOfShojinFocusedWillStacks] < 4 {
        champ.buffs_stacks[BuffStackId::SpearOfShojinFocusedWillStacks] += 1;
    }
    champ.buffs_values[BuffValueId::SpearOfShojinFocusedWillLastHitTime] = champ.time;
    f32::from(champ.buffs_stacks[BuffStackId::SpearOfShojinFocusedWillStacks])
        * SPEAR_OF_SHOJIN_FOCUSED_WILL_SPELL_COEF_PER_STACK //use value after adding a stack
}

pub const SPEAR_OF_SHOJIN: Item = Item {
    id: ItemId::SpearOfShojin,
    full_name: "Spear_of_shojin",
    short_name: "Shojin",
    cost: 3100.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 300.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 55.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 20.,
        basic_haste: 15.,
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
    },
    init_item: Some(spear_of_shojin_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: Some(spear_of_shojin_focused_will),
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Spirit visage (useless?) (need to add heals_shields power)

//Staff of flowing water (useless?) (need to add Unit::heal_shield_on_ally fct and heals_shields power)

//Statikk shiv (electroshock passive not implemented because too situationnal)
pub const STATIKK_SHIV: Item = Item {
    id: ItemId::StatikkShiv,
    full_name: "Statikk_shiv",
    short_name: "Statikk",
    cost: 2900.,
    item_groups: enum_set!(),
    utils: enum_set!(ItemUtils::Other), //electrospark wave clear
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 55.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.45,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.05,
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
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Sterak's gage
fn steraks_gage_init(champ: &mut Unit) {
    //the claw that catch passive
    champ.stats.bonus_ad += 0.5 * (champ.lvl_stats.base_ad + champ.items_stats.base_ad);

    //lifeline passive
    champ.sim_results.heals_shields += 0.5
        * 0.8
        * (champ.items_stats.hp)
        * effect_availability_formula(
            90. * haste_formula(champ.lvl_stats.item_haste + champ.items_stats.item_haste),
        );
    //actual value halved because shield decays, only counts bonus hp
}

pub const STERAKS_GAGE: Item = Item {
    id: ItemId::SteraksGage,
    full_name: "Steraks_gage",
    short_name: "Steraks",
    cost: 3200.,
    item_groups: enum_set!(ItemGroups::Lifeline),
    utils: enum_set!(),
    stats: UnitStats {
        //todo: missing tenacity stat (tenacity not implemented)
        hp: 400.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
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
    },
    init_item: Some(steraks_gage_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Stormsurge (assumes passive is always triggered)
fn stormsurge_init(champ: &mut Unit) {
    champ.buffs_values[BuffValueId::StormsurgeStormraiderMsPercent] = 0.;
    champ.buffs_stacks[BuffStackId::StormsurgeStormraiderTriggered] = 0;
}

fn stormsurge_stormraider_ms_enable(champ: &mut Unit, availability_coef: f32) {
    if champ.buffs_values[BuffValueId::StormsurgeStormraiderMsPercent] == 0. {
        let percent_ms_buff: f32 = availability_coef * 0.35;
        champ.stats.ms_percent += percent_ms_buff;
        champ.buffs_values[BuffValueId::StormsurgeStormraiderMsPercent] = percent_ms_buff;
    }
}

fn stormsurge_stormraider_ms_disable(champ: &mut Unit) {
    champ.stats.ms_percent -= champ.buffs_values[BuffValueId::StormsurgeStormraiderMsPercent];
    champ.buffs_values[BuffValueId::StormsurgeStormraiderMsPercent] = 0.;
}

const STORMSURGE_STORMRAIDER_MS_COOLDOWN: f32 = 30.;
const STORMSURGE_STORMRAIDER_MS: TemporaryBuff = TemporaryBuff {
    id: BuffId::StormsurgeStormraiderMS,
    add_stack: stormsurge_stormraider_ms_enable,
    remove_every_stack: stormsurge_stormraider_ms_disable,
    duration: 2.0,
    cooldown: STORMSURGE_STORMRAIDER_MS_COOLDOWN,
};

fn stormsurge_stormraider(champ: &mut Unit, _target_stats: &UnitStats) -> (f32, f32, f32) {
    //stormraider passive, triggers once, by a default 2.5sec after the first dmg instance since we don't record dmg done over time and cannot check the real activation condition
    if champ.buffs_stacks[BuffStackId::StormsurgeStormraiderTriggered] == 0 && champ.time > 2.5 {
        champ.buffs_stacks[BuffStackId::StormsurgeStormraiderTriggered] = 1;
        champ.add_temporary_buff(&STORMSURGE_STORMRAIDER_MS, champ.stats.item_haste);
        let avalability_coef: f32 = effect_availability_formula(
            STORMSURGE_STORMRAIDER_MS_COOLDOWN * haste_formula(champ.stats.item_haste),
        );
        return (
            0.,
            avalability_coef * 0.90 * (140. + 0.20 * champ.stats.ap()),
            0.,
        ); //ranged value
    }
    (0., 0., 0.)
}

pub const STORMSURGE: Item = Item {
    id: ItemId::Stormsurge,
    full_name: "Stormsurge",
    short_name: "Stormsurge",
    cost: 2900.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 95.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.08,
        lethality: 0.,
        armor_pen_percent: 0.,
        magic_pen_flat: 10.,
        magic_pen_percent: 0.,
        armor_red_flat: 0.,
        armor_red_percent: 0.,
        mr_red_flat: 0.,
        mr_red_percent: 0.,
        life_steal: 0.,
        omnivamp: 0.,
    },
    init_item: Some(stormsurge_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: Some(stormsurge_stormraider),
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Stridebreaker
fn stridebreaker_init(champ: &mut Unit) {
    champ.buffs_values[BuffValueId::StridebreakerBreakingShockwaveMsPercent] = 0.;
}

const STRIDEBREAKER_BREAKING_SHOCKWAVE_P_MS: f32 = 0.35;
fn stridebreaker_braking_shockwave_ms_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.buffs_values[BuffValueId::StridebreakerBreakingShockwaveMsPercent] == 0. {
        //ms buff halved because decays over time
        champ.stats.ms_percent += 0.5 * STRIDEBREAKER_BREAKING_SHOCKWAVE_P_MS;
        champ.buffs_values[BuffValueId::StridebreakerBreakingShockwaveMsPercent] =
            0.5 * STRIDEBREAKER_BREAKING_SHOCKWAVE_P_MS;
    }
}

fn stridebreaker_braking_shockwave_ms_disable(champ: &mut Unit) {
    champ.stats.ms_percent -=
        champ.buffs_values[BuffValueId::StridebreakerBreakingShockwaveMsPercent];
    champ.buffs_values[BuffValueId::StridebreakerBreakingShockwaveMsPercent] = 0.;
}

const STRIDEBREAKER_BREAKING_SHOCKWAVE_MS: TemporaryBuff = TemporaryBuff {
    id: BuffId::StridebreakerBreakingShockwaveMS,
    add_stack: stridebreaker_braking_shockwave_ms_enable,
    remove_every_stack: stridebreaker_braking_shockwave_ms_disable,
    duration: 3.,
    cooldown: 0.,
};

fn stridebreaker_breaking_shockwave(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    let dmg: f32 = champ.dmg_on_target(
        target_stats,
        (0.8 * champ.stats.ad(), 0., 0.),
        (1, 1),
        DmgSource::Other,
        false,
        1.,
    ); //calculate dmg before ms boost
    champ.add_temporary_buff(&STRIDEBREAKER_BREAKING_SHOCKWAVE_MS, champ.stats.item_haste);
    dmg
}

fn stridebreaker_cleave(champ: &mut Unit, _target_stats: &UnitStats) -> RawDmg {
    (
        STRIDEBREAKER_CLEAVE_AVG_TARGETS * 0.20 * champ.stats.ad(),
        0.,
        0.,
    ) //value for ranged champions
}

fn stridebreaker_temper_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.buffs_values[BuffValueId::StridebreakerTemperMsFlat] == 0. {
        let flat_ms_buff: f32 = 20.;
        champ.stats.ms_flat += flat_ms_buff;
        champ.buffs_values[BuffValueId::StridebreakerTemperMsFlat] = flat_ms_buff;
    }
}

fn stridebreaker_temper_disable(champ: &mut Unit) {
    champ.stats.ms_flat -= champ.buffs_values[BuffValueId::StridebreakerTemperMsFlat];
    champ.buffs_values[BuffValueId::StridebreakerTemperMsFlat] = 0.;
}

const STRIDEBREAKER_TEMPER: TemporaryBuff = TemporaryBuff {
    id: BuffId::StridebreakerTemper,
    add_stack: stridebreaker_temper_enable,
    remove_every_stack: stridebreaker_temper_disable,
    duration: 2.,
    cooldown: 0.,
};

fn stridebreaker_temper_on_ad_hit(champ: &mut Unit) {
    champ.add_temporary_buff(&STRIDEBREAKER_TEMPER, champ.stats.item_haste);
}

pub const STRIDEBREAKER_CLEAVE_RANGE: f32 = 350.;
pub const STRIDEBREAKER: Item = Item {
    id: ItemId::Stridebreaker,
    full_name: "Stridebreaker",
    short_name: "Stridebreaker",
    cost: 3300.,
    item_groups: enum_set!(ItemGroups::Hydra),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 450.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 50.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.30,
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
    },
    init_item: Some(stridebreaker_init),
    active: Some(stridebreaker_breaking_shockwave),
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: Some(stridebreaker_cleave),
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: Some(stridebreaker_temper_on_ad_hit),
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Sundered sky
const SUNDERED_SKY_COOLDOWN: f32 = 8.;
fn sundered_sky_init(champ: &mut Unit) {
    champ.buffs_values[BuffValueId::SunderedSkyLastTriggerTime] =
        -(SUNDERED_SKY_COOLDOWN + F32_TOL); //to allow for effect at time==0
}

fn sundered_sky_lightshield_strike(champ: &mut Unit, _target_stats: &UnitStats) -> RawDmg {
    //if on cooldown, do nothing
    if champ.time - champ.buffs_values[BuffValueId::SunderedSkyLastTriggerTime]
        <= SUNDERED_SKY_COOLDOWN
    {
        return (0., 0., 0.);
    }
    //if not on cooldown, put on cooldown and trigger effect
    champ.buffs_values[BuffValueId::SunderedSkyLastTriggerTime] = champ.time;
    champ.sim_results.heals_shields += 1.2 * champ.stats.base_ad
        + SUNDERED_SKY_LIGHTSHIELD_STRIKE_MISSING_HP * 0.06 * champ.stats.hp;
    let ad_dmg: f32 =
        champ.stats.ad() * (1. - champ.stats.crit_chance) * (champ.stats.crit_dmg - 1.); //bonus dmg from a basic attack with 100% crit chance compared to an average basic_attack
    (ad_dmg, 0., 0.)
}

pub const SUNDERED_SKY: Item = Item {
    id: ItemId::SunderedSky,
    full_name: "Sundered_sky",
    short_name: "Sundered_sky",
    cost: 3100.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 450.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 45.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 15.,
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
    },
    init_item: Some(sundered_sky_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: Some(sundered_sky_lightshield_strike),
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Sunfire aegis (useless?)

//Terminus
fn terminus_init(champ: &mut Unit) {
    champ.buffs_stacks[BuffStackId::TerminusJuxtapositionMode] = 0; //0==light, 1==dark
    champ.buffs_stacks[BuffStackId::TerminusJuxtapositionLightStacks] = 0;
    champ.buffs_stacks[BuffStackId::TerminusJuxtapositionDarkStacks] = 0;
    champ.buffs_values[BuffValueId::TerminusJuxtapositionLightRes] = 0.;
    champ.buffs_values[BuffValueId::TerminusJuxtapositionDarkPen] = 0.;
}

fn terminus_shadow(_champ: &mut Unit, _target_stats: &UnitStats) -> RawDmg {
    (0., 30., 0.)
}

const TERMINUS_JUXTAPOSITION_MAX_STACKS: u8 = 3;
const TERMINUS_JUXTAPOSITION_RES_PER_LIGHT_STACK_BY_LVL: [f32; MAX_UNIT_LVL] = [
    6., //lvl 1
    6., //lvl 2
    6., //lvl 3
    6., //lvl 4
    6., //lvl 5
    6., //lvl 6
    6., //lvl 7
    6., //lvl 8
    6., //lvl 9
    6., //lvl 10
    7., //lvl 11
    7., //lvl 12
    7., //lvl 13
    8., //lvl 14
    8., //lvl 15
    8., //lvl 16
    8., //lvl 17
    8., //lvl 18
];
fn terminus_juxtaposition_add_light_stack(champ: &mut Unit, _availability_coef: f32) {
    if champ.buffs_stacks[BuffStackId::TerminusJuxtapositionLightStacks]
        < TERMINUS_JUXTAPOSITION_MAX_STACKS
    {
        champ.buffs_stacks[BuffStackId::TerminusJuxtapositionLightStacks] += 1;
        let res_buff: f32 =
            TERMINUS_JUXTAPOSITION_RES_PER_LIGHT_STACK_BY_LVL[usize::from(champ.lvl.get() - 1)];
        champ.stats.armor += res_buff;
        champ.stats.mr += res_buff;
        champ.buffs_values[BuffValueId::TerminusJuxtapositionLightRes] += res_buff;
    }
}

fn terminus_juxtaposition_remove_every_light_stack(champ: &mut Unit) {
    champ.stats.armor -= champ.buffs_values[BuffValueId::TerminusJuxtapositionLightRes];
    champ.stats.mr -= champ.buffs_values[BuffValueId::TerminusJuxtapositionLightRes];
    champ.buffs_values[BuffValueId::TerminusJuxtapositionLightRes] = 0.;
    champ.buffs_stacks[BuffStackId::TerminusJuxtapositionLightStacks] = 0;
}

const TERMINUS_JUXTAPOSITION_DURATION: f32 = 5.;
const TERMINUS_JUXTAPOSITION_LIGHT: TemporaryBuff = TemporaryBuff {
    id: BuffId::TerminusJuxtapositionLight,
    add_stack: terminus_juxtaposition_add_light_stack,
    remove_every_stack: terminus_juxtaposition_remove_every_light_stack,
    duration: TERMINUS_JUXTAPOSITION_DURATION,
    cooldown: 0.,
};

const TERMINUS_JUXTAPOSITION_PEN_PER_DARK_STACK: f32 = 0.10;
fn terminus_juxtaposition_add_dark_stack(champ: &mut Unit, _availability_coef: f32) {
    if champ.buffs_stacks[BuffStackId::TerminusJuxtapositionDarkStacks]
        < TERMINUS_JUXTAPOSITION_MAX_STACKS
    {
        champ.buffs_stacks[BuffStackId::TerminusJuxtapositionDarkStacks] += 1;
        champ.stats.armor_pen_percent += TERMINUS_JUXTAPOSITION_PEN_PER_DARK_STACK;
        champ.stats.magic_pen_percent += TERMINUS_JUXTAPOSITION_PEN_PER_DARK_STACK;
        champ.buffs_values[BuffValueId::TerminusJuxtapositionDarkPen] +=
            TERMINUS_JUXTAPOSITION_PEN_PER_DARK_STACK;
    }
}

fn terminus_juxtaposition_remove_every_dark_stack(champ: &mut Unit) {
    champ.stats.armor_pen_percent -= champ.buffs_values[BuffValueId::TerminusJuxtapositionDarkPen];
    champ.stats.magic_pen_percent -= champ.buffs_values[BuffValueId::TerminusJuxtapositionDarkPen];
    champ.buffs_values[BuffValueId::TerminusJuxtapositionDarkPen] = 0.;
    champ.buffs_stacks[BuffStackId::TerminusJuxtapositionDarkStacks] = 0;
}

const TERMINUS_JUXTAPOSITION_DARK: TemporaryBuff = TemporaryBuff {
    id: BuffId::TerminusJuxtapositionDark,
    add_stack: terminus_juxtaposition_add_dark_stack,
    remove_every_stack: terminus_juxtaposition_remove_every_dark_stack,
    duration: TERMINUS_JUXTAPOSITION_DURATION,
    cooldown: 0.,
};

fn terminus_juxtaposition(champ: &mut Unit, _target_stats: &UnitStats) -> RawDmg {
    if champ.buffs_stacks[BuffStackId::TerminusJuxtapositionMode] == 0 {
        //add light stack and swap mode
        champ.add_temporary_buff(&TERMINUS_JUXTAPOSITION_LIGHT, champ.stats.item_haste);
        champ.buffs_stacks[BuffStackId::TerminusJuxtapositionMode] = 1;
    } else {
        //add dark stack and swap mode
        champ.add_temporary_buff(&TERMINUS_JUXTAPOSITION_DARK, champ.stats.item_haste);
        champ.buffs_stacks[BuffStackId::TerminusJuxtapositionMode] = 0;
    }
    (0., 0., 0.)
}

pub const TERMINUS: Item = Item {
    id: ItemId::Terminus,
    full_name: "Terminus",
    short_name: "Terminus",
    cost: 3000.,
    item_groups: enum_set!(ItemGroups::Blight | ItemGroups::Fatality),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 35.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.35,
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
    },
    init_item: Some(terminus_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: Some(terminus_shadow),
    on_basic_attack_hit_dynamic: Some(terminus_juxtaposition),
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//The collector
fn the_collector_init(champ: &mut Unit) {
    champ.buffs_stacks[BuffStackId::TheCollectorExecuted] = 0;
}

const THE_COLLECTOR_DEATH_EXECUTE_THRESHOLD: f32 = 0.05;
fn the_collector_death(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    if champ.buffs_stacks[BuffStackId::TheCollectorExecuted] != 1
        && champ.sim_results.dmg_done
            >= (1. - THE_COLLECTOR_DEATH_EXECUTE_THRESHOLD) * target_stats.hp
    {
        champ.sim_results.dmg_done += THE_COLLECTOR_DEATH_EXECUTE_THRESHOLD * target_stats.hp;
        champ.buffs_stacks[BuffStackId::TheCollectorExecuted] = 1;
    }
    0.
}

pub const THE_COLLECTOR: Item = Item {
    id: ItemId::TheCollector,
    full_name: "The_collector",
    short_name: "Collector",
    cost: 3200.,
    item_groups: enum_set!(),
    utils: enum_set!(), //taxes passive not big enough and too situationnal
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 60.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.25,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.,
        lethality: 12.,
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
    init_item: Some(the_collector_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: Some(the_collector_death),
};

//Thornmail (useless?)

//Titanic hydra
fn titanic_hydra_titanic_crescent(champ: &mut Unit, target_stats: &UnitStats) -> f32 {
    //only return bonus dmg (doesn't take into account if basic_attack hits multiple target e.g. with runaans but not a big deal)
    champ.dmg_on_target(
        target_stats,
        (
            0.02 * champ.stats.hp + TITANIC_HYDRA_CLEAVE_AVG_TARGETS * 0.045 * champ.stats.hp,
            0.,
            0.,
        ),
        (1, 1),
        DmgSource::Other,
        false,
        1. + TITANIC_HYDRA_CLEAVE_AVG_TARGETS,
    ) //value for ranged champions
}

fn titanic_hydra_cleave_static(champ: &mut Unit, _target_stats: &UnitStats) -> RawDmg {
    (0.0075 * champ.stats.hp, 0., 0.) //value for ranged champions
}

//AoE is dynamic because the area hits behind the target -> we consider that the areas won't overlap when hitting multiple targets
//so when a basic_attack hits mutiple ennemies (e.g. with runaans), only the first ennemy creates an AoE
fn titanic_hydra_cleave_dynamic(champ: &mut Unit, _target_stats: &UnitStats) -> RawDmg {
    (
        TITANIC_HYDRA_CLEAVE_AVG_TARGETS * 0.015 * champ.stats.hp,
        0.,
        0.,
    ) //value for ranged champions
}

pub const TITANIC_HYDRA: Item = Item {
    id: ItemId::TitanicHydra,
    full_name: "Titanic_hydra",
    short_name: "Titanic_hydra",
    cost: 3300.,
    item_groups: enum_set!(ItemGroups::Hydra),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 550.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 50.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
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
    },
    init_item: None,
    active: Some(titanic_hydra_titanic_crescent),
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: Some(titanic_hydra_cleave_static),
    on_basic_attack_hit_dynamic: Some(titanic_hydra_cleave_dynamic),
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Trailblazer (useless?)

//Trinity force
fn trinity_force_init(champ: &mut Unit) {
    //spellblade generic variables
    spellblade_init(champ);

    //quicken variables
    champ.buffs_values[BuffValueId::TrinityForceQuickenMsFlat] = 0.;
}

fn trinity_force_quicken_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.buffs_values[BuffValueId::TrinityForceQuickenMsFlat] == 0. {
        let flat_ms_buff: f32 = 20.;
        champ.stats.ms_flat += flat_ms_buff;
        champ.buffs_values[BuffValueId::TrinityForceQuickenMsFlat] = flat_ms_buff;
    }
}

fn trinity_force_quicken_disable(champ: &mut Unit) {
    champ.stats.ms_flat -= champ.buffs_values[BuffValueId::TrinityForceQuickenMsFlat];
    champ.buffs_values[BuffValueId::TrinityForceQuickenMsFlat] = 0.;
}

const TRINITY_FORCE_QUICKEN: TemporaryBuff = TemporaryBuff {
    id: BuffId::TrinityForceQuicken,
    add_stack: trinity_force_quicken_enable,
    remove_every_stack: trinity_force_quicken_disable,
    duration: 2.,
    cooldown: 0.,
};

fn trinity_force_spellblade_on_basic_attack_hit(
    champ: &mut Unit,
    _target_stats: &UnitStats,
) -> RawDmg {
    //quicken
    champ.add_temporary_buff(&TRINITY_FORCE_QUICKEN, champ.stats.item_haste);

    //spellblade
    //do nothing if not empowered
    if champ.buffs_stacks[BuffStackId::SpellbladeEmpowered] != 1 {
        return (0., 0., 0.);
    }
    //if empowered (previous condition) but last spell cast from too long ago, reset spellblade
    else if champ.time - champ.buffs_values[BuffValueId::SpellbladeLastEmpowerTime]
        >= SPELLBLADE_DELAY
    {
        champ.buffs_stacks[BuffStackId::SpellbladeEmpowered] = 0;
        return (0., 0., 0.);
    }
    //if empowered and last spell cast is recent enough (previous condition), reset and trigger spellblade
    champ.buffs_stacks[BuffStackId::SpellbladeEmpowered] = 0;
    champ.buffs_values[BuffValueId::SpellbladeLastConsumeTime] = champ.time;
    (2. * champ.stats.base_ad, 0., 0.)
}

pub const TRINITY_FORCE: Item = Item {
    id: ItemId::TrinityForce,
    full_name: "Trinity_force",
    short_name: "Triforce",
    cost: 3333.,
    item_groups: enum_set!(ItemGroups::Spellblade),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 300.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 45.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.33,
        ability_haste: 20.,
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
    },
    init_item: Some(trinity_force_init),
    active: None,
    on_basic_spell_cast: Some(spellblade_on_spell_cast),
    on_ultimate_cast: Some(spellblade_on_spell_cast),
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: Some(trinity_force_spellblade_on_basic_attack_hit),
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Umbral glaive
pub const UMBRAL_GLAIVE: Item = Item {
    id: ItemId::UmbralGlaive,
    full_name: "Umbral_glaive",
    short_name: "Umbral_glaive",
    cost: 2600.,
    item_groups: enum_set!(),
    utils: enum_set!(ItemUtils::Other), //blackout passive
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 50.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 15.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.,
        lethality: 15.,
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
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Unending despair (useless?)

//Vigilant wardstone (useless?)

//Void staff
pub const VOID_STAFF: Item = Item {
    id: ItemId::VoidStaff,
    full_name: "Void_staff",
    short_name: "Void_staff",
    cost: 3000.,
    item_groups: enum_set!(ItemGroups::Blight),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 95.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
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
        magic_pen_percent: 0.40,
        armor_red_flat: 0.,
        armor_red_percent: 0.,
        mr_red_flat: 0.,
        mr_red_percent: 0.,
        life_steal: 0.,
        omnivamp: 0.,
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Voltaic cyclosword
fn voltaic_cyclosword_init(champ: &mut Unit) {
    champ.buffs_values[BuffValueId::VoltaicCycloswordFirmamentLastTriggerDistance] =
        -(ENERGIZED_ATTACKS_TRAVEL_REQUIRED + F32_TOL); // to allow for effect at time == 0
}

fn voltaic_cyclosword_firmament(champ: &mut Unit, _target_stats: &UnitStats) -> RawDmg {
    //if not enough energy, add basic attack energy stacks
    if champ.sim_results.units_travelled
        - champ.buffs_values[BuffValueId::VoltaicCycloswordFirmamentLastTriggerDistance]
        < ENERGIZED_ATTACKS_TRAVEL_REQUIRED
    {
        champ.buffs_values[BuffValueId::VoltaicCycloswordFirmamentLastTriggerDistance] -=
            ENERGIZED_ATTACKS_TRAVEL_REQUIRED * 6. / 100.; //basic attacks generate 6 energy stacks
        return (0., 0., 0.);
    }
    //if enough energy (previous condition), trigger energized attack
    champ.buffs_values[BuffValueId::VoltaicCycloswordFirmamentLastTriggerDistance] =
        champ.sim_results.units_travelled;
    (100., 0., 0.) //slow not implemented
}

pub const VOLTAIC_CYCLOSWORD: Item = Item {
    id: ItemId::VoltaicCyclosword,
    full_name: "Voltaic_cyclosword",
    short_name: "Voltaic_cyclosword",
    cost: 2900.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 55.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 15.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.,
        lethality: 18.,
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
    init_item: Some(voltaic_cyclosword_init),
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: Some(voltaic_cyclosword_firmament),
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Warmog's armor (useless?)

//Winter's approach not implemented (Fimbulwinter takes its place)

//Wit's end
const WITS_END_FRAY_AP_DMG_BY_LVL: [f32; MAX_UNIT_LVL] = [
    40., //lvl 1
    40., //lvl 2
    40., //lvl 3
    40., //lvl 4
    40., //lvl 5
    40., //lvl 6
    40., //lvl 7
    40., //lvl 8
    44., //lvl 9
    48., //lvl 10
    52., //lvl 11
    56., //lvl 12
    60., //lvl 13
    64., //lvl 14
    68., //lvl 15
    72., //lvl 16
    76., //lvl 17
    80., //lvl 18
];
fn wits_end_fray(champ: &mut Unit, _target_stats: &UnitStats) -> RawDmg {
    (
        0.,
        WITS_END_FRAY_AP_DMG_BY_LVL[usize::from(champ.lvl.get() - 1)],
        0.,
    )
}

pub const WITS_END: Item = Item {
    id: ItemId::WitsEnd,
    full_name: "Wits_end",
    short_name: "Wits_end",
    cost: 2800.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        //todo: missing tenacity stat (tenacity not implemented)
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 50.,
        base_as: 0.,
        bonus_as: 0.55,
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
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: Some(wits_end_fray),
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Youmuu's ghostblade, haunt passive not implemented
fn youmuus_ghostblade_init(champ: &mut Unit) {
    champ.buffs_values[BuffValueId::YoumuusGhostbladeWraithStepMsPercent] = 0.;
}

fn youmuus_ghostblade_wraith_step_active(champ: &mut Unit, _target_stats: &UnitStats) -> f32 {
    champ.add_temporary_buff(&YOUMUUS_GHOSTBLADE_WRAITH_STEP, champ.stats.item_haste);
    0.
}

fn youmuus_ghostblade_wraith_step_enable(champ: &mut Unit, availability_coef: f32) {
    if champ.buffs_values[BuffValueId::YoumuusGhostbladeWraithStepMsPercent] == 0. {
        let percent_ms_buff: f32 = availability_coef * 0.15; //ms value for ranged champions
        champ.stats.ms_percent += percent_ms_buff;
        champ.buffs_values[BuffValueId::YoumuusGhostbladeWraithStepMsPercent] = percent_ms_buff;
    }
}

fn youmuus_ghostblade_wraith_step_disable(champ: &mut Unit) {
    champ.stats.ms_percent -= champ.buffs_values[BuffValueId::YoumuusGhostbladeWraithStepMsPercent];
    champ.buffs_values[BuffValueId::YoumuusGhostbladeWraithStepMsPercent] = 0.;
}

const YOUMUUS_GHOSTBLADE_WRAITH_STEP: TemporaryBuff = TemporaryBuff {
    id: BuffId::YoumuusGhostbladeWraithStep,
    add_stack: youmuus_ghostblade_wraith_step_enable,
    remove_every_stack: youmuus_ghostblade_wraith_step_disable,
    duration: 6.,
    cooldown: 45.,
};

pub const YOUMUUS_GHOSTBLADE: Item = Item {
    id: ItemId::YoumuusGhostblade,
    full_name: "Youmuus_ghostblade",
    short_name: "Youmuus",
    cost: 2700.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 60.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: 0.,
        ms_flat: 0.,
        ms_percent: 0.,
        lethality: 18.,
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
    init_item: Some(youmuus_ghostblade_init),
    active: Some(youmuus_ghostblade_wraith_step_active),
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Yun Tal Wildarrows
fn yun_tal_serrated_edge(champ: &mut Unit, _target_stats: &UnitStats) -> (f32, f32, f32) {
    (
        champ.stats.crit_chance * (2. / 0.5) * 0.0875 * champ.stats.ad(),
        0.,
        0.,
    )
}

pub const YUN_TAL_WILDARROWS: Item = Item {
    id: ItemId::YunTalWildarrows,
    full_name: "Yun_Tal_wildarrows",
    short_name: "Yun_Tal",
    cost: 3200.,
    item_groups: enum_set!(),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 65.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.25,
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
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: Some(yun_tal_serrated_edge),
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Zak'Zak's realmspike

//Zeke's convergence (useless?)

//Zhonya's hourglass
pub const ZHONYAS_HOURGLASS: Item = Item {
    id: ItemId::ZhonyasHourglass,
    full_name: "Zhonyas_hourlgass",
    short_name: "Zhonyas",
    cost: 3250.,
    item_groups: enum_set!(ItemGroups::Stasis),
    utils: enum_set!(ItemUtils::Survivability), //time stop active (item le plus broken du jeu ^^)
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 120.,
        ap_coef: 0.,
        armor: 50.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
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
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//
// --- BOOTS LISTING --- //
//

//Berserkers_greaves
pub const BERSERKERS_GREAVES: Item = Item {
    id: ItemId::BerserkersGreaves,
    full_name: "Berserkers_greaves",
    short_name: "Berserkers",
    cost: 1100.,
    item_groups: enum_set!(ItemGroups::Boots),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.30,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: 0.,
        ms_flat: 45.,
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
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Boots of swiftness
pub const BOOTS_OF_SWIFTNESS: Item = Item {
    id: ItemId::BootsOfSwiftness,
    full_name: "Boots_of_swiftness",
    short_name: "Swiftness",
    cost: 1000.,
    item_groups: enum_set!(ItemGroups::Boots),
    utils: enum_set!(), //25% slow resist, but not big enough utility
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: 0.,
        ms_flat: 60.,
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
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Ionian boots of lucidity
pub const IONIAN_BOOTS_OF_LUCIDITY: Item = Item {
    id: ItemId::IonianBootsOfLucidity,
    full_name: "Ionian_boots_of_lucidity",
    short_name: "Lucidity",
    cost: 1000.,
    item_groups: enum_set!(ItemGroups::Boots),
    utils: enum_set!(), //10 summoner spell haste, but not big enough utility
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 15.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: 0.,
        ms_flat: 45.,
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
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Mercury's treads
pub const MERCURYS_TREADS: Item = Item {
    id: ItemId::MercurysTreads,
    full_name: "Mercurys_treads",
    short_name: "Mercurys",
    cost: 1200.,
    item_groups: enum_set!(ItemGroups::Boots),
    utils: enum_set!(), //30% tenacity, but not big enough utility
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 20.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: 0.,
        ms_flat: 45.,
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
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Plated steelcaps
pub const PLATED_STEELCAPS: Item = Item {
    id: ItemId::PlatedSteelcaps,
    full_name: "Plated_steelcaps",
    short_name: "Steelcaps",
    cost: 1200.,
    item_groups: enum_set!(ItemGroups::Boots),
    utils: enum_set!(), //-10% from basic attacks, but not big enough utility
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 25.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
        ability_haste: 0.,
        basic_haste: 0.,
        ultimate_haste: 0.,
        item_haste: 0.,
        crit_chance: 0.,
        crit_dmg: 0.,
        ms_flat: 45.,
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
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};

//Sorcerer's shoes
pub const SORCERERS_SHOES: Item = Item {
    id: ItemId::SorcerersShoes,
    full_name: "Sorcerers_shoes",
    short_name: "Sorcerers",
    cost: 1100.,
    item_groups: enum_set!(ItemGroups::Boots),
    utils: enum_set!(),
    stats: UnitStats {
        hp: 0.,
        mana: 0.,
        base_ad: 0.,
        bonus_ad: 0.,
        ap_flat: 0.,
        ap_coef: 0.,
        armor: 0.,
        mr: 0.,
        base_as: 0.,
        bonus_as: 0.,
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
        magic_pen_flat: 15.,
        magic_pen_percent: 0.,
        armor_red_flat: 0.,
        armor_red_percent: 0.,
        mr_red_flat: 0.,
        mr_red_percent: 0.,
        life_steal: 0.,
        omnivamp: 0.,
    },
    init_item: None,
    active: None,
    on_basic_spell_cast: None,
    on_ultimate_cast: None,
    spell_coef: None,
    on_basic_spell_hit: None,
    on_ultimate_spell_hit: None,
    on_basic_attack_hit_static: None,
    on_basic_attack_hit_dynamic: None,
    on_any_hit: None,
    on_ad_hit: None,
    ap_true_dmg_coef: None,
    tot_dmg_coef: None,
};
