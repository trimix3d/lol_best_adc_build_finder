use super::units_data::Unit;

use enum_map::Enum;

use core::hash::{Hash, Hasher};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Enum)]
pub enum BuffId {
    //the convention to name variants is to write, in CamelCase (with no extra space between them), the following in order :
    // - the name of the source of the passive/active effect (either an item or a champion spell)
    // - the name of the passive/active effect
    //exemple: YoumuusGhostbladeWraithStep
    AsheRangersFocus,
    BlackCleaverCarve,
    BlackCleaverFervor,
    CosmicDriveSpellDance,
    DravenThrowAxe1,
    DravenThrowAxe2,
    DravenBloodRush,
    ExperimentalHexplateOverdrive,
    EzrealRisingSpellForce,
    GuinsoosRagebladeSeethingStrike,
    KaisaSuperchargeAS,
    LucianArdentBlazeMS,
    MalignanceHatefogCurse,
    OpportunityPreparation,
    PhantomDancerSpectralWalkz,
    RiftmakerVoidCorruption,
    SivirFleetOfFoot,
    SivirOnTheHuntMS,
    SivirRicochet,
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

//If you have the choice, prefer using BuffStackId over BuffValueId, as working with integers is more reliable than floats
#[derive(Debug, enum_map::Enum, Clone, Copy)]
pub enum BuffStackId {
    //the convention to name variants is to write, in CamelCase (with no extra space between them), the following in order :
    // - the name of the source of the passive/active effect (either an item or a champion spell)
    // - the name of the passive/active effect
    // - "Stacks" at the end (+optionally, additionnal relevant information)
    //exemple: KrakenSlayerBringItDownStacks
    AsheFrosted,
    AsheFocusStacks,
    BlackCleaverCarveStacks,
    CaitlynBonusHeadshot,
    CaitlynHeadshotStacks,
    DravenAxesInAir,
    DravenAxesInHand,
    EclipseEverRisingMoonStacks,
    EzrealRisingSpellForceStacks,
    EzrealWMark,
    SpellbladeEmpowered,
    GuinsoosRagebladeSeethingStrikeStacks,
    GuinsoosRagebladePhantomStacks,
    HullbreakerSkipperStacks,
    KaisaSecondSkinStacks,
    KaisaQEvolved,
    KaisaWEvolved,
    //KaisaEEvolved, //e evolve invisibility not implemented
    KrakenSlayerBringItDownStacks,
    LucianLightslingerEmpowered,
    LucianVigilanceProcsRemaning,
    PhantomDancerSpectralWalkzStacks,
    SpearOfShojinFocusedWillStacks,
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

#[derive(Debug, enum_map::Enum, Clone, Copy)]
pub enum BuffValueId {
    //the convention to name variants is to write, in CamelCase (with no extra space between them), the following in order :
    // - the name of the source of the passive/active effect (either an item or a champion spell)
    // - the name of the passive/active effect
    // - the name of the affected stat (+optionally, additionnal relevant information)
    //exemple : YoumuusGhostbladeWraithStepMsPercent
    AsheRangersFocusBonusAS,
    BlackCleaverCarveArmorRedPercent,
    BlackCleaverFervorMsFlat,
    BlackfireTorchBalefulBlazeLastApplicationTime,
    CosmicDriveSpellDanceMsFlat,
    DravenBloodRushBonusAS,
    DravenBloodRushBonusMsPercent,
    EclipseEverRisingMoonLastStackTime,
    EclipseEverRisingMoonLastTriggerTime,
    EzrealRisingSpellForceBonusAS,
    SpellbladeLastEmpowerTime,
    SpellbladeLastConsumeTime,
    ExperimentalHexplateOverdriveBonusAS,
    ExperimentalHexplateOverdriveMsPercent,
    HullbreakerSkipperLastStackTime,
    OpportunityPreparationLethality,
    YoumuusGhostbladeWraithStepMsPercent,
    DeadMansPlateShipwreckerLastHitdistance,
    KaisaSuperchargeBonusAS,
    KrakenSlayerBringItDownLastStackTime,
    LiandrysTormentTormentLastApplicationTime,
    LiandrysTormentSufferingCombatStartTime,
    LiandrysTormentSufferingLastHitTime,
    LucianArdentBlazeMsFlat,
    LudensCompanionFireLastConsumeTime,
    MalignanceHatefogCurseMrRedFlat,
    MuramanaShockLastSpellHitTime,
    RiftmakerVoidCorruptionCoef,
    RiftmakerVoidCorruptionCombatStartTime,
    RiftmakerVoidCorruptionOmnivamp,
    RapidFirecannonSharpshooterLastTriggerDistance,
    SivirFleetOfFootMsFlat,
    SivirOnTheHuntMsPercent,
    SivirRicochetBonusAS,
    SpearOfShojinFocusedWillLastHitTime,
    StormsurgeStormraiderMsPercent,
    StridebreakerTemperMsFlat,
    StridebreakerBreakingShockwaveMsPercent,
    SunderedSkyLastTriggerTime,
    TerminusJuxtapositionLightRes,
    TerminusJuxtapositionDarkPen,
    TrinityForceQuickenMsFlat,
    VoltaicCycloswordFirmamentLastTriggerDistance,
    XayahDeadlyPlumageBonusAS,
    XayahDeadlyPlumageMsPercent,
    XayahWBasicAttackCoef,
}

#[derive(Debug)]
pub struct TemporaryBuff {
    pub id: BuffId,
    /// Adds buff stats AND records the added value on the unit (in `Unit.buff_values` or `Unit.buffs_stacks`).
    ///
    /// First argument is the Unit to add a stack to.
    /// The second argument (`availability_coef`) should multiply every buff stat that is added to the Unit beforehand,
    /// it exists to weight effects with different cooldowns (an effect with a longer cooldown should weight less than the same effect with a smaller cooldown).
    pub add_stack: fn(&mut Unit, f32),
    /// Removes buff stats AND resets to zero the associated values on the unit (in `Unit.buffs_values` or `Unit.buffs_stacks`).
    pub remove_every_stack: fn(&mut Unit),
    pub duration: f32,
    pub cooldown: f32,
}

impl PartialEq for TemporaryBuff {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id //assumes every Buff id is different, or rather that i'm not too retaaarded to put the same id on different buffs
    }
}
impl Eq for TemporaryBuff {}

impl Hash for TemporaryBuff {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.id.hash(hasher);
    }
}
