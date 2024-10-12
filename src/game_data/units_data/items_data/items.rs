use super::{Item, ItemGroups, ItemId, ItemUtils};

use crate::game_data::*;

use effects_data::{EffectId, EffectStackId, EffectValueId, TemporaryEffect};
use units_data::*;

use enumset::{enum_set, EnumSet};

// This is the file containing every items stats + their passive/active effects scripts

//items parameters:
//for effects that depends on the target hp, we calculate an adapted value based on the assumption that pdf for ennemy hp % is f(x)=x on ]0, 1]
///x*x, where x is the % of hp under which it crits.
const SHADOWFLAME_CINDERBLOOM_COEF: f32 = 0.40 * 0.40;
///x*x, where x is the % of hp under which the passive activates.
const STORMSURGE_STORMRAIDER_COEF: f32 = 0.75 * 0.75;
/// Percentage of target missing hp to account for the average dmg calculation.
const KRAKEN_SLAYER_BRING_IT_DOWN_AVG_TARGET_MISSING_HP_PERCENT: f32 = 0.33;
/// % of missing HP used to calculate the heal from Sundered sky lightshield strike
const SUNDERED_SKY_LIGHTSHIELD_STRIKE_MISSING_HP: f32 = 0.33;
/// Average hp % considered for mists edge dmg.
const BLADE_OF_THE_RUINED_KING_MISTS_EDGE_AVG_TARGET_HP_PERCENT: f32 = 0.67;
/// Percentage of dmg that is done in the passive range and profit from mr reduction.
const ABYSSAL_MASK_UNMAKE_PERCENT_OF_DMG_IN_RANGE: f32 = 0.70;
/// Percentage of abilities that benefits from the increased dmg modifier.
const HORIZON_FOCUS_HYPERSHOT_ABILITIES_TRIGGER_PERCENT: f32 = 0.60;
/// Actual duration of Malignance hatefog curse on the ennemy
const MALIGNANCE_HATEFOG_CURSE_TIME: f32 = 0.8;
/// Number of bolts fired by Runaan's hurricane wind's fury on average (adding to the primary basic attack).
pub(crate) const RUNAANS_HURRICANE_WINDS_FURY_AVG_BOLTS: f32 = 0.25;
/// Number of targets hit by titanic hydra cleave aoe on average
const TITANIC_HYDRA_CLEAVE_AVG_TARGETS: f32 = 0.25;
/// Number of targets hit by profane hydra cleave aoe on average
const PROFANE_HYDRA_CLEAVE_AVG_TARGETS: f32 =
    basic_attack_aoe_effect_avg_additionnal_targets!(PROFANE_HYDRA_CLEAVE_RADIUS);
/// Number of targets hit by ravenous hydra cleave aoe on average
const RAVENOUS_HYDRA_CLEAVE_AVG_TARGETS: f32 =
    basic_attack_aoe_effect_avg_additionnal_targets!(RAVENOUS_HYDRA_CLEAVE_RADIUS);
/// Number of targets hit by stridebreaker cleave aoe on average
const STRIDEBREAKER_CLEAVE_AVG_TARGETS: f32 =
    basic_attack_aoe_effect_avg_additionnal_targets!(STRIDEBREAKER_CLEAVE_RADIUS);
/// % of mana considered during the activation of the shield (1. = 100%)
const SERAPHS_EMBRACE_LIFELINE_MANA_PERCENT: f32 = 0.5;

//spellblade (generic functions for spellblade items)
//some lich bane spellblade functions are separate (because it modifies attack speed)
const SPELLBLADE_COOLDOWN: f32 = 1.5;
const SPELLBLADE_DELAY: f32 = 10.; //effect duration
fn spellblade_init(champ: &mut Unit) {
    champ.effects_stacks[EffectStackId::SpellbladeEmpowered] = 0;
    champ.effects_values[EffectValueId::SpellbladeLastEmpowerTime] = -(SPELLBLADE_DELAY + F32_TOL); //to allow for effect at time = 0.
    champ.effects_values[EffectValueId::SpellbladeLastConsumeTime] =
        -(SPELLBLADE_COOLDOWN + F32_TOL);
    //to allow for effect at time = 0.
}

fn spellblade_on_spell_cast(champ: &mut Unit) {
    //empower next basic attack only if not on cooldown
    if champ.time - champ.effects_values[EffectValueId::SpellbladeLastConsumeTime]
        > SPELLBLADE_COOLDOWN * haste_formula(champ.stats.item_haste)
    {
        champ.effects_stacks[EffectStackId::SpellbladeEmpowered] = 1;
        champ.effects_values[EffectValueId::SpellbladeLastEmpowerTime] = champ.time;
    }
}

/*
fn template_item_spellblade_on_basic_attack_hit(champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    //do nothing if not empowered
    if champ.effects_stacks[EffectStackId::SpellbladeEmpowered] != 1 {
        return (0., 0., 0.);
    }
    //if empowered (previous condition) but last ability cast from too long ago, reset spellblade
    if champ.time - champ.effects_values[EffectValueId::SpellbladeLastEmpowerTime]
        >= SPELLBLADE_DELAY
    {
        champ.effects_stacks[EffectStackId::SpellbladeEmpowered] = 0;
        return (0., 0., 0.);
    }
    //if empowered and last ability cast is recent enough (previous condition), reset and trigger spellblade
    champ.effects_stacks[EffectStackId::SpellbladeEmpowered] = 0;
    champ.effects_values[EffectValueId::SpellbladeLastConsumeTime] = champ.time;
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
    champ.effects_values[EffectValueId::TemplateItemEffectStat] = 0.;
    champ.add_temporary_effect(&TEMPLATE_EFFECT, champ.stats.item_haste);
}

fn template_effect_enable(champ: &mut Unit, availability_coef: f32) {
    if champ.effects_values[EffectValueId::TemplateItemEffectStat] == 0. {
        let some_stat_effect: f32 = availability_coef * some_value;
        champ.stats.some_stat += some_stat_effect;
        champ.effects_values[EffectValueId::TemplateItemEffectStat] = some_stat_effect;
    }
}

fn template_effect_disable(champ: &mut Unit) {
    champ.stats.some_stat -= champ.effects_values[EffectValueId::TemplateItemEffectStat];
    champ.effects_values[EffectValueId::TemplateItemEffectStat] = 0.;
}

const TEMPLATE_EFFECT: TemporaryEffect = TemporaryEffect {
    id: EffectId::TemplateItemEffect,
    add_stack: template_effect_enable,
    remove_every_stack: template_effect_disable,
    duration: some_duration,
    cooldown: some_cooldown,
};

impl Item {
    pub const TEMPLATE_ITEM: Item = Item {
        id: ItemId::,
        full_name: "Template_item",
        short_name: "Template_item",
        cost: 0.,
        item_groups: enum_set!(),
        utils: enum_set!(),
        stats: UnitStats {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}
*/

/*
//template energized item
fn template_energized_item_init(champ: &mut Unit) {
    champ.effects_values[EffectValueId::TemplateEnergizedItemEnergizedPassiveLastTriggerDistance] =
        -(ENERGIZED_ATTACKS_TRAVEL_REQUIRED + F32_TOL); //to allow for effect at time == 0
}

fn template_energized_item_energized_passive(champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    //if not enough energy, add basic attack energy stacks
    if champ.sim_results.units_travelled
        - champ.effects_values[EffectValueId::TemplateEnergizedItemEnergizedPassiveLastTriggerDistance]
        < ENERGIZED_ATTACKS_TRAVEL_REQUIRED
    {
        champ.effects_values[EffectValueId::TemplateEnergizedItemEnergizedPassiveLastTriggerDistance] -=
            ENERGIZED_ATTACKS_TRAVEL_REQUIRED * (ENERGIZED_STACKS_PER_BASIC_ATTACK / 100.);
        return (0., 0., 0.);
    }
    //if enough energy (previous condition), trigger energized attack
    champ.effects_values[EffectValueId::TemplateEnergizedItemEnergizedPassiveLastTriggerDistance] =
        champ.sim_results.units_travelled;
    (0, 0, 0)
}
*/

