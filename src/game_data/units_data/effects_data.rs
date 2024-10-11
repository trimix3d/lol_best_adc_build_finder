use super::units_data::Unit;

use enum_map::Enum;

use core::hash::{Hash, Hasher};

#[derive(Enum, Debug, PartialEq, Eq, Hash)]
pub(crate) enum EffectId {
    //the convention to name variants is to write, in CamelCase (with no extra space between them), the following in order :
    // - the name of the source of the passive/active effect (either an item or a champion ability)
    // - the name of the passive/active effect
    //exemple: YoumuusGhostbladeWraithStep
    AsheRangersFocus,
    BlackCleaverCarve,
    BlackCleaverFervor,
    Conqueror,
    CosmicDriveSpellDance,
    DravenThrowAxe1,
    DravenThrowAxe2,
    DravenBloodRush,
    ExperimentalHexplateOverdrive,
    EzrealRisingSpellForce,
    FleetFootworkMS,
    GuinsoosRagebladeSeethingStrike,
    KaisaSuperchargeAS,
    LethalTempoAS,
    LiandrysTormentSuffering,
    LucianArdentBlazeMS,
    MalignanceHatefogCurse,
    OpportunityPreparation,
    PhantomDancerSpectralWalkz,
    RiftmakerVoidCorruption,
    SivirFleetOfFoot,
    SivirOnTheHuntMS,
    SivirRicochet,
    SpearOfShojinFocusedWill,
    StormsurgeStormraiderMS,
    StridebreakerBreakingShockwaveMS,
    StridebreakerTemper,
    TerminusJuxtapositionLight,
    TerminusJuxtapositionDark,
    TrinityForceQuicken,
    VarusRAddDelayedBlightStacks05,
    VarusRAddDelayedBlightStacks10,
    VarusRAddDelayedBlightStacks15,
    XayahDeadlyPlumageAS,
    XayahDeadlyPlumageMS,
    YoumuusGhostbladeWraithStep,
}

//If you have the choice, prefer using EffectfStackId over EffectValueId, as working with integers is more reliable than floats
#[derive(Enum, Debug)]
pub(crate) enum EffectStackId {
    //the convention to name variants is to write, in CamelCase (with no extra space between them), the following in order :
    // - the name of the source of the passive/active effect (either an item or a champion ability)
    // - the name of the passive/active effect
    // - "Stacks" at the end (+optionally, additionnal relevant information)
    //exemple: KrakenSlayerBringItDownStacks
    AsheFocusStacks,
    BlackCleaverCarveStacks,
    CaitlynBonusHeadshot,
    CaitlynHeadshotStacks,
    ConquerorAdaptiveIsPhys,
    ConquerorStacks,
    DravenAxesInAir,
    DravenAxesInHand,
    EclipseEverRisingMoonStacks,
    EzrealRisingSpellForceStacks,
    EzrealEssenceFluxMark,
    GuinsoosRagebladeSeethingStrikeStacks,
    GuinsoosRagebladePhantomStacks,
    HullbreakerSkipperStacks,
    KaisaSecondSkinStacks,
    KaisaQEvolved,
    KaisaWEvolved,
    //KaisaEEvolved, //e evolve invisibility not implemented
    KrakenSlayerBringItDownStacks,
    LethalTempoStacks,
    LucianLightslingerEmpowered,
    LucianVigilanceProcsRemaning,
    PhantomDancerSpectralWalkzStacks,
    PressTheAttackStacks,
    SpearOfShojinFocusedWillStacks,
    SpellbladeEmpowered,
    StormsurgeStormraiderTriggered,
    TerminusJuxtapositionMode,
    TerminusJuxtapositionLightStacks,
    TerminusJuxtapositionDarkStacks,
    TheCollectorExecuted,
    VarusBlightStacks,
    VarusBlightedQuiverEmpowered,
    XayahNFeathersOnGround,
    XayahCleanCutsStacks,
}

