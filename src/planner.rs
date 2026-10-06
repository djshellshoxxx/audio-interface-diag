use crate::model::Edition;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestKind {
    DeviceInventory, DriverCapability, PassiveStreamWatch, LevelWatch, ClippingWatch,
    XrunWatch, CallbackTiming, SessionChangeWatch, ProcessLoopback,
    BufferSweep, RoundTripLatency, ChannelMap, Polarity, DcOffset,
    NoiseFloor, FrequencyResponse, Crosstalk, ClockDrift, BurnIn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedTest { pub kind: TestKind, pub intrusive: bool, pub requires_loopback_cable: bool }

pub fn plan(edition: Edition) -> Vec<PlannedTest> {
    use TestKind::*;
    let passive=|kind| PlannedTest{kind,intrusive:false,requires_loopback_cable:false};
    let active=|kind,cable| PlannedTest{kind,intrusive:true,requires_loopback_cable:cable};
    match edition {
        Edition::Live => vec![passive(DeviceInventory),passive(DriverCapability),passive(PassiveStreamWatch),passive(LevelWatch),passive(ClippingWatch),passive(XrunWatch),passive(CallbackTiming),passive(SessionChangeWatch)],
        Edition::Daw => vec![passive(DeviceInventory),passive(DriverCapability),passive(PassiveStreamWatch),passive(LevelWatch),passive(ClippingWatch),passive(XrunWatch),passive(CallbackTiming),passive(SessionChangeWatch),passive(ProcessLoopback)],
        Edition::Engineer => vec![passive(DeviceInventory),passive(DriverCapability),active(BufferSweep,false),active(RoundTripLatency,true),active(ChannelMap,true),active(Polarity,true),active(DcOffset,true),active(NoiseFloor,true),active(FrequencyResponse,true),active(Crosstalk,true),active(ClockDrift,true),active(BurnIn,true)],
    }
}