//Null item
/// For performance reason, we use a `NULL_ITEM` constant to represent empty items slots instead of an Option.
///
/// This is to avoid checking an Option everytime when working with items, since the majority of items aren't null.
impl Item {
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
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//todo: support items?

//Abyssal mask
fn abyssal_mask_init(champ: &mut Unit) {
    //unmake passive
    increase_multiplicatively_scaling_stat(
        &mut champ.stats.mr_red_percent,
        ABYSSAL_MASK_UNMAKE_PERCENT_OF_DMG_IN_RANGE * 0.30,
    );
}

impl Item {
    pub const ABYSSAL_MASK: Item = Item {
        id: ItemId::AbyssalMask,
        full_name: "Abyssal_mask",
        short_name: "Abyssal_mask",
        cost: 2650.,
        item_groups: enum_set!(ItemGroups::Blight),
        utils: enum_set!(),
        stats: UnitStats {
            hp: 300.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 0.,
            mr: 45.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(abyssal_mask_init),
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
    };
}

//Archangel staff not implemented (Seraph's embrace takes its place)

//Ardent censer (useless?)

//Axiom arc
impl Item {
    pub const AXIOM_ARC: Item = Item {
        id: ItemId::AxiomArc,
        full_name: "Axiom_arc",
        short_name: "Axiom_arc",
        cost: 3000.,
        item_groups: enum_set!(),
        utils: enum_set!(ItemUtils::Special), //flux ultimate cd reduction passive
        stats: UnitStats {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 55.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Banshee's veil
impl Item {
    pub const BANSHEES_VEIL: Item = Item {
        id: ItemId::BansheesVeil,
        full_name: "Banshees_veil",
        short_name: "Banshees",
        cost: 3000.,
        item_groups: enum_set!(ItemGroups::Annul),
        utils: enum_set!(ItemUtils::Survivability), //annul spellshield
        stats: UnitStats {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 0.,
            ap_flat: 105.,
            ap_percent: 0.,
            armor: 0.,
            mr: 40.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Black cleaver
fn black_cleaver_init(champ: &mut Unit) {
    champ.effects_stacks[EffectStackId::BlackCleaverCarveStacks] = 0;
    champ.effects_values[EffectValueId::BlackCleaverCarveArmorRedPercent] = 0.;
    champ.effects_values[EffectValueId::BlackCleaverFervorMsFlat] = 0.;
}

const BLACK_CLEAVER_CARVE_ARMOR_RED_PERCENT_PER_STACK: f32 = 0.06;
fn black_cleaver_carve_add_stack(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_stacks[EffectStackId::BlackCleaverCarveStacks] < 5 {
        champ.effects_stacks[EffectStackId::BlackCleaverCarveStacks] += 1;

        decrease_multiplicatively_scaling_stat(
            &mut champ.stats.armor_red_percent,
            champ.effects_values[EffectValueId::BlackCleaverCarveArmorRedPercent],
        ); //decrease amount temporarly
        champ.effects_values[EffectValueId::BlackCleaverCarveArmorRedPercent] +=
            BLACK_CLEAVER_CARVE_ARMOR_RED_PERCENT_PER_STACK;
        increase_multiplicatively_scaling_stat(
            &mut champ.stats.armor_red_percent,
            champ.effects_values[EffectValueId::BlackCleaverCarveArmorRedPercent],
        );
    }
}

fn black_cleaver_carve_remove_every_stack(champ: &mut Unit) {
    decrease_multiplicatively_scaling_stat(
        &mut champ.stats.armor_red_percent,
        champ.effects_values[EffectValueId::BlackCleaverCarveArmorRedPercent],
    );
    champ.effects_values[EffectValueId::BlackCleaverCarveArmorRedPercent] = 0.;
    champ.effects_stacks[EffectStackId::BlackCleaverCarveStacks] = 0;
}

const BLACK_CLEAVER_CARVE: TemporaryEffect = TemporaryEffect {
    id: EffectId::BlackCleaverCarve,
    add_stack: black_cleaver_carve_add_stack,
    remove_every_stack: black_cleaver_carve_remove_every_stack,
    duration: 6.,
    cooldown: 0.,
};

fn black_cleaver_fervor_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_values[EffectValueId::BlackCleaverFervorMsFlat] == 0. {
        let flat_ms_buff: f32 = 20.;
        champ.stats.ms_flat += flat_ms_buff;
        champ.effects_values[EffectValueId::BlackCleaverFervorMsFlat] = flat_ms_buff;
    }
}

fn black_cleaver_fervor_disable(champ: &mut Unit) {
    champ.stats.ms_flat -= champ.effects_values[EffectValueId::BlackCleaverFervorMsFlat];
    champ.effects_values[EffectValueId::BlackCleaverFervorMsFlat] = 0.;
}

const BLACK_CLEAVER_FERVOR: TemporaryEffect = TemporaryEffect {
    id: EffectId::BlackCleaverFervor,
    add_stack: black_cleaver_fervor_enable,
    remove_every_stack: black_cleaver_fervor_disable,
    duration: 2.,
    cooldown: 0.,
};

fn black_cleaver_on_phys_hit(champ: &mut Unit) {
    champ.add_temporary_effect(&BLACK_CLEAVER_CARVE, champ.stats.item_haste);
    champ.add_temporary_effect(&BLACK_CLEAVER_FERVOR, champ.stats.item_haste);
}

impl Item {
    pub const BLACK_CLEAVER: Item = Item {
        id: ItemId::BlackCleaver,
        full_name: "Black_cleaver",
        short_name: "Black_cleaver",
        cost: 3000.,
        item_groups: enum_set!(ItemGroups::Fatality),
        utils: enum_set!(ItemUtils::Special), //carve armor reduction passive
        stats: UnitStats {
            hp: 400.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 40.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },

        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(black_cleaver_init),
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: None,
            on_phys_hit: Some(black_cleaver_on_phys_hit),
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//Blackfire torch
const BLACKFIRE_TORCH_BALEFUL_BLAZE_DOT_DURATION: f32 = 3.;
fn blackfire_torch_init(champ: &mut Unit) {
    champ.effects_values[EffectValueId::BlackfireTorchBalefulBlazeLastApplicationTime] =
        -(BLACKFIRE_TORCH_BALEFUL_BLAZE_DOT_DURATION + F32_TOL); //to allow for effect at time == 0
}

fn blackfire_torch_baleful_blaze(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    n_targets: f32,
) -> PartDmg {
    let dot_time: f32 = f32::min(
        BLACKFIRE_TORCH_BALEFUL_BLAZE_DOT_DURATION,
        champ.time
            - champ.effects_values[EffectValueId::BlackfireTorchBalefulBlazeLastApplicationTime],
    ); //account for DoT overlap with the previous ability hit
    champ.effects_values[EffectValueId::BlackfireTorchBalefulBlazeLastApplicationTime] = champ.time;
    PartDmg(
        0.,
        n_targets * dot_time * (10. + 0.01 * champ.stats.ap()) * (1. / 0.5),
        0.,
    )
}

impl Item {
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
            ap_flat: 80.,
            ap_percent: 0.04, //assumes 1 ennemy is affected by passive
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(blackfire_torch_init),
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: Some(blackfire_torch_baleful_blaze),
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: None,
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//Blade of the ruined king
fn blade_of_the_ruined_king_mists_edge(
    _champion: &mut Unit,
    target_stats: &UnitStats,
    n_targets: f32,
    from_other_effect: bool,
) -> PartDmg {
    if from_other_effect {
        return PartDmg(0., 0., 0.);
    }
    PartDmg(
        n_targets
            * (BLADE_OF_THE_RUINED_KING_MISTS_EDGE_AVG_TARGET_HP_PERCENT * 0.06 * target_stats.hp),
        0.,
        0.,
    ) //value for ranged champions
}

impl Item {
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
            bonus_ad: 40.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: None,
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: Some(blade_of_the_ruined_king_mists_edge),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//Bloodthirster
const BLOODTHIRSTER_ICHORSHIELD_MAX_SHIELD_BY_LVL: [f32; MAX_UNIT_LVL] = [
    165., //lvl 1
    165., //lvl 2
    165., //lvl 3
    165., //lvl 4
    165., //lvl 5
    165., //lvl 6
    165., //lvl 7
    165., //lvl 8
    180., //lvl 9
    195., //lvl 10
    210., //lvl 11
    225., //lvl 12
    240., //lvl 13
    255., //lvl 14
    270., //lvl 15
    285., //lvl 16
    300., //lvl 17
    315., //lvl 18
];
fn bloodthirster_init(champ: &mut Unit) {
    //ichorshield passive
    champ.single_use_heals_shields +=
        BLOODTHIRSTER_ICHORSHIELD_MAX_SHIELD_BY_LVL[usize::from(champ.lvl.get() - 1)];
}

impl Item {
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
            ap_percent: 0.,
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
            life_steal: 0.15,
            omnivamp: 0.,
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(bloodthirster_init),
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
    };
}

//Chempunk chainsword
impl Item {
    pub const CHEMPUNK_CHAINSWORD: Item = Item {
        id: ItemId::ChempunkChainsword,
        full_name: "Chempunk_chainsword",
        short_name: "Chempunk_chainsword",
        cost: 3100.,
        item_groups: enum_set!(),
        utils: enum_set!(ItemUtils::AntiHealShield),
        stats: UnitStats {
            hp: 450.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 45.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Cosmic drive
fn cosmic_drive_init(champ: &mut Unit) {
    champ.effects_values[EffectValueId::CosmicDriveSpellDanceMsFlat] = 0.;
}

fn cosmic_drive_spelldance_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_values[EffectValueId::CosmicDriveSpellDanceMsFlat] == 0. {
        let flat_ms_buff: f32 = 20.;
        champ.stats.ms_flat += flat_ms_buff;
        champ.effects_values[EffectValueId::CosmicDriveSpellDanceMsFlat] = flat_ms_buff;
    }
}

fn cosmic_drive_spelldance_disable(champ: &mut Unit) {
    champ.stats.ms_flat -= champ.effects_values[EffectValueId::CosmicDriveSpellDanceMsFlat];
    champ.effects_values[EffectValueId::CosmicDriveSpellDanceMsFlat] = 0.;
}

const COSMIC_DRIVE_SPELLDANCE: TemporaryEffect = TemporaryEffect {
    id: EffectId::CosmicDriveSpellDance,
    add_stack: cosmic_drive_spelldance_enable,
    remove_every_stack: cosmic_drive_spelldance_disable,
    duration: 4.,
    cooldown: 0.,
};

fn cosmic_drive_spelldance_on_magic_or_true_dmg_hit(champ: &mut Unit) {
    champ.add_temporary_effect(&COSMIC_DRIVE_SPELLDANCE, champ.stats.item_haste);
}

impl Item {
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
            ap_flat: 70.,
            ap_percent: 0.,
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
            ms_percent: 0.04,
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
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(cosmic_drive_init),
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: None,
            on_phys_hit: None,
            on_magic_hit: Some(cosmic_drive_spelldance_on_magic_or_true_dmg_hit),
            on_true_dmg_hit: Some(cosmic_drive_spelldance_on_magic_or_true_dmg_hit),
            on_any_hit: None,
        },
    };
}

//Cryptbloom
impl Item {
    pub const CRYPTBLOOM: Item = Item {
        id: ItemId::Cryptbloom,
        full_name: "Cryptbloom",
        short_name: "Cryptbloom",
        cost: 2850.,
        item_groups: enum_set!(ItemGroups::Blight),
        utils: enum_set!(ItemUtils::Special), //life from death healing passive
        stats: UnitStats {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 0.,
            ap_flat: 60.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Dawncore (useless?) (need to add mana regen & heals_shields power (with Unit::heal_shield_on_ally fn?))

//Dead man's plate
const DEAD_MANS_PLATE_SHIPWRECKER_STACKS_PER_SEC: f32 = 7. / 0.25;
fn dead_mans_plate_init(champ: &mut Unit) {
    let ms: f32 = capped_ms(
        (champ.lvl_stats.ms_flat + champ.items_stats.ms_flat)
            * (1. + (champ.lvl_stats.ms_percent + champ.items_stats.ms_percent)),
    ); //can't use champ.ms() as it uses champ.stats that can be modified by other items init functions
    champ.effects_values[EffectValueId::DeadMansPlateShipwreckerLastHitdistance] =
        -ms * 100. / DEAD_MANS_PLATE_SHIPWRECKER_STACKS_PER_SEC; //to allow for effect at time == 0
}

fn dead_mans_plate_shipwrecker(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    _n_targets: f32,
    from_other_effect: bool,
) -> PartDmg {
    if from_other_effect {
        return PartDmg(0., 0., 0.);
    }

    //not affected by n_targets and from_other_effect
    //current implementation doesn't handle dashes correctly, so maybe disable item on concerned champs
    let time_moving: f32 = (champ.units_travelled
        - champ.effects_values[EffectValueId::DeadMansPlateShipwreckerLastHitdistance])
        / champ.stats.ms();

    let stacks: f32 = f32::min(
        100.,
        DEAD_MANS_PLATE_SHIPWRECKER_STACKS_PER_SEC * time_moving,
    ); //bound stacks to 100.

    champ.effects_values[EffectValueId::DeadMansPlateShipwreckerLastHitdistance] =
        champ.units_travelled;
    PartDmg((stacks / 100.) * (40. + champ.stats.base_ad), 0., 0.)
}

impl Item {
    pub const DEAD_MANS_PLATE: Item = Item {
        id: ItemId::DeadMansPlate,
        full_name: "Dead_mans_plate",
        short_name: "Dead_mans",
        cost: 2900.,
        item_groups: enum_set!(ItemGroups::Momentum),
        utils: enum_set!(),
        stats: UnitStats {
            hp: 350.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 55.,
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
            ms_percent: 0.04,
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
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(dead_mans_plate_init),
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: Some(dead_mans_plate_shipwrecker),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//Death's dance
impl Item {
    pub const DEATHS_DANCE: Item = Item {
        id: ItemId::DeathsDance,
        full_name: "Deaths_dance",
        short_name: "Deaths_dance",
        cost: 3300.,
        item_groups: enum_set!(),
        utils: enum_set!(), //ignore pain passive not big enough utility
        stats: UnitStats {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 60.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Echoes of helia (useless?) (need to add Unit::heal_shield_on_ally fct & on_shield attributes to items)

//Eclipse
const ECLIPSE_EVER_RISING_MOON_COOLDOWN: f32 = 6.;
const ECLIPSE_EVER_RISING_MOON_DELAY: f32 = 2.; //stacks duration
const ECLIPSE_EVER_RISING_MOON_MAX_STACKS: u8 = 2;

fn eclipse_init(champ: &mut Unit) {
    champ.effects_stacks[EffectStackId::EclipseEverRisingMoonStacks] = 0;
    champ.effects_values[EffectValueId::EclipseEverRisingMoonLastStackTime] =
        -(ECLIPSE_EVER_RISING_MOON_DELAY + F32_TOL); //to allow for effect at time = 0.
    champ.effects_values[EffectValueId::EclipseEverRisingMoonLastTriggerTime] =
        -(ECLIPSE_EVER_RISING_MOON_COOLDOWN + F32_TOL); //to allow for effect at time = 0.
}

fn eclipse_ever_rising_moon(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    //do nothing if on cooldown
    if champ.time - champ.effects_values[EffectValueId::EclipseEverRisingMoonLastTriggerTime]
        <= ECLIPSE_EVER_RISING_MOON_COOLDOWN * haste_formula(champ.stats.item_haste)
    {
        return PartDmg(0., 0., 0.);
    }
    //if last hit from too long ago, reset stacks and add 1
    if champ.time - champ.effects_values[EffectValueId::EclipseEverRisingMoonLastStackTime]
        >= ECLIPSE_EVER_RISING_MOON_DELAY
    {
        champ.effects_stacks[EffectStackId::EclipseEverRisingMoonStacks] = 1;
        champ.effects_values[EffectValueId::EclipseEverRisingMoonLastStackTime] = champ.time;
        return PartDmg(0., 0., 0.);
    }
    //if last hit is recent enough (previous condition) but not fully stacked, add 1 stack (useless since max 2 stacks)
    if champ.effects_stacks[EffectStackId::EclipseEverRisingMoonStacks]
        < ECLIPSE_EVER_RISING_MOON_MAX_STACKS - 1
    {
        champ.effects_stacks[EffectStackId::EclipseEverRisingMoonStacks] += 1;
        champ.effects_values[EffectValueId::EclipseEverRisingMoonLastStackTime] = champ.time;
        return PartDmg(0., 0., 0.);
    }
    //if last hit is recent enough and fully stacked (previous condition), reset stacks and trigger ever rising moon
    champ.effects_stacks[EffectStackId::EclipseEverRisingMoonStacks] = 0;
    champ.effects_values[EffectValueId::EclipseEverRisingMoonLastTriggerTime] = champ.time;
    champ.periodic_heals_shields += 80. + 0.2 * champ.stats.bonus_ad; //value for ranged champions
    PartDmg(0.04 * target_stats.hp, 0., 0.)
}

impl Item {
    pub const ECLIPSE: Item = Item {
        id: ItemId::Eclipse,
        full_name: "Eclipse",
        short_name: "Eclipse",
        cost: 2900.,
        item_groups: enum_set!(),
        utils: enum_set!(),
        stats: UnitStats {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 60.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(eclipse_init),
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
            on_any_hit: Some(eclipse_ever_rising_moon),
        },
    };
}

//Edge of night
impl Item {
    pub const EDGE_OF_NIGHT: Item = Item {
        id: ItemId::EdgeOfNight,
        full_name: "Edge_of_night",
        short_name: "Edge_of_night",
        cost: 3000.,
        item_groups: enum_set!(ItemGroups::Annul),
        utils: enum_set!(ItemUtils::Survivability), //annul spellshield
        stats: UnitStats {
            hp: 250.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 50.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Essence reaver
impl Item {
    pub const ESSENCE_REAVER: Item = Item {
        id: ItemId::EssenceReaver,
        full_name: "Essence_reaver",
        short_name: "ER",
        cost: 3150.,
        item_groups: enum_set!(),
        utils: enum_set!(ItemUtils::Special), //essence drain passive mana refund
        stats: UnitStats {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 65.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 0.,
            mr: 0.,
            base_as: 0.,
            bonus_as: 0.,
            ability_haste: 20.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Experimental hexplate
fn experimental_hexplate_init(champ: &mut Unit) {
    champ.effects_values[EffectValueId::ExperimentalHexplateOverdriveBonusAS] = 0.;
    champ.effects_values[EffectValueId::ExperimentalHexplateOverdriveMsPercent] = 0.;
}

fn experimental_hexplate_enable(champ: &mut Unit, availability_coef: f32) {
    if champ.effects_values[EffectValueId::ExperimentalHexplateOverdriveBonusAS] == 0. {
        let bonus_as_buff: f32 = 0.30 * availability_coef;
        let percent_ms_buff: f32 = 0.15 * availability_coef;
        champ.stats.bonus_as += bonus_as_buff;
        champ.stats.ms_percent += percent_ms_buff;
        champ.effects_values[EffectValueId::ExperimentalHexplateOverdriveBonusAS] = bonus_as_buff;
        champ.effects_values[EffectValueId::ExperimentalHexplateOverdriveMsPercent] =
            percent_ms_buff;
    }
}

fn experimental_hexplate_disable(champ: &mut Unit) {
    champ.stats.bonus_as -=
        champ.effects_values[EffectValueId::ExperimentalHexplateOverdriveBonusAS];
    champ.stats.ms_percent -=
        champ.effects_values[EffectValueId::ExperimentalHexplateOverdriveMsPercent];
    champ.effects_values[EffectValueId::ExperimentalHexplateOverdriveBonusAS] = 0.;
    champ.effects_values[EffectValueId::ExperimentalHexplateOverdriveMsPercent] = 0.;
}

const EXPERIMENTAL_HEXPLATE_OVERDRIVE: TemporaryEffect = TemporaryEffect {
    id: EffectId::ExperimentalHexplateOverdrive,
    add_stack: experimental_hexplate_enable,
    remove_every_stack: experimental_hexplate_disable,
    duration: 8.,
    cooldown: 30.,
};

fn experimental_hexplate_overdrive(champ: &mut Unit) {
    champ.add_temporary_effect(&EXPERIMENTAL_HEXPLATE_OVERDRIVE, champ.stats.item_haste);
}

impl Item {
    pub const EXPERIMENTAL_HEXPLATE: Item = Item {
        id: ItemId::ExperimentalHexplate,
        full_name: "Experimental_hexplate",
        short_name: "Hexplate",
        cost: 3000.,
        item_groups: enum_set!(),
        utils: enum_set!(),
        stats: UnitStats {
            hp: 450.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 40.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 0.,
            mr: 0.,
            base_as: 0.,
            bonus_as: 0.20,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(experimental_hexplate_init),
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: Some(experimental_hexplate_overdrive),
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: None,
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//Fimbulwinter (useless?)

//Force of nature (useless?)

//Frozen heart
impl Item {
    pub const FROZEN_HEART: Item = Item {
        id: ItemId::FrozenHeart,
        full_name: "Frozen_heart",
        short_name: "Frozen_heart",
        cost: 2500.,
        item_groups: enum_set!(),
        utils: enum_set!(), //winter's caress passive not big enough
        stats: UnitStats {
            hp: 0.,
            mana: 400.,
            base_ad: 0.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 75.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Guardian angel
impl Item {
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
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Guinsoo's rageblade
fn guinsoos_rageblade_init(champ: &mut Unit) {
    champ.effects_stacks[EffectStackId::GuinsoosRagebladeSeethingStrikeStacks] = 0;
    champ.effects_values[EffectValueId::GuinsoosRagebladeSeethingStrikeBonusAS] = 0.;
    champ.effects_stacks[EffectStackId::GuinsoosRagebladePhantomStacks] = 0;
}

const GUINSOOS_RAGEBLADE_SEETHING_STRIKE_MAX_STACKS: u8 = 4;
fn guinsoos_rageblade_seething_strike_add_stack(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_stacks[EffectStackId::GuinsoosRagebladeSeethingStrikeStacks]
        < GUINSOOS_RAGEBLADE_SEETHING_STRIKE_MAX_STACKS
    {
        champ.effects_stacks[EffectStackId::GuinsoosRagebladeSeethingStrikeStacks] += 1;
        let bonus_as_buff: f32 = 0.08;
        champ.stats.bonus_as += bonus_as_buff;
        champ.effects_values[EffectValueId::GuinsoosRagebladeSeethingStrikeBonusAS] +=
            bonus_as_buff;
    }
}

fn guinsoos_rageblade_seething_strike_remove_every_stack(champ: &mut Unit) {
    champ.stats.bonus_as -=
        champ.effects_values[EffectValueId::GuinsoosRagebladeSeethingStrikeBonusAS];
    champ.effects_values[EffectValueId::GuinsoosRagebladeSeethingStrikeBonusAS] = 0.;
    champ.effects_stacks[EffectStackId::GuinsoosRagebladeSeethingStrikeStacks] = 0;
    champ.effects_stacks[EffectStackId::GuinsoosRagebladePhantomStacks] = 0;
}

const GUINSOOS_RAGEBLADE_SEETHING_STRIKE: TemporaryEffect = TemporaryEffect {
    id: EffectId::GuinsoosRagebladeSeethingStrike,
    add_stack: guinsoos_rageblade_seething_strike_add_stack,
    remove_every_stack: guinsoos_rageblade_seething_strike_remove_every_stack,
    duration: 3.,
    cooldown: 0.,
};

fn guinsoos_rageblade_on_basic_attack_hit(
    champ: &mut Unit,
    target_stats: &UnitStats,
    n_targets: f32,
    from_other_effect: bool,
) -> PartDmg {
    //if from other effect, only wrath passive
    let wrath_ap_dmg: f32 = n_targets * (30.);
    if from_other_effect {
        return PartDmg(0., wrath_ap_dmg, 0.);
    }

    //if not from other effect, wrath + seething strike passives
    //seething strike effect (and stacks) must be applied first, phantom stacks second
    champ.add_temporary_effect(&GUINSOOS_RAGEBLADE_SEETHING_STRIKE, champ.stats.item_haste);

    //if seething strike is not fully stacked, do nothing more
    if champ.effects_stacks[EffectStackId::GuinsoosRagebladeSeethingStrikeStacks]
        < GUINSOOS_RAGEBLADE_SEETHING_STRIKE_MAX_STACKS
    {
        return PartDmg(0., wrath_ap_dmg, 0.);
    }
    //if seething strike is fully stacked (previous condition) but phantom stacks are not fully stacked, add 1 phantom stack
    if champ.effects_stacks[EffectStackId::GuinsoosRagebladePhantomStacks] < 2 {
        champ.effects_stacks[EffectStackId::GuinsoosRagebladePhantomStacks] += 1;
        return PartDmg(0., wrath_ap_dmg, 0.);
    }
    //if seething strike is fully stacked and phantom stacks are fully stacked (previous conditions), reset and return phantom hit dmg
    champ.effects_stacks[EffectStackId::GuinsoosRagebladePhantomStacks] = 0;
    let PartDmg(phantom_hit_ad_dmg, phantom_hit_ap_dmg, phantom_hit_true_dmg) =
        champ.all_on_basic_attack_hit(target_stats, 1., true); //phantom him only applies on 1 target
    PartDmg(
        phantom_hit_ad_dmg,
        phantom_hit_ap_dmg + wrath_ap_dmg,
        phantom_hit_true_dmg,
    )
}

impl Item {
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
            bonus_ad: 30.,
            ap_flat: 30.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(guinsoos_rageblade_init),
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: Some(guinsoos_rageblade_on_basic_attack_hit),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//Heartsteel (useless?)

//Hextech Rocketbelt
fn hextech_rocketbelt_supersonic(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    let availability_coef: f32 =
        effect_availability_formula(40. * haste_formula(champ.stats.item_haste));
    champ.units_travelled += availability_coef * 275.; //maximum dash distance
    let magic_dmg: f32 = availability_coef * (100. + 0.1 * champ.stats.ap());
    champ.dmg_on_target(
        target_stats,
        PartDmg(0., magic_dmg, 0.),
        (1, 1),
        enum_set!(DmgTag::Ability),
        1.,
    )
}

impl Item {
    pub const HEXTECH_ROCKETBELT: Item = Item {
        id: ItemId::HextechRocketbelt,
        full_name: "Hextech_rocketbelt",
        short_name: "Rocketbelt",
        cost: 2600.,
        item_groups: enum_set!(),
        utils: enum_set!(),
        stats: UnitStats {
            hp: 350.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 0.,
            ap_flat: 60.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: None,
            special_active: Some(hextech_rocketbelt_supersonic),
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
    };
}

//Hollow radiance (useless?)

//Horizon focus
impl Item {
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
            ap_flat: 75.,
            ap_percent: 0.,
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
            ability_dmg_modifier: HORIZON_FOCUS_HYPERSHOT_ABILITIES_TRIGGER_PERCENT * 0.10, //hypershot passive is treated as an ability dmg modifier because most basic attacks won't trigger it, less than the real value since we don't know if the ability hits at a far enough distance
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Hubris, ego passive not implemented (too situationnal)
impl Item {
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
            ap_percent: 0.,
            armor: 0.,
            mr: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Hullbreaker, doesn't take into account skipper bonus dmg on structures
const HULLBREAKER_SKIPPER_DELAY: f32 = 10.; //stacks duration
fn hullbreaker_init(champ: &mut Unit) {
    champ.effects_stacks[EffectStackId::HullbreakerSkipperStacks] = 0;
    champ.effects_values[EffectValueId::HullbreakerSkipperLastStackTime] =
        -(HULLBREAKER_SKIPPER_DELAY + F32_TOL); //to allow for effect at time = 0.
}

fn hullbreaker_skipper(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    _n_targets: f32,
    _from_other_effect: bool,
) -> PartDmg {
    //if last hit from too long ago, reset stacks and add 1
    if champ.time - champ.effects_values[EffectValueId::HullbreakerSkipperLastStackTime]
        >= HULLBREAKER_SKIPPER_DELAY
    {
        champ.effects_stacks[EffectStackId::HullbreakerSkipperStacks] = 1;
        champ.effects_values[EffectValueId::HullbreakerSkipperLastStackTime] = champ.time;
        return PartDmg(0., 0., 0.);
    }
    //if last hit is recent enough (previous condition) but not fully stacked, add 1 stack
    if champ.effects_stacks[EffectStackId::HullbreakerSkipperStacks] < 4 {
        champ.effects_stacks[EffectStackId::HullbreakerSkipperStacks] += 1;
        champ.effects_values[EffectValueId::HullbreakerSkipperLastStackTime] = champ.time;
        return PartDmg(0., 0., 0.);
    }
    //if fully stacked, (previous conditions) reset stacks and return skipper dmg
    champ.effects_stacks[EffectStackId::HullbreakerSkipperStacks] = 0;
    PartDmg(0.84 * champ.stats.base_ad + 0.035 * champ.stats.hp, 0., 0.) //value for ranged champions
}

impl Item {
    pub const HULLBREAKER: Item = Item {
        id: ItemId::Hullbreaker,
        full_name: "Hullbreaker",
        short_name: "Hullbreaker",
        cost: 3000.,
        item_groups: enum_set!(),
        utils: enum_set!(), //boarding party passive kinda wathever for ADCs?
        stats: UnitStats {
            hp: 500.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 40.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ms_percent: 0.04,
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
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(hullbreaker_init),
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: Some(hullbreaker_skipper),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//Iceborn gauntlet
//no init function needed since we use generic spellblade variables
fn iceborn_gauntlet_spellblade_on_basic_attack_hit(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    _n_targets: f32,
    from_other_effect: bool,
) -> PartDmg {
    if from_other_effect {
        return PartDmg(0., 0., 0.);
    }

    //do nothing if not empowered
    if champ.effects_stacks[EffectStackId::SpellbladeEmpowered] != 1 {
        return PartDmg(0., 0., 0.);
    }
    //if empowered (previous condition) but last ability cast from too long ago, reset spellblade
    if champ.time - champ.effects_values[EffectValueId::SpellbladeLastEmpowerTime]
        >= SPELLBLADE_DELAY
    {
        champ.effects_stacks[EffectStackId::SpellbladeEmpowered] = 0;
        return PartDmg(0., 0., 0.);
    }
    //if empowered and last ability cast is recent enough (previous condition), reset and trigger spellblade
    champ.effects_stacks[EffectStackId::SpellbladeEmpowered] = 0;
    champ.effects_values[EffectValueId::SpellbladeLastConsumeTime] = champ.time;
    PartDmg(1.5 * champ.stats.base_ad, 0., 0.)
}

impl Item {
    pub const ICEBORN_GAUNTLET: Item = Item {
        id: ItemId::IcebornGauntlet,
        full_name: "Iceborn_gauntlet",
        short_name: "Iceborn_gauntlet",
        cost: 2900.,
        item_groups: enum_set!(ItemGroups::Spellblade),
        utils: enum_set!(),
        stats: UnitStats {
            hp: 300.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(spellblade_init),
            special_active: None,
            on_ability_cast: Some(spellblade_on_spell_cast),
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: Some(iceborn_gauntlet_spellblade_on_basic_attack_hit),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

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
    344., //lvl 9
    368., //lvl 10
    392., //lvl 11
    416., //lvl 12
    440., //lvl 13
    464., //lvl 14
    488., //lvl 15
    512., //lvl 16
    536., //lvl 17
    560., //lvl 18
];
fn immortal_shieldbow_init(champ: &mut Unit) {
    //lifeline passive
    champ.single_use_heals_shields += IMMORTAL_SHIELDBOW_LIFELINE_SHIELD_BY_LVL
        [usize::from(champ.lvl.get() - 1)]
        * effect_availability_formula(
            90. * haste_formula(champ.lvl_stats.item_haste + champ.items_stats.item_haste),
        );
}

impl Item {
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
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(immortal_shieldbow_init),
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
    };
}

//Imperial mandate (useless?)

//Infinity edge
impl Item {
    pub const INFINITY_EDGE: Item = Item {
        id: ItemId::InfinityEdge,
        full_name: "Infinity_edge",
        short_name: "IE",
        cost: 3600.,
        item_groups: enum_set!(),
        utils: enum_set!(),
        stats: UnitStats {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 70.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Jak'sho, voidborn resilience passive not implemented since i find it takes too much time to kick in for a dps
impl Item {
    pub const JAKSHO: Item = Item {
        id: ItemId::Jaksho,
        full_name: "Jaksho",
        short_name: "Jaksho",
        cost: 3200.,
        item_groups: enum_set!(),
        utils: enum_set!(),
        stats: UnitStats {
            hp: 350.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 45.,
            mr: 45.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Kaenic Rookern
fn kaenic_rookern_init(champ: &mut Unit) {
    //magebane passive
    champ.single_use_heals_shields += 0.15 * (champ.lvl_stats.hp + champ.items_stats.hp);
}

impl Item {
    pub const KAENIC_ROOKERN: Item = Item {
        id: ItemId::KaenicRookern,
        full_name: "Kaenic_rookern",
        short_name: "Kaenic_rookern",
        cost: 2900.,
        item_groups: enum_set!(),
        utils: enum_set!(),
        stats: UnitStats {
            //todo: missing base hp regeneration stat
            hp: 400.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(kaenic_rookern_init),
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
    };
}

//Knight's vow (useles?)

//Kraken slayer
const KRAKEN_SLAYER_BRING_IT_DOWN_DELAY: f32 = 3.; //stacks duration
fn kraken_slayer_init(champ: &mut Unit) {
    champ.effects_stacks[EffectStackId::KrakenSlayerBringItDownStacks] = 0;
    champ.effects_values[EffectValueId::KrakenSlayerBringItDownLastStackTime] =
        -(KRAKEN_SLAYER_BRING_IT_DOWN_DELAY + F32_TOL); //to allow for effect at time == 0
}

const KRAKEN_SLAYER_BRING_IT_DOWN_PHYS_DMG_BY_LVL: [f32; MAX_UNIT_LVL] = [
    120., //lvl 1
    120., //lvl 2
    120., //lvl 3
    120., //lvl 4
    120., //lvl 5
    120., //lvl 6
    120., //lvl 7
    120., //lvl 8
    124., //lvl 9
    128., //lvl 10
    132., //lvl 11
    136., //lvl 12
    140., //lvl 13
    144., //lvl 14
    148., //lvl 15
    152., //lvl 16
    156., //lvl 17
    160., //lvl 18
];
fn kraken_slayer_bring_it_down(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    _n_targets: f32,
    _from_other_effect: bool,
) -> PartDmg {
    //if last hit from too long ago, reset stacks and add 1
    if champ.time - champ.effects_values[EffectValueId::KrakenSlayerBringItDownLastStackTime]
        >= KRAKEN_SLAYER_BRING_IT_DOWN_DELAY
    {
        champ.effects_stacks[EffectStackId::KrakenSlayerBringItDownStacks] = 1;
        champ.effects_values[EffectValueId::KrakenSlayerBringItDownLastStackTime] = champ.time;
        return PartDmg(0., 0., 0.);
    }
    //if last hit is recent enough (previous condition) but not fully stacked, add 1 stack
    if champ.effects_stacks[EffectStackId::KrakenSlayerBringItDownStacks] < 2 {
        champ.effects_stacks[EffectStackId::KrakenSlayerBringItDownStacks] += 1;
        champ.effects_values[EffectValueId::KrakenSlayerBringItDownLastStackTime] = champ.time;
        return PartDmg(0., 0., 0.);
    }
    //if fully stacked (previous conditions), reset stacks, return bring it down dmg
    champ.effects_stacks[EffectStackId::KrakenSlayerBringItDownStacks] = 0;
    let phys_dmg: f32 = (1. + 0.5 * KRAKEN_SLAYER_BRING_IT_DOWN_AVG_TARGET_MISSING_HP_PERCENT)
        * KRAKEN_SLAYER_BRING_IT_DOWN_PHYS_DMG_BY_LVL[usize::from(champ.lvl.get() - 1)];
    PartDmg(phys_dmg, 0., 0.) //value for ranged champions
}

impl Item {
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
            bonus_ad: 45.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ms_percent: 0.04,
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
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(kraken_slayer_init),
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: Some(kraken_slayer_bring_it_down),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//Lidandry's torment
const LIANDRYS_TORMENT_TORMENT_DOT_DURATION: f32 = 3.;
const LIANDRYS_TORMENT_SUFFERING_OUTSIDE_COMBAT_TIME_VALUE: f32 = -1.; //special value to indicate that the unit is not in combat, MUST BE NEGATIVE to not interfere with an actual combat start time value
const LIANDRYS_TORMENT_SUFFERING_MAX_TOT_DMG_MODIFIER: f32 = 0.06;
fn liandrys_torment_init(champ: &mut Unit) {
    champ.effects_values[EffectValueId::LiandrysTormentTormentLastApplicationTime] =
        -(LIANDRYS_TORMENT_TORMENT_DOT_DURATION + F32_TOL); //to allow for effect at time == 0
    champ.effects_values[EffectValueId::LiandrysTormentSufferingCombatStartTime] =
        LIANDRYS_TORMENT_SUFFERING_OUTSIDE_COMBAT_TIME_VALUE;
    champ.effects_values[EffectValueId::LiandrysTormentSufferingTotDmgModifier] = 0.;
}

fn liandrys_torment_torment(champ: &mut Unit, target_stats: &UnitStats, n_targets: f32) -> PartDmg {
    let dot_time: f32 = f32::min(
        LIANDRYS_TORMENT_TORMENT_DOT_DURATION,
        champ.time - champ.effects_values[EffectValueId::LiandrysTormentTormentLastApplicationTime],
    ); //account for DoT overlap with the previous ability hit
    champ.effects_values[EffectValueId::LiandrysTormentTormentLastApplicationTime] = champ.time;
    PartDmg(
        0.,
        n_targets * dot_time * (0.01 / 0.5) * target_stats.hp,
        0.,
    )
}

fn liandrys_torment_suffering_refresh(champ: &mut Unit, _availability_coef: f32) {
    //test if it's the first refresh of the effect, reset combat start time if so
    if champ.effects_values[EffectValueId::LiandrysTormentSufferingCombatStartTime]
        == LIANDRYS_TORMENT_SUFFERING_OUTSIDE_COMBAT_TIME_VALUE
    {
        champ.effects_values[EffectValueId::LiandrysTormentSufferingCombatStartTime] = champ.time;
        return;
    }

    //if not the first refresh (previous condition), update tot dmg modifier
    decrease_exponentially_scaling_stat(
        &mut champ.stats.tot_dmg_modifier,
        champ.effects_values[EffectValueId::LiandrysTormentSufferingTotDmgModifier],
    );

    champ.effects_values[EffectValueId::LiandrysTormentSufferingTotDmgModifier] = f32::min(
        LIANDRYS_TORMENT_SUFFERING_MAX_TOT_DMG_MODIFIER,
        0.02 * f32::round(
            champ.time
                - champ.effects_values[EffectValueId::LiandrysTormentSufferingCombatStartTime],
        ),
    ); //as of patch 14.19, using round is the correct way to get the value

    increase_exponentially_scaling_stat(
        &mut champ.stats.tot_dmg_modifier,
        champ.effects_values[EffectValueId::LiandrysTormentSufferingTotDmgModifier],
    );
}

fn liandrys_torment_suffering_disable(champ: &mut Unit) {
    decrease_exponentially_scaling_stat(
        &mut champ.stats.tot_dmg_modifier,
        champ.effects_values[EffectValueId::LiandrysTormentSufferingTotDmgModifier],
    );
    champ.effects_values[EffectValueId::LiandrysTormentSufferingTotDmgModifier] = 0.;
    champ.effects_values[EffectValueId::LiandrysTormentSufferingCombatStartTime] =
        LIANDRYS_TORMENT_SUFFERING_OUTSIDE_COMBAT_TIME_VALUE;
}

const LIANDRYS_TORMENT_SUFFERING: TemporaryEffect = TemporaryEffect {
    id: EffectId::LiandrysTormentSuffering,
    add_stack: liandrys_torment_suffering_refresh,
    remove_every_stack: liandrys_torment_suffering_disable,
    duration: 3.,
    cooldown: 0.,
};

fn liandrys_torment_suffering(champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    champ.add_temporary_effect(&LIANDRYS_TORMENT_SUFFERING, champ.stats.item_haste);
    PartDmg(0., 0., 0.)
}

impl Item {
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
            ap_flat: 70.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(liandrys_torment_init),
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: Some(liandrys_torment_torment),
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: None,
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: Some(liandrys_torment_suffering),
        },
    };
}

//Lich bane
const LICH_BANE_SPELLBLADE_BONUS_AS: f32 = 0.5;
fn lich_bane_spellblade_on_spell_cast(champ: &mut Unit) {
    //empower next basic attack only if not on cooldown
    if champ.time - champ.effects_values[EffectValueId::SpellbladeLastConsumeTime]
        > SPELLBLADE_COOLDOWN * haste_formula(champ.stats.item_haste)
    {
        champ.effects_stacks[EffectStackId::SpellbladeEmpowered] = 1;
        champ.effects_values[EffectValueId::SpellbladeLastEmpowerTime] = champ.time;
        champ.stats.bonus_as += LICH_BANE_SPELLBLADE_BONUS_AS;
    }
}

fn lich_bane_spellblade_on_basic_attack_hit(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    _n_targets: f32,
    from_other_effect: bool,
) -> PartDmg {
    if from_other_effect {
        return PartDmg(0., 0., 0.);
    }

    //do nothing if not empowered
    if champ.effects_stacks[EffectStackId::SpellbladeEmpowered] != 1 {
        return PartDmg(0., 0., 0.);
    }
    //if empowered (previous condition) but last ability cast from too long ago, reset spellblade
    if champ.time - champ.effects_values[EffectValueId::SpellbladeLastEmpowerTime]
        >= SPELLBLADE_DELAY
    {
        champ.effects_stacks[EffectStackId::SpellbladeEmpowered] = 0;
        champ.stats.bonus_as -= LICH_BANE_SPELLBLADE_BONUS_AS;
        return PartDmg(0., 0., 0.);
    }
    //if empowered and last ability cast is recent enough (previous condition), reset and trigger spellblade
    champ.effects_stacks[EffectStackId::SpellbladeEmpowered] = 0;
    champ.stats.bonus_as -= LICH_BANE_SPELLBLADE_BONUS_AS;
    champ.effects_values[EffectValueId::SpellbladeLastConsumeTime] = champ.time;
    PartDmg(0., 0.75 * champ.stats.base_ad + 0.4 * champ.stats.ap(), 0.)
}

impl Item {
    pub const LICH_BANE: Item = Item {
        id: ItemId::LichBane,
        full_name: "Lich_bane",
        short_name: "Lich_bane",
        cost: 3200.,
        item_groups: enum_set!(ItemGroups::Spellblade),
        utils: enum_set!(),
        stats: UnitStats {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 0.,
            ap_flat: 115.,
            ap_percent: 0.,
            armor: 0.,
            mr: 0.,
            base_as: 0.,
            bonus_as: 0.,
            ability_haste: 10.,
            basic_haste: 0.,
            ultimate_haste: 0.,
            item_haste: 0.,
            crit_chance: 0.,
            crit_dmg: 0.,
            ms_flat: 0.,
            ms_percent: 0.04,
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
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(spellblade_init),
            special_active: None,
            on_ability_cast: Some(lich_bane_spellblade_on_spell_cast),
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: Some(lich_bane_spellblade_on_basic_attack_hit),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//Locket of the iron solari (useless?)

//Lord dominik's regards
impl Item {
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
            bonus_ad: 35.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Luden's companion
const LUDENS_COMPANION_FIRE_LOADED_STACKS: f32 = 6.; //f32 because it is directly used in f32 operations
const LUDENS_COMPANION_FIRE_STACKS_CHARGE_TIME: f32 = 12.; //f32 because it is directly used in f32 operations
fn ludens_companion_init(champ: &mut Unit) {
    champ.effects_values[EffectValueId::LudensCompanionFireLastConsumeTime] =
        -(LUDENS_COMPANION_FIRE_STACKS_CHARGE_TIME + F32_TOL);
    //to allow for max stacks at time==0
}

fn ludens_companion_fire(champ: &mut Unit, _target_stats: &UnitStats, n_targets: f32) -> PartDmg {
    //if stacks not loaded, do nothing (previous condition), consume them and return fire dmg
    if champ.time - champ.effects_values[EffectValueId::LudensCompanionFireLastConsumeTime]
        <= LUDENS_COMPANION_FIRE_STACKS_CHARGE_TIME
    {
        return PartDmg(0., 0., 0.);
    }

    //if stacks loaded (previous condition), consume stacks
    champ.effects_values[EffectValueId::LudensCompanionFireLastConsumeTime] = champ.time;
    let dmg: f32 = if n_targets >= LUDENS_COMPANION_FIRE_LOADED_STACKS {
        LUDENS_COMPANION_FIRE_LOADED_STACKS * (60. + 0.04 * champ.stats.ap())
    } else {
        n_targets * (60. + 0.04 * champ.stats.ap())
            + (LUDENS_COMPANION_FIRE_LOADED_STACKS - n_targets) * (30. + 0.02 * champ.stats.ap())
    };
    PartDmg(0., dmg, 0.)
}

impl Item {
    pub const LUDENS_COMPANION: Item = Item {
        id: ItemId::LudensCompanion,
        full_name: "Ludens_companion",
        short_name: "Ludens",
        cost: 2850.,
        item_groups: enum_set!(),
        utils: enum_set!(),
        stats: UnitStats {
            hp: 0.,
            mana: 600.,
            base_ad: 0.,
            bonus_ad: 0.,
            ap_flat: 100.,
            ap_percent: 0.,
            armor: 0.,
            mr: 0.,
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
            mr_red_percent: 0.,
            life_steal: 0.,
            omnivamp: 0.,
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(ludens_companion_init),
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: Some(ludens_companion_fire),
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: None,
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//Malignance
fn malignance_init(champ: &mut Unit) {
    champ.effects_values[EffectValueId::MalignanceHatefogCurseMrRedFlat] = 0.;
}

fn malignance_hatefog_curse_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_values[EffectValueId::MalignanceHatefogCurseMrRedFlat] == 0. {
        let mr_ref_flat: f32 = 10.;
        champ.stats.mr_red_flat += mr_ref_flat;
        champ.effects_values[EffectValueId::MalignanceHatefogCurseMrRedFlat] = mr_ref_flat;
    }
}

fn malignance_hatefog_curse_disable(champ: &mut Unit) {
    champ.stats.mr_red_flat -= champ.effects_values[EffectValueId::MalignanceHatefogCurseMrRedFlat];
    champ.effects_values[EffectValueId::MalignanceHatefogCurseMrRedFlat] = 0.;
}

const MALIGNANCE_HATEFOG_CURSE: TemporaryEffect = TemporaryEffect {
    id: EffectId::MalignanceHatefogCurse,
    add_stack: malignance_hatefog_curse_enable,
    remove_every_stack: malignance_hatefog_curse_disable,
    duration: MALIGNANCE_HATEFOG_CURSE_TIME,
    cooldown: 3.,
};

fn malignance_hatefog(champ: &mut Unit, _target_stats: &UnitStats, n_targets: f32) -> PartDmg {
    //if on cooldown, do nothing
    if !champ.add_temporary_effect(&MALIGNANCE_HATEFOG_CURSE, champ.stats.item_haste) {
        return PartDmg(0., 0., 0.);
    }
    //if not on cooldown (previous condition), return dmg
    PartDmg(
        0.,
        n_targets * (MALIGNANCE_HATEFOG_CURSE_TIME / 0.25) * (15. + 0.0125 * champ.stats.ap()),
        0.,
    )
}

impl Item {
    pub const MALIGNANCE: Item = Item {
        id: ItemId::Malignance,
        full_name: "Malignance",
        short_name: "Malignance",
        cost: 2700.,
        item_groups: enum_set!(),
        utils: enum_set!(ItemUtils::Special), //hatefog magic resistance reduction
        stats: UnitStats {
            hp: 0.,
            mana: 600.,
            base_ad: 0.,
            bonus_ad: 0.,
            ap_flat: 85.,
            ap_percent: 0.,
            armor: 0.,
            mr: 0.,
            base_as: 0.,
            bonus_as: 0.,
            ability_haste: 15.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(malignance_init),
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: Some(malignance_hatefog),
            on_basic_attack_cast: None,
            on_basic_attack_hit: None,
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//Manamune not implemented (Muramana takes its place)

//Maw of malmortius
fn maw_of_malmortius_init(champ: &mut Unit) {
    //lifeline passive (omnivamp not implemented)
    champ.single_use_heals_shields += (150.
        + 1.125 * (champ.lvl_stats.bonus_ad + champ.items_stats.bonus_ad))
        * effect_availability_formula(
            90. * haste_formula(champ.lvl_stats.item_haste + champ.items_stats.item_haste),
        );
    //value for ranged champions
}

impl Item {
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
            bonus_ad: 60.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(maw_of_malmortius_init),
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
    };
}

//Mejai's soulstealer not implemented (too situationnal)

//Mercirial scimitar
impl Item {
    pub const MERCURIAL_SCIMITAR: Item = Item {
        id: ItemId::MercurialScimitar,
        full_name: "Mercurial_scimitar",
        short_name: "Mercurial",
        cost: 3200.,
        item_groups: enum_set!(ItemGroups::Quicksilver),
        utils: enum_set!(ItemUtils::Survivability),
        stats: UnitStats {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 40.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 0.,
            mr: 40.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Mikael's blessing (useless?)

//Moonstone renewer (useless?)

//Morellonomicon
impl Item {
    pub const MORELLONOMICON: Item = Item {
        id: ItemId::Morellonomicon,
        full_name: "Morellonomicon",
        short_name: "Morello",
        cost: 2950.,
        item_groups: enum_set!(),
        utils: enum_set!(ItemUtils::AntiHealShield),
        stats: UnitStats {
            hp: 350.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 0.,
            ap_flat: 75.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Mortal reminder
impl Item {
    pub const MORTAL_REMINDER: Item = Item {
        id: ItemId::MortalReminder,
        full_name: "Mortal_reminder",
        short_name: "Mortal_reminder",
        cost: 3200.,
        item_groups: enum_set!(ItemGroups::Fatality),
        utils: enum_set!(ItemUtils::AntiHealShield),
        stats: UnitStats {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 35.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            armor_pen_percent: 0.30,
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
    };
}

//Muramana
fn muramana_init(champ: &mut Unit) {
    champ.effects_values[EffectValueId::MuramanaShockLastSpellHitTime] = -F32_TOL; //to allow for effect at time == 0

    //awe passive
    champ.stats.bonus_ad += 0.02 * (champ.lvl_stats.mana + champ.items_stats.mana);
}

fn muramana_shock_on_ability_hit(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    n_targets: f32,
) -> PartDmg {
    //set shock last ability hit, to prevent potential on basic attack hit effects triggered by this ability to apply shock twice
    champ.effects_values[EffectValueId::MuramanaShockLastSpellHitTime] = champ.time;
    PartDmg(n_targets * 0.03 * champ.stats.mana, 0., 0.) //value for ranged champions
}

fn muramana_shock_on_basic_attack_hit(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    n_targets: f32,
    _from_other_effect: bool,
) -> PartDmg {
    //if same instance of dmg (==exact same time) as muramana_shock_on_ability_hit, do nothing (to prevent basic attack that trigger on hit to apply muramana passive twice)
    if champ.time == champ.effects_values[EffectValueId::MuramanaShockLastSpellHitTime] {
        return PartDmg(0., 0., 0.);
    }
    //if not the same instance, return dmg (no need to update shock last ability hit time since ability effects are called first)
    PartDmg(n_targets * (0.012 * champ.stats.mana), 0., 0.)
}

impl Item {
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
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(muramana_init),
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: Some(muramana_shock_on_ability_hit),
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: Some(muramana_shock_on_basic_attack_hit),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//Nashor's tooth
fn nashors_tooth_icathian_bite(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    n_targets: f32,
    _from_other_effect: bool,
) -> PartDmg {
    PartDmg(0., n_targets * (15. + 0.15 * champ.stats.ap()), 0.)
}

impl Item {
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
            ap_flat: 80.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: None,
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: Some(nashors_tooth_icathian_bite),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//Navori flickerblade
const NAVORI_FLICKERBLADE_TRANSCENDENCE_CD_REFUND_PERCENT: f32 = 0.15;
fn navori_flickerblade_transcendence(champ: &mut Unit) {
    champ.q_cd *= 1. - NAVORI_FLICKERBLADE_TRANSCENDENCE_CD_REFUND_PERCENT;
    champ.w_cd *= 1. - NAVORI_FLICKERBLADE_TRANSCENDENCE_CD_REFUND_PERCENT;
    champ.e_cd *= 1. - NAVORI_FLICKERBLADE_TRANSCENDENCE_CD_REFUND_PERCENT;
}

impl Item {
    pub const NAVORI_FLICKERBLADE: Item = Item {
        id: ItemId::NavoriFlickerblade,
        full_name: "Navori_flickerblade",
        short_name: "Navori",
        cost: 2650.,
        item_groups: enum_set!(),
        utils: enum_set!(),
        stats: UnitStats {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ms_percent: 0.04,
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
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: None,
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: Some(navori_flickerblade_transcendence),
            on_basic_attack_hit: None,
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//Opportunity, extration passive not implemented (too situationnal)
fn opportunity_init(champ: &mut Unit) {
    champ.effects_values[EffectValueId::OpportunityPreparationLethality] = 0.;

    //preparation passive
    champ.add_temporary_effect(
        &OPPORTUNITY_PREPARATION,
        champ.lvl_stats.item_haste + champ.items_stats.item_haste,
    );
}

fn opportunity_preparation_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_values[EffectValueId::OpportunityPreparationLethality] == 0. {
        let lethality_buff: f32 = 6.; //ranged value
        champ.stats.lethality += lethality_buff;
        champ.effects_values[EffectValueId::OpportunityPreparationLethality] = lethality_buff;
    }
}

fn opportunity_preparation_disable(champ: &mut Unit) {
    champ.stats.lethality -= champ.effects_values[EffectValueId::OpportunityPreparationLethality];
    champ.effects_values[EffectValueId::OpportunityPreparationLethality] = 0.;
}

const OPPORTUNITY_PREPARATION: TemporaryEffect = TemporaryEffect {
    id: EffectId::OpportunityPreparation,
    add_stack: opportunity_preparation_enable,
    remove_every_stack: opportunity_preparation_disable,
    duration: 3.,
    cooldown: 0., //cooldown too small to be relevant (as of patch 14.08)
};

impl Item {
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
            bonus_ad: 50.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ms_percent: 0.04,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(opportunity_init),
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
    };
}

//Overlord's bloodmail
fn overlords_bloodmail_init(champ: &mut Unit) {
    //tyranny passive
    champ.stats.bonus_ad += 0.02 * champ.items_stats.hp;

    //retribution passive not implemented (too situationnal)
}

impl Item {
    pub const OVERLORDS_BLOODMAIL: Item = Item {
        id: ItemId::OverlordsBloodmail,
        full_name: "Overlords_bloodmail",
        short_name: "Overlords_bloodmail",
        cost: 3300.,
        item_groups: enum_set!(),
        utils: enum_set!(),
        stats: UnitStats {
            hp: 550.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 30.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(overlords_bloodmail_init),
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
    };
}

//Phantom dancer
impl Item {
    pub const PHANTOM_DANCER: Item = Item {
        id: ItemId::PhantomDancer,
        full_name: "Phantom_dancer",
        short_name: "PD",
        cost: 2650.,
        item_groups: enum_set!(),
        utils: enum_set!(),
        stats: UnitStats {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Profane hydra
fn _profane_hydra_heretical_cleave(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    //we do not reduce the dmg value because the cd is short enough (10 sec, as of patch 14.06)
    champ.dmg_on_target(
        target_stats,
        PartDmg(0.8 * champ.stats.ad(), 0., 0.),
        (1, 1),
        enum_set!(),
        1.,
    ) //assumes the target is not under 50% hp (worst case scenario)
}

fn profane_hydra_cleave(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    n_targets: f32,
    _from_other_effect: bool,
) -> PartDmg {
    PartDmg(
        n_targets * (PROFANE_HYDRA_CLEAVE_AVG_TARGETS * 0.20 * champ.stats.ad()),
        0.,
        0.,
    ) //value for ranged champions
}

const PROFANE_HYDRA_CLEAVE_RADIUS: f32 = 350.; //used to determine how much targets are hit by cleave
impl Item {
    pub const PROFANE_HYDRA: Item = Item {
        id: ItemId::ProfaneHydra,
        full_name: "Profane_hydra",
        short_name: "Profane_hydra",
        cost: 3200.,
        item_groups: enum_set!(ItemGroups::Hydra),
        utils: enum_set!(),
        stats: UnitStats {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 60.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 0.,
            mr: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: None,
            special_active: None, //Some(profane_hydra_heretical_cleave), //active not used for ranged champions
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: Some(profane_hydra_cleave),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//Rabadon's deathcap
impl Item {
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
            ap_flat: 130.,
            ap_percent: 0.30,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Randuin's omen
impl Item {
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
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Rapid firecannon
fn rapid_firecannon_init(champ: &mut Unit) {
    champ.effects_values[EffectValueId::RapidFirecannonSharpshooterLastTriggerDistance] =
        -(ENERGIZED_ATTACKS_TRAVEL_REQUIRED + F32_TOL); // to allow for effect at time == 0
}

fn rapid_firecannon_sharpshooter(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    _n_targets: f32,
    from_other_effect: bool,
) -> PartDmg {
    if from_other_effect {
        return PartDmg(0., 0., 0.);
    }

    //if not enough energy, add basic attack energy stacks
    if champ.units_travelled
        - champ.effects_values[EffectValueId::RapidFirecannonSharpshooterLastTriggerDistance]
        < ENERGIZED_ATTACKS_TRAVEL_REQUIRED
    {
        champ.effects_values[EffectValueId::RapidFirecannonSharpshooterLastTriggerDistance] -=
            ENERGIZED_ATTACKS_TRAVEL_REQUIRED * (ENERGIZED_STACKS_PER_BASIC_ATTACK / 100.);
        return PartDmg(0., 0., 0.);
    }
    //if enough energy (previous condition), trigger energized attack
    champ.effects_values[EffectValueId::RapidFirecannonSharpshooterLastTriggerDistance] =
        champ.units_travelled;
    PartDmg(0., 40., 0.)
}

impl Item {
    pub const RAPID_FIRECANNON: Item = Item {
        id: ItemId::RapidFirecannon,
        full_name: "Rapid_firecannon",
        short_name: "RFC",
        cost: 2650.,
        item_groups: enum_set!(),
        utils: enum_set!(ItemUtils::Special), //sharpshooter bonus range
        stats: UnitStats {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ms_percent: 0.04,
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
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(rapid_firecannon_init),
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: Some(rapid_firecannon_sharpshooter),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//Ravenous hydra
fn _ravenous_hydra_ravenous_crescent(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    //we do not reduce the dmg value because the cd is short enough (10 sec, as of patch 14.06)
    let dmg: PartDmg = champ.dmg_on_target(
        target_stats,
        PartDmg(0.8 * champ.stats.ad(), 0., 0.),
        (1, 1),
        enum_set!(),
        1.,
    );
    champ.periodic_heals_shields += dmg.as_sum() * champ.stats.life_steal; //life steal applies to crescent
    dmg
}

fn ravenous_hydra_cleave(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    n_targets: f32,
    _from_other_effect: bool,
) -> PartDmg {
    PartDmg(
        n_targets * (RAVENOUS_HYDRA_CLEAVE_AVG_TARGETS * 0.20 * champ.stats.ad()),
        0.,
        0.,
    ) //value for ranged champions
}

const RAVENOUS_HYDRA_CLEAVE_RADIUS: f32 = 350.; //used to determine how much targets are hit by cleave
impl Item {
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
            bonus_ad: 65.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            life_steal: 0.12,
            omnivamp: 0.,
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: None,
            special_active: None, //Some(ravenous_hydra_ravenous_crescent), //active not used for ranged champions
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: Some(ravenous_hydra_cleave),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//Redemption (useless?)

//Riftmaker
const RIFTMAKER_VOID_CORRUPTION_OUTSIDE_COMBAT_TIME_VALUE: f32 = -1.; //special value to indicate that the unit is not in combat, MUST BE NEGATIVE to not interfere with an actual combat start time value
fn riftmaker_init(champ: &mut Unit) {
    champ.effects_values[EffectValueId::RiftmakerVoidCorruptionOmnivamp] = 0.;
    champ.effects_values[EffectValueId::RiftmakerVoidCorruptionCombatStartTime] =
        RIFTMAKER_VOID_CORRUPTION_OUTSIDE_COMBAT_TIME_VALUE;
    champ.effects_values[EffectValueId::RiftmakerVoidCorruptionTotDmgModifier] = 0.;

    //void infusion passive
    champ.stats.ap_flat += 0.02 * champ.items_stats.hp;
}

const RIFTMAKER_VOID_CORRUPTION_MAX_COEF: f32 = 0.08;
const RIFTMAKER_VOID_CORRUPTION_OMNIVAMP: f32 = 0.06; //value for ranged champions
fn riftmaker_void_corruption_refresh(champ: &mut Unit, _availability_coef: f32) {
    //test if it's the first refresh of the effect, reset combat start time if so
    if champ.effects_values[EffectValueId::RiftmakerVoidCorruptionCombatStartTime]
        == RIFTMAKER_VOID_CORRUPTION_OUTSIDE_COMBAT_TIME_VALUE
    {
        champ.effects_values[EffectValueId::RiftmakerVoidCorruptionCombatStartTime] = champ.time;
        return;
    }

    //if not the first refresh (previous condition), update tot dmg modifier
    decrease_exponentially_scaling_stat(
        &mut champ.stats.tot_dmg_modifier,
        champ.effects_values[EffectValueId::RiftmakerVoidCorruptionTotDmgModifier],
    ); //remove current tot dmg multiplier temporarly

    let tot_dmg_modifier: f32 = f32::min(
        RIFTMAKER_VOID_CORRUPTION_MAX_COEF,
        0.02 * f32::trunc(
            champ.time
                - champ.effects_values[EffectValueId::RiftmakerVoidCorruptionCombatStartTime],
        ),
    ); //as of patch 14.19, using trunc is the correct way to get the value
    champ.effects_values[EffectValueId::RiftmakerVoidCorruptionTotDmgModifier] = tot_dmg_modifier;

    increase_exponentially_scaling_stat(&mut champ.stats.tot_dmg_modifier, tot_dmg_modifier);

    //gain omnivamp if fully stacked
    if (champ.effects_values[EffectValueId::RiftmakerVoidCorruptionOmnivamp] == 0.)
        && (tot_dmg_modifier == RIFTMAKER_VOID_CORRUPTION_MAX_COEF)
    {
        champ.stats.omnivamp += RIFTMAKER_VOID_CORRUPTION_OMNIVAMP;
        champ.effects_values[EffectValueId::RiftmakerVoidCorruptionOmnivamp] =
            RIFTMAKER_VOID_CORRUPTION_OMNIVAMP;
    }
}

fn riftmaker_void_corruption_disable(champ: &mut Unit) {
    champ.stats.omnivamp -= champ.effects_values[EffectValueId::RiftmakerVoidCorruptionOmnivamp];
    champ.effects_values[EffectValueId::RiftmakerVoidCorruptionOmnivamp] = 0.;
    decrease_exponentially_scaling_stat(
        &mut champ.stats.tot_dmg_modifier,
        champ.effects_values[EffectValueId::RiftmakerVoidCorruptionTotDmgModifier],
    );
    champ.effects_values[EffectValueId::RiftmakerVoidCorruptionTotDmgModifier] = 0.;
    champ.effects_values[EffectValueId::RiftmakerVoidCorruptionCombatStartTime] =
        RIFTMAKER_VOID_CORRUPTION_OUTSIDE_COMBAT_TIME_VALUE;
}

const RIFTMAKER_VOID_CORRUPTION: TemporaryEffect = TemporaryEffect {
    id: EffectId::RiftmakerVoidCorruption,
    add_stack: riftmaker_void_corruption_refresh,
    remove_every_stack: riftmaker_void_corruption_disable,
    duration: 4.,
    cooldown: 0.,
};

fn riftmaker_void_corruption(champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    champ.add_temporary_effect(&RIFTMAKER_VOID_CORRUPTION, champ.stats.item_haste);
    PartDmg(0., 0., 0.)
}

impl Item {
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
            ap_flat: 70.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(riftmaker_init),
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
            on_any_hit: Some(riftmaker_void_corruption),
        },
    };
}

//Rod of ages
fn rod_of_ages_timeless_init(champ: &mut Unit) {
    //get time elapsed since bought (assuming items are in purchase order)
    let mut take_item: bool = false;
    let mut cost_since_bought: f32 = 0.;
    for &item in champ.build.iter() {
        if take_item {
            cost_since_bought += item.cost;
        }
        if *item == Item::ROD_OF_AGES {
            take_item = true;
        }
    }
    let min_since_bought: f32 = f32::min(10., cost_since_bought / TOT_GOLDS_PER_MIN);

    //add timeless stats based on time elapsed
    champ.stats.hp += min_since_bought * 10.;
    champ.stats.mana += min_since_bought * 20.;
    champ.stats.ap_flat += min_since_bought * 3.;
}

impl Item {
    pub const ROD_OF_AGES: Item = Item {
        id: ItemId::RodOfAges,
        full_name: "Rod_of_ages",
        short_name: "RoA",
        cost: 2600.,
        item_groups: enum_set!(ItemGroups::Eternity),
        utils: enum_set!(),
        stats: UnitStats {
            hp: 400.,
            mana: 400.,
            base_ad: 0.,
            bonus_ad: 0.,
            ap_flat: 50.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(rod_of_ages_timeless_init),
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
    };
}

//Runaan's hurricane
fn runaans_hurricane_winds_fury(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    mut n_targets: f32,
    from_other_effect: bool,
) -> PartDmg {
    if from_other_effect {
        return PartDmg(0., 0., 0.);
    }
    n_targets /= 1. + RUNAANS_HURRICANE_WINDS_FURY_AVG_BOLTS; //get number of targets without runaans bolts
    PartDmg(
        n_targets
            * RUNAANS_HURRICANE_WINDS_FURY_AVG_BOLTS
            * (0.55 * champ.stats.ad() * champ.stats.crit_coef()),
        0.,
        0.,
    )
}

impl Item {
    pub const RUNAANS_HURRICANE: Item = Item {
        id: ItemId::RunaansHurricane,
        full_name: "Runaans_hurricane",
        short_name: "Runaans",
        cost: 2650.,
        item_groups: enum_set!(),
        utils: enum_set!(),
        stats: UnitStats {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ms_percent: 0.04,
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
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: None,
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: Some(runaans_hurricane_winds_fury),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//Rylais crystal scepter
impl Item {
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
            ap_flat: 65.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Seraph's_embrace
fn seraphs_embrace_init(champ: &mut Unit) {
    //awe passive
    champ.stats.ap_flat += 0.02 * champ.items_stats.mana; //only take bonus mana into account

    //lifeline passive
    champ.single_use_heals_shields += (200.
        + SERAPHS_EMBRACE_LIFELINE_MANA_PERCENT
            * 0.2
            * (champ.lvl_stats.mana + champ.items_stats.mana))
        * effect_availability_formula(
            90. * haste_formula(champ.lvl_stats.item_haste + champ.items_stats.item_haste),
        );
    //shield depends on current mana
}

impl Item {
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
            ap_flat: 70.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(seraphs_embrace_init),
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
    };
}

//Serpent's fang
impl Item {
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
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Serylda's grudge
impl Item {
    pub const SERYLDAS_GRUDGE: Item = Item {
        id: ItemId::SeryldasGrudge,
        full_name: "Seryldas_grudge",
        short_name: "Seryldas",
        cost: 3000.,
        item_groups: enum_set!(ItemGroups::Fatality),
        utils: enum_set!(), //bitter cold passive slow not big enough
        stats: UnitStats {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 45.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            armor_pen_percent: 0.30,
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
    };
}

//Shadowflame
fn shadowflame_init(champ: &mut Unit) {
    let modifier: f32 =
        SHADOWFLAME_CINDERBLOOM_COEF * (0.2 * (1. + champ.stats.crit_dmg - Unit::BASE_CRIT_DMG)); //crit dmg above BASE_CRIT_DMG only affects only the bonus dmg of shadowflame not the entire dmg instance
    increase_exponentially_scaling_stat(&mut champ.stats.magic_dmg_modifier, modifier);
    increase_exponentially_scaling_stat(&mut champ.stats.true_dmg_modifier, modifier);
}

impl Item {
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
            ap_flat: 110.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(shadowflame_init),
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
    };
}

//Shurelya's Battlesong (useless?)

//Solstice sleigh (useless?)

//Spear of shojin
fn spear_of_shojin_init(champ: &mut Unit) {
    champ.effects_stacks[EffectStackId::SpearOfShojinFocusedWillStacks] = 0;
    champ.effects_values[EffectValueId::SpearOfShojinFocusedWillAbilityDmgModifier] = 0.;
}

const SPEAR_OF_SHOJIN_FOCUSED_WILL_SPELL_COEF_PER_STACK: f32 = 0.03;
fn spear_of_shojin_focused_will(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    _n_targets: f32,
) -> PartDmg {
    champ.add_temporary_effect(&SPEAR_OF_SHOJIN_FOCUSED_WILL, champ.stats.item_haste);
    PartDmg(0., 0., 0.)
}

fn spear_of_shojin_focused_will_add_stack(champ: &mut Unit, _availability_coef: f32) {
    //if not fully stacked, add 1 stack and update ability dmg modifier
    if champ.effects_stacks[EffectStackId::SpearOfShojinFocusedWillStacks] < 4 {
        decrease_exponentially_scaling_stat(
            &mut champ.stats.ability_dmg_modifier,
            champ.effects_values[EffectValueId::SpearOfShojinFocusedWillAbilityDmgModifier],
        ); //decrease amount temporarly

        champ.effects_stacks[EffectStackId::SpearOfShojinFocusedWillStacks] += 1;
        champ.effects_values[EffectValueId::SpearOfShojinFocusedWillAbilityDmgModifier] +=
            SPEAR_OF_SHOJIN_FOCUSED_WILL_SPELL_COEF_PER_STACK;

        increase_exponentially_scaling_stat(
            &mut champ.stats.ability_dmg_modifier,
            champ.effects_values[EffectValueId::SpearOfShojinFocusedWillAbilityDmgModifier],
        );
    }
}

fn spear_of_shojin_focused_will_disable(champ: &mut Unit) {
    decrease_exponentially_scaling_stat(
        &mut champ.stats.ability_dmg_modifier,
        champ.effects_values[EffectValueId::SpearOfShojinFocusedWillAbilityDmgModifier],
    );
    champ.effects_values[EffectValueId::SpearOfShojinFocusedWillAbilityDmgModifier] = 0.;
    champ.effects_stacks[EffectStackId::SpearOfShojinFocusedWillStacks] = 0;
}

const SPEAR_OF_SHOJIN_FOCUSED_WILL: TemporaryEffect = TemporaryEffect {
    id: EffectId::SpearOfShojinFocusedWill,
    add_stack: spear_of_shojin_focused_will_add_stack,
    remove_every_stack: spear_of_shojin_focused_will_disable,
    duration: 6.,
    cooldown: 0.,
};

impl Item {
    pub const SPEAR_OF_SHOJIN: Item = Item {
        id: ItemId::SpearOfShojin,
        full_name: "Spear_of_shojin",
        short_name: "Shojin",
        cost: 3100.,
        item_groups: enum_set!(),
        utils: enum_set!(),
        stats: UnitStats {
            hp: 450.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 45.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 0.,
            mr: 0.,
            base_as: 0.,
            bonus_as: 0.,
            ability_haste: 0.,
            basic_haste: 25.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(spear_of_shojin_init),
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: Some(spear_of_shojin_focused_will),
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: None,
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//Spirit visage (useless?) (need to add heals_shields power)

//Staff of flowing water (useless?) (need to add Unit::heal_shield_on_ally fct and heals_shields power)

//Statikk shiv (electroshock passive not implemented because too situationnal)
impl Item {
    pub const STATIKK_SHIV: Item = Item {
        id: ItemId::StatikkShiv,
        full_name: "Statikk_shiv",
        short_name: "Statikk",
        cost: 2900.,
        item_groups: enum_set!(),
        utils: enum_set!(ItemUtils::Special), //electrospark wave clear
        stats: UnitStats {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 50.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ms_percent: 0.04,
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
    };
}

//Sterak's gage
fn steraks_gage_init(champ: &mut Unit) {
    //the claw that catch passive
    champ.stats.bonus_ad += 0.45 * (champ.lvl_stats.base_ad + champ.items_stats.base_ad);

    //lifeline passive
    champ.single_use_heals_shields += 0.5
        * 0.6
        * champ.items_stats.hp
        * effect_availability_formula(
            90. * haste_formula(champ.lvl_stats.item_haste + champ.items_stats.item_haste),
        );
    //actual value halved because shield decays, only counts bonus hp
}

impl Item {
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
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(steraks_gage_init),
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
    };
}

//stormsurge
fn stormsurge_init(champ: &mut Unit) {
    champ.effects_values[EffectValueId::StormsurgeStormraiderMsPercent] = 0.;
    champ.effects_stacks[EffectStackId::StormsurgeStormraiderTriggered] = 0;
}

fn stormsurge_stormraider_ms_enable(champ: &mut Unit, availability_coef: f32) {
    if champ.effects_values[EffectValueId::StormsurgeStormraiderMsPercent] == 0. {
        let percent_ms_buff: f32 = STORMSURGE_STORMRAIDER_COEF * availability_coef * 0.25;
        champ.stats.ms_percent += percent_ms_buff;
        champ.effects_values[EffectValueId::StormsurgeStormraiderMsPercent] = percent_ms_buff;
    }
}

fn stormsurge_stormraider_ms_disable(champ: &mut Unit) {
    champ.stats.ms_percent -= champ.effects_values[EffectValueId::StormsurgeStormraiderMsPercent];
    champ.effects_values[EffectValueId::StormsurgeStormraiderMsPercent] = 0.;
}

const STORMSURGE_STORMRAIDER_MS: TemporaryEffect = TemporaryEffect {
    id: EffectId::StormsurgeStormraiderMS,
    add_stack: stormsurge_stormraider_ms_enable,
    remove_every_stack: stormsurge_stormraider_ms_disable,
    duration: 1.5,
    cooldown: STORMSURGE_STORMRAIDER_COOLDOWN,
};

const STORMSURGE_STORMRAIDER_COOLDOWN: f32 = 30.;
fn stormsurge_stormraider(champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    //stormraider passive, triggers once, after a fixed time after the first dmg instance since we don't record dmg done over time and cannot check the real activation condition
    if champ.effects_stacks[EffectStackId::StormsurgeStormraiderTriggered] == 0 && champ.time > 2.0
    {
        champ.effects_stacks[EffectStackId::StormsurgeStormraiderTriggered] = 1;
        champ.add_temporary_effect(&STORMSURGE_STORMRAIDER_MS, champ.stats.item_haste);
        let avalability_coef: f32 = effect_availability_formula(
            STORMSURGE_STORMRAIDER_COOLDOWN * haste_formula(champ.stats.item_haste),
        );
        return PartDmg(
            0.,
            STORMSURGE_STORMRAIDER_COEF * avalability_coef * (150. + 0.15 * champ.stats.ap()),
            0.,
        );
    }
    PartDmg(0., 0., 0.)
}

impl Item {
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
            ap_percent: 0.,
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
            ms_percent: 0.04,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(stormsurge_init),
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
            on_any_hit: Some(stormsurge_stormraider),
        },
    };
}

//Stridebreaker
fn stridebreaker_init(champ: &mut Unit) {
    champ.effects_values[EffectValueId::StridebreakerBreakingShockwaveMsPercent] = 0.;
}

fn stridebreaker_braking_shockwave_ms_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_values[EffectValueId::StridebreakerBreakingShockwaveMsPercent] == 0. {
        let ms_percent_buff: f32 = 0.5 * 0.35; //ms buff halved because decays over time
        champ.stats.ms_percent += ms_percent_buff;
        champ.effects_values[EffectValueId::StridebreakerBreakingShockwaveMsPercent] =
            ms_percent_buff;
    }
}

fn stridebreaker_braking_shockwave_ms_disable(champ: &mut Unit) {
    champ.stats.ms_percent -=
        champ.effects_values[EffectValueId::StridebreakerBreakingShockwaveMsPercent];
    champ.effects_values[EffectValueId::StridebreakerBreakingShockwaveMsPercent] = 0.;
}

const STRIDEBREAKER_BREAKING_SHOCKWAVE_MS: TemporaryEffect = TemporaryEffect {
    id: EffectId::StridebreakerBreakingShockwaveMS,
    add_stack: stridebreaker_braking_shockwave_ms_enable,
    remove_every_stack: stridebreaker_braking_shockwave_ms_disable,
    duration: 3.,
    cooldown: 0.,
};

fn stridebreaker_breaking_shockwave(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    let dmg: PartDmg = champ.dmg_on_target(
        target_stats,
        PartDmg(0.8 * champ.stats.ad(), 0., 0.),
        (1, 1),
        enum_set!(),
        1.,
    ); //calculate dmg before ms boost
    champ.add_temporary_effect(&STRIDEBREAKER_BREAKING_SHOCKWAVE_MS, champ.stats.item_haste);
    dmg
}

fn stridebreaker_cleave(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    n_targets: f32,
    _from_other_effect: bool,
) -> PartDmg {
    PartDmg(
        n_targets * (STRIDEBREAKER_CLEAVE_AVG_TARGETS * 0.20 * champ.stats.ad()),
        0.,
        0.,
    ) //value for ranged champions
}

//temper passive removed in patch 14.19 (keep this in case it gets reverted)
//fn stridebreaker_temper_enable(champ: &mut Unit, _availability_coef: f32) {
//    if champ.effects_values[EffectValueId::StridebreakerTemperMsFlat] == 0. {
//        let flat_ms_buff: f32 = 20.;
//        champ.stats.ms_flat += flat_ms_buff;
//        champ.effects_values[EffectValueId::StridebreakerTemperMsFlat] = flat_ms_buff;
//    }
//}
//
//fn stridebreaker_temper_disable(champ: &mut Unit) {
//    champ.stats.ms_flat -= champ.effects_values[EffectValueId::StridebreakerTemperMsFlat];
//    champ.effects_values[EffectValueId::StridebreakerTemperMsFlat] = 0.;
//}
//
//const STRIDEBREAKER_TEMPER: TemporaryEffect = TemporaryEffect {
//    id: EffectId::StridebreakerTemper,
//    add_stack: stridebreaker_temper_enable,
//    remove_every_stack: stridebreaker_temper_disable,
//    duration: 2.,
//    cooldown: 0.,
//};

//fn stridebreaker_temper_on_phys_hit(champ: &mut Unit) {
//    champ.add_temporary_effect(&STRIDEBREAKER_TEMPER, champ.stats.item_haste);
//}

const STRIDEBREAKER_CLEAVE_RADIUS: f32 = 350.;
impl Item {
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
            bonus_ad: 40.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(stridebreaker_init),
            special_active: Some(stridebreaker_breaking_shockwave),
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: Some(stridebreaker_cleave),
            on_phys_hit: None, //Some(stridebreaker_temper_on_phys_hit),
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//Sundered sky
const SUNDERED_SKY_COOLDOWN: f32 = 8.;
fn sundered_sky_init(champ: &mut Unit) {
    champ.effects_values[EffectValueId::SunderedSkyLastTriggerTime] =
        -(SUNDERED_SKY_COOLDOWN + F32_TOL); //to allow for effect at time==0
}

fn sundered_sky_lightshield_strike(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    _n_targets: f32,
    from_other_effect: bool,
) -> PartDmg {
    if from_other_effect {
        return PartDmg(0., 0., 0.);
    }

    //if on cooldown, do nothing
    if champ.time - champ.effects_values[EffectValueId::SunderedSkyLastTriggerTime]
        <= SUNDERED_SKY_COOLDOWN
    {
        return PartDmg(0., 0., 0.);
    }
    //if not on cooldown, put on cooldown and trigger effect
    champ.effects_values[EffectValueId::SunderedSkyLastTriggerTime] = champ.time;
    champ.periodic_heals_shields +=
        champ.stats.base_ad + SUNDERED_SKY_LIGHTSHIELD_STRIKE_MISSING_HP * 0.06 * champ.stats.hp;
    let phys_dmg: f32 =
        champ.stats.ad() * (1. - champ.stats.crit_chance) * (champ.stats.crit_dmg - 1.); //bonus dmg from a basic attack with 100% crit chance compared to an average basic_attack
    PartDmg(phys_dmg, 0., 0.)
}

impl Item {
    pub const SUNDERED_SKY: Item = Item {
        id: ItemId::SunderedSky,
        full_name: "Sundered_sky",
        short_name: "Sundered_sky",
        cost: 3100.,
        item_groups: enum_set!(),
        utils: enum_set!(),
        stats: UnitStats {
            hp: 400.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 40.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 0.,
            mr: 0.,
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
            mr_red_percent: 0.,
            life_steal: 0.,
            omnivamp: 0.,
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(sundered_sky_init),
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: Some(sundered_sky_lightshield_strike),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//Sunfire aegis (useless?)

//Terminus
fn terminus_init(champ: &mut Unit) {
    champ.effects_stacks[EffectStackId::TerminusJuxtapositionMode] = 0; //0==light, 1==dark
    champ.effects_stacks[EffectStackId::TerminusJuxtapositionLightStacks] = 0;
    champ.effects_stacks[EffectStackId::TerminusJuxtapositionDarkStacks] = 0;
    champ.effects_values[EffectValueId::TerminusJuxtapositionLightRes] = 0.;
    champ.effects_values[EffectValueId::TerminusJuxtapositionDarkPen] = 0.;
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
    if champ.effects_stacks[EffectStackId::TerminusJuxtapositionLightStacks]
        < TERMINUS_JUXTAPOSITION_MAX_STACKS
    {
        champ.effects_stacks[EffectStackId::TerminusJuxtapositionLightStacks] += 1;
        let res_buff: f32 =
            TERMINUS_JUXTAPOSITION_RES_PER_LIGHT_STACK_BY_LVL[usize::from(champ.lvl.get() - 1)];
        champ.stats.armor += res_buff;
        champ.stats.mr += res_buff;
        champ.effects_values[EffectValueId::TerminusJuxtapositionLightRes] += res_buff;
    }
}

fn terminus_juxtaposition_remove_every_light_stack(champ: &mut Unit) {
    champ.stats.armor -= champ.effects_values[EffectValueId::TerminusJuxtapositionLightRes];
    champ.stats.mr -= champ.effects_values[EffectValueId::TerminusJuxtapositionLightRes];
    champ.effects_values[EffectValueId::TerminusJuxtapositionLightRes] = 0.;
    champ.effects_stacks[EffectStackId::TerminusJuxtapositionLightStacks] = 0;
}

const TERMINUS_JUXTAPOSITION_DURATION: f32 = 5.;
const TERMINUS_JUXTAPOSITION_LIGHT: TemporaryEffect = TemporaryEffect {
    id: EffectId::TerminusJuxtapositionLight,
    add_stack: terminus_juxtaposition_add_light_stack,
    remove_every_stack: terminus_juxtaposition_remove_every_light_stack,
    duration: TERMINUS_JUXTAPOSITION_DURATION,
    cooldown: 0.,
};

const TERMINUS_JUXTAPOSITION_PEN_PER_DARK_STACK: f32 = 0.10;
fn terminus_juxtaposition_add_dark_stack(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_stacks[EffectStackId::TerminusJuxtapositionDarkStacks]
        < TERMINUS_JUXTAPOSITION_MAX_STACKS
    {
        decrease_multiplicatively_scaling_stat(
            &mut champ.stats.armor_pen_percent,
            champ.effects_values[EffectValueId::TerminusJuxtapositionDarkPen],
        ); //decrease value temporarly
        decrease_multiplicatively_scaling_stat(
            &mut champ.stats.magic_pen_percent,
            champ.effects_values[EffectValueId::TerminusJuxtapositionDarkPen],
        ); //decrease value temporarly

        champ.effects_stacks[EffectStackId::TerminusJuxtapositionDarkStacks] += 1;
        champ.effects_values[EffectValueId::TerminusJuxtapositionDarkPen] +=
            TERMINUS_JUXTAPOSITION_PEN_PER_DARK_STACK;

        increase_multiplicatively_scaling_stat(
            &mut champ.stats.armor_pen_percent,
            champ.effects_values[EffectValueId::TerminusJuxtapositionDarkPen],
        );
        increase_multiplicatively_scaling_stat(
            &mut champ.stats.magic_pen_percent,
            champ.effects_values[EffectValueId::TerminusJuxtapositionDarkPen],
        );
    }
}

fn terminus_juxtaposition_remove_every_dark_stack(champ: &mut Unit) {
    decrease_multiplicatively_scaling_stat(
        &mut champ.stats.armor_pen_percent,
        champ.effects_values[EffectValueId::TerminusJuxtapositionDarkPen],
    );
    decrease_multiplicatively_scaling_stat(
        &mut champ.stats.magic_pen_percent,
        champ.effects_values[EffectValueId::TerminusJuxtapositionDarkPen],
    );
    champ.effects_values[EffectValueId::TerminusJuxtapositionDarkPen] = 0.;
    champ.effects_stacks[EffectStackId::TerminusJuxtapositionDarkStacks] = 0;
}

const TERMINUS_JUXTAPOSITION_DARK: TemporaryEffect = TemporaryEffect {
    id: EffectId::TerminusJuxtapositionDark,
    add_stack: terminus_juxtaposition_add_dark_stack,
    remove_every_stack: terminus_juxtaposition_remove_every_dark_stack,
    duration: TERMINUS_JUXTAPOSITION_DURATION,
    cooldown: 0.,
};

fn terminus_on_basic_attack_hit(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    n_targets: f32,
    _from_other_effect: bool,
) -> PartDmg {
    if champ.effects_stacks[EffectStackId::TerminusJuxtapositionMode] == 0 {
        //add light stack and swap mode
        champ.add_temporary_effect(&TERMINUS_JUXTAPOSITION_LIGHT, champ.stats.item_haste);
        champ.effects_stacks[EffectStackId::TerminusJuxtapositionMode] = 1;
    } else {
        //add dark stack and swap mode
        champ.add_temporary_effect(&TERMINUS_JUXTAPOSITION_DARK, champ.stats.item_haste);
        champ.effects_stacks[EffectStackId::TerminusJuxtapositionMode] = 0;
    }
    PartDmg(0., n_targets * (30.), 0.)
}

impl Item {
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
            bonus_ad: 30.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(terminus_init),
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: Some(terminus_on_basic_attack_hit),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//The collector
fn the_collector_init(champ: &mut Unit) {
    champ.effects_stacks[EffectStackId::TheCollectorExecuted] = 0;
}

const THE_COLLECTOR_DEATH_EXECUTE_THRESHOLD: f32 = 0.05;
fn the_collector_death(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    if champ.effects_stacks[EffectStackId::TheCollectorExecuted] != 1
        && champ.dmg_done.as_sum() >= (1. - THE_COLLECTOR_DEATH_EXECUTE_THRESHOLD) * target_stats.hp
    {
        champ.dmg_done.2 += THE_COLLECTOR_DEATH_EXECUTE_THRESHOLD * target_stats.hp;
        champ.effects_stacks[EffectStackId::TheCollectorExecuted] = 1;
    }
    PartDmg(0., 0., 0.)
}

impl Item {
    pub const THE_COLLECTOR: Item = Item {
        id: ItemId::TheCollector,
        full_name: "The_collector",
        short_name: "Collector",
        cost: 3400.,
        item_groups: enum_set!(),
        utils: enum_set!(), //taxes passive not big enough and too situationnal
        stats: UnitStats {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 60.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            lethality: 10.,
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
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(the_collector_init),
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
            on_any_hit: Some(the_collector_death),
        },
    };
}

//Thornmail (useless?)

//Titanic hydra
fn titanic_hydra_titanic_crescent(champ: &mut Unit, target_stats: &UnitStats) -> PartDmg {
    //only return bonus dmg (doesn't take into account if basic_attack hits multiple target e.g. with runaans but not a big deal)
    champ.dmg_on_target(
        target_stats,
        PartDmg(
            0.02 * champ.stats.hp + TITANIC_HYDRA_CLEAVE_AVG_TARGETS * 0.045 * champ.stats.hp,
            0.,
            0.,
        ),
        (1, 1),
        enum_set!(),
        1. + TITANIC_HYDRA_CLEAVE_AVG_TARGETS,
    ) //value for ranged champions
}

fn titanic_hydra_cleave(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    n_targets: f32,
    _from_other_effect: bool,
) -> PartDmg {
    PartDmg(
        TITANIC_HYDRA_CLEAVE_AVG_TARGETS * 0.015 * champ.stats.hp
            + n_targets * (0.005 * champ.stats.hp),
        0.,
        0.,
    ) //value for ranged champions, cleave not affected by n_targets because AoE is behind target
}

impl Item {
    pub const TITANIC_HYDRA: Item = Item {
        id: ItemId::TitanicHydra,
        full_name: "Titanic_hydra",
        short_name: "Titanic_hydra",
        cost: 3300.,
        item_groups: enum_set!(ItemGroups::Hydra),
        utils: enum_set!(),
        stats: UnitStats {
            hp: 600.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 40.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: None,
            special_active: Some(titanic_hydra_titanic_crescent),
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: Some(titanic_hydra_cleave),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//Trailblazer (useless?)

//Trinity force
fn trinity_force_init(champ: &mut Unit) {
    //spellblade generic variables
    spellblade_init(champ);

    //quicken variables
    champ.effects_values[EffectValueId::TrinityForceQuickenMsFlat] = 0.;
}

fn trinity_force_quicken_enable(champ: &mut Unit, _availability_coef: f32) {
    if champ.effects_values[EffectValueId::TrinityForceQuickenMsFlat] == 0. {
        let flat_ms_buff: f32 = 20.;
        champ.stats.ms_flat += flat_ms_buff;
        champ.effects_values[EffectValueId::TrinityForceQuickenMsFlat] = flat_ms_buff;
    }
}

fn trinity_force_quicken_disable(champ: &mut Unit) {
    champ.stats.ms_flat -= champ.effects_values[EffectValueId::TrinityForceQuickenMsFlat];
    champ.effects_values[EffectValueId::TrinityForceQuickenMsFlat] = 0.;
}

const TRINITY_FORCE_QUICKEN: TemporaryEffect = TemporaryEffect {
    id: EffectId::TrinityForceQuicken,
    add_stack: trinity_force_quicken_enable,
    remove_every_stack: trinity_force_quicken_disable,
    duration: 2.,
    cooldown: 0.,
};

fn trinity_force_spellblade_on_basic_attack_hit(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    _n_targets: f32,
    from_other_effect: bool,
) -> PartDmg {
    if from_other_effect {
        return PartDmg(0., 0., 0.);
    }

    //quicken
    champ.add_temporary_effect(&TRINITY_FORCE_QUICKEN, champ.stats.item_haste);

    //spellblade
    //do nothing if not empowered
    if champ.effects_stacks[EffectStackId::SpellbladeEmpowered] != 1 {
        return PartDmg(0., 0., 0.);
    }
    //if empowered (previous condition) but last ability cast from too long ago, reset spellblade
    if champ.time - champ.effects_values[EffectValueId::SpellbladeLastEmpowerTime]
        >= SPELLBLADE_DELAY
    {
        champ.effects_stacks[EffectStackId::SpellbladeEmpowered] = 0;
        return PartDmg(0., 0., 0.);
    }
    //if empowered and last ability cast is recent enough (previous condition), reset and trigger spellblade
    champ.effects_stacks[EffectStackId::SpellbladeEmpowered] = 0;
    champ.effects_values[EffectValueId::SpellbladeLastConsumeTime] = champ.time;
    PartDmg(2. * champ.stats.base_ad, 0., 0.)
}

impl Item {
    pub const TRINITY_FORCE: Item = Item {
        id: ItemId::TrinityForce,
        full_name: "Trinity_force",
        short_name: "Triforce",
        cost: 3333.,
        item_groups: enum_set!(ItemGroups::Spellblade),
        utils: enum_set!(),
        stats: UnitStats {
            hp: 333.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 36.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 0.,
            mr: 0.,
            base_as: 0.,
            bonus_as: 0.30,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(trinity_force_init),
            special_active: None,
            on_ability_cast: Some(spellblade_on_spell_cast),
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: Some(trinity_force_spellblade_on_basic_attack_hit),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//Umbral glaive
impl Item {
    pub const UMBRAL_GLAIVE: Item = Item {
        id: ItemId::UmbralGlaive,
        full_name: "Umbral_glaive",
        short_name: "Umbral_glaive",
        cost: 2600.,
        item_groups: enum_set!(),
        utils: enum_set!(ItemUtils::Special), //blackout passive
        stats: UnitStats {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 50.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 0.,
            mr: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Unending despair (useless?)

//Vigilant wardstone (useless?)

//Void staff
impl Item {
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
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Voltaic cyclosword
fn voltaic_cyclosword_init(champ: &mut Unit) {
    champ.effects_values[EffectValueId::VoltaicCycloswordFirmamentLastTriggerDistance] =
        -(ENERGIZED_ATTACKS_TRAVEL_REQUIRED + F32_TOL); // to allow for effect at time == 0
}

fn voltaic_cyclosword_firmament(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    _n_targets: f32,
    from_other_effect: bool,
) -> PartDmg {
    if from_other_effect {
        return PartDmg(0., 0., 0.);
    }

    //if not enough energy, add basic attack energy stacks
    if champ.units_travelled
        - champ.effects_values[EffectValueId::VoltaicCycloswordFirmamentLastTriggerDistance]
        < ENERGIZED_ATTACKS_TRAVEL_REQUIRED
    {
        champ.effects_values[EffectValueId::VoltaicCycloswordFirmamentLastTriggerDistance] -=
            ENERGIZED_ATTACKS_TRAVEL_REQUIRED * (ENERGIZED_STACKS_PER_BASIC_ATTACK / 100.);
        return PartDmg(0., 0., 0.);
    }
    //if enough energy (previous condition), trigger energized attack
    champ.effects_values[EffectValueId::VoltaicCycloswordFirmamentLastTriggerDistance] =
        champ.units_travelled;
    PartDmg(100., 0., 0.) //slow not implemented
}

impl Item {
    pub const VOLTAIC_CYCLOSWORD: Item = Item {
        id: ItemId::VoltaicCyclosword,
        full_name: "Voltaic_cyclosword",
        short_name: "Voltaic_cyclosword",
        cost: 3000.,
        item_groups: enum_set!(),
        utils: enum_set!(),
        stats: UnitStats {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 55.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 0.,
            mr: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(voltaic_cyclosword_init),
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: Some(voltaic_cyclosword_firmament),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//Warmog's armor (useless?)

//Winter's approach not implemented (Fimbulwinter takes its place)

//Wit's end
fn wits_end_fray(
    _champ: &mut Unit,
    _target_stats: &UnitStats,
    n_targets: f32,
    _from_other_effect: bool,
) -> PartDmg {
    PartDmg(0., n_targets * (45.), 0.)
}

impl Item {
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
            ap_percent: 0.,
            armor: 0.,
            mr: 45.,
            base_as: 0.,
            bonus_as: 0.50,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: None,
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: Some(wits_end_fray),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//Youmuu's ghostblade, haunt passive not implemented
fn youmuus_ghostblade_init(champ: &mut Unit) {
    champ.effects_values[EffectValueId::YoumuusGhostbladeWraithStepMsPercent] = 0.;
}

fn youmuus_ghostblade_wraith_step_active(champ: &mut Unit, _target_stats: &UnitStats) -> PartDmg {
    champ.add_temporary_effect(&YOUMUUS_GHOSTBLADE_WRAITH_STEP, champ.stats.item_haste);
    PartDmg(0., 0., 0.)
}

fn youmuus_ghostblade_wraith_step_enable(champ: &mut Unit, availability_coef: f32) {
    if champ.effects_values[EffectValueId::YoumuusGhostbladeWraithStepMsPercent] == 0. {
        let percent_ms_buff: f32 = availability_coef * 0.15; //ms value for ranged champions
        champ.stats.ms_percent += percent_ms_buff;
        champ.effects_values[EffectValueId::YoumuusGhostbladeWraithStepMsPercent] = percent_ms_buff;
    }
}

fn youmuus_ghostblade_wraith_step_disable(champ: &mut Unit) {
    champ.stats.ms_percent -=
        champ.effects_values[EffectValueId::YoumuusGhostbladeWraithStepMsPercent];
    champ.effects_values[EffectValueId::YoumuusGhostbladeWraithStepMsPercent] = 0.;
}

const YOUMUUS_GHOSTBLADE_WRAITH_STEP: TemporaryEffect = TemporaryEffect {
    id: EffectId::YoumuusGhostbladeWraithStep,
    add_stack: youmuus_ghostblade_wraith_step_enable,
    remove_every_stack: youmuus_ghostblade_wraith_step_disable,
    duration: 6.,
    cooldown: 45.,
};

impl Item {
    pub const YOUMUUS_GHOSTBLADE: Item = Item {
        id: ItemId::YoumuusGhostblade,
        full_name: "Youmuus_ghostblade",
        short_name: "Youmuus",
        cost: 2800.,
        item_groups: enum_set!(),
        utils: enum_set!(),
        stats: UnitStats {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 60.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: Some(youmuus_ghostblade_init),
            special_active: Some(youmuus_ghostblade_wraith_step_active),
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
    };
}

//Yun Tal Wildarrows
fn yun_tal_serrated_edge(
    champ: &mut Unit,
    _target_stats: &UnitStats,
    n_targets: f32,
    _from_other_effect: bool,
) -> PartDmg {
    PartDmg(n_targets * (champ.stats.crit_chance * 70.), 0., 0.)
}

impl Item {
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
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
        on_action_fns: OnActionFns {
            on_lvl_set: None,
            on_fight_init: None,
            special_active: None,
            on_ability_cast: None,
            on_ultimate_cast: None,
            on_ability_hit: None,
            on_ultimate_hit: None,
            on_basic_attack_cast: None,
            on_basic_attack_hit: Some(yun_tal_serrated_edge),
            on_phys_hit: None,
            on_magic_hit: None,
            on_true_dmg_hit: None,
            on_any_hit: None,
        },
    };
}

//Zak'Zak's realmspike

//Zeke's convergence (useless?)

//Zhonya's hourglass
impl Item {
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
            ap_flat: 105.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//
// --- BOOTS LISTING --- //
//

//Berserkers_greaves
impl Item {
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
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Boots of swiftness
impl Item {
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
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Ionian boots of lucidity
impl Item {
    pub const IONIAN_BOOTS_OF_LUCIDITY: Item = Item {
        id: ItemId::IonianBootsOfLucidity,
        full_name: "Ionian_boots_of_lucidity",
        short_name: "Lucidity",
        cost: 900.,
        item_groups: enum_set!(ItemGroups::Boots),
        utils: enum_set!(), //10 summoner spell haste, but not big enough utility
        stats: UnitStats {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
            armor: 0.,
            mr: 0.,
            base_as: 0.,
            bonus_as: 0.,
            ability_haste: 10.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Mercury's treads
impl Item {
    pub const MERCURYS_TREADS: Item = Item {
        id: ItemId::MercurysTreads,
        full_name: "Mercurys_treads",
        short_name: "Mercurys",
        cost: 1300.,
        item_groups: enum_set!(ItemGroups::Boots),
        utils: enum_set!(), //30% tenacity, but not big enough utility
        stats: UnitStats {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Plated steelcaps
impl Item {
    pub const PLATED_STEELCAPS: Item = Item {
        id: ItemId::PlatedSteelcaps,
        full_name: "Plated_steelcaps",
        short_name: "Steelcaps",
        cost: 1200.,
        item_groups: enum_set!(ItemGroups::Boots),
        utils: enum_set!(), //basic attacks dmg reduction not big enough utility
        stats: UnitStats {
            hp: 0.,
            mana: 0.,
            base_ad: 0.,
            bonus_ad: 0.,
            ap_flat: 0.,
            ap_percent: 0.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}

//Sorcerer's shoes
impl Item {
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
            ap_percent: 0.,
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
            ms_flat: 45.,
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
            ability_dmg_modifier: 0.,
            phys_dmg_modifier: 0.,
            magic_dmg_modifier: 0.,
            true_dmg_modifier: 0.,
            tot_dmg_modifier: 0.,
        },
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
    };
}