#[derive(Enum, Debug)]
pub(crate) enum EffectValueId {
    //the convention to name variants is to write, in CamelCase (with no extra space between them), the following in order :
    // - the name of the source of the passive/active effect (either an item or a champion spell)
    // - the name of the passive/active effect
    // - the name of the affected stat (+optionally, additionnal relevant information)
    //exemple : YoumuusGhostbladeWraithStepMsPercent
    AsheLastFrostTime,
    AsheRangersFocusBonusAS,
    BlackCleaverCarveArmorRedPercent,
    BlackCleaverFervorMsFlat,
    BlackfireTorchBalefulBlazeLastApplicationTime,
    ConquerorAdaptiveAP,
    ConquerorOmnivamp,
    ConquerorLastAbilityHitTime,
    ConquerorLastBasicAttackHitTime,
    CosmicDriveSpellDanceMsFlat,
    DeadMansPlateShipwreckerLastHitdistance,
    DravenBloodRushBonusAS,
    DravenBloodRushBonusMsPercent,
    EclipseEverRisingMoonLastStackTime,
    EclipseEverRisingMoonLastTriggerTime,
    EzrealEssenceFluxHitTime,
    EzrealRisingSpellForceBonusAS,
    ExperimentalHexplateOverdriveBonusAS,
    ExperimentalHexplateOverdriveMsPercent,
    FleetFootworkLastTriggerDistance,
    FleetFootworkMSPercent,
    GuinsoosRagebladeSeethingStrikeBonusAS,
    HullbreakerSkipperLastStackTime,
    KaisaSecondSkinLastStackTime,
    KaisaSuperchargeBonusAS,
    KrakenSlayerBringItDownLastStackTime,
    LethalTempoBonusAS,
    LiandrysTormentTormentLastApplicationTime,
    LiandrysTormentSufferingCombatStartTime,
    LiandrysTormentSufferingTotDmgModifier,
    LucianArdentBlazeMsFlat,
    LudensCompanionFireLastConsumeTime,
    MalignanceHatefogCurseMrRedFlat,
    MuramanaShockLastSpellHitTime,
    OpportunityPreparationLethality,
    PressTheAttackLastStackTime,
    RiftmakerVoidCorruptionTotDmgModifier,
    RiftmakerVoidCorruptionCombatStartTime,
    RiftmakerVoidCorruptionOmnivamp,
    RapidFirecannonSharpshooterLastTriggerDistance,
    SpellbladeLastEmpowerTime,
    SpellbladeLastConsumeTime,
    SivirFleetOfFootMsFlat,
    SivirOnTheHuntMsPercent,
    SivirRicochetBonusAS,
    SpearOfShojinFocusedWillAbilityDmgModifier,
    StormsurgeStormraiderMsPercent,
    StridebreakerTemperMsFlat,
    StridebreakerBreakingShockwaveMsPercent,
    SunderedSkyLastTriggerTime,
    TerminusJuxtapositionLightRes,
    TerminusJuxtapositionDarkPen,
    TrinityForceQuickenMsFlat,
    VarusBlightLastStackTime,
    VoltaicCycloswordFirmamentLastTriggerDistance,
    XayahDeadlyPlumageBonusAS,
    XayahDeadlyPlumageMsPercent,
    XayahWBasicAttackCoef,
    YoumuusGhostbladeWraithStepMsPercent,
}

#[derive(Debug)]
pub(crate) struct TemporaryEffect {
    pub(crate) id: EffectId,
    /// Adds effect stats AND records the added value on the unit (in `Unit.effect_values` or `Unit.effects_stacks`).
    ///
    /// First argument is the Unit to add a stack to.
    /// The second argument (`availability_coef`) should multiply every effect stat that is added to the Unit beforehand,
    /// it exists to weight effects with different cooldowns (an effect with a longer cooldown should weight less than the same effect with a smaller cooldown).
    pub(crate) add_stack: fn(&mut Unit, f32),
    /// Removes effect stats AND resets to zero the associated values on the unit (in `Unit.effects_values` or `Unit.effects_stacks`).
    pub(crate) remove_every_stack: fn(&mut Unit),
    pub(crate) duration: f32,
    pub(crate) cooldown: f32,
}

impl PartialEq for TemporaryEffect {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id //assumes every effect id is different, or rather that i'm not too retaaarded to put the same id on different effects
    }
}
impl Eq for TemporaryEffect {}

impl Hash for TemporaryEffect {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.id.hash(hasher);
    }
}
