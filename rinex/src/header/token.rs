use crate::{

};

#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DcbCompensation {
    /// URL: source of corrections
    pub url: String,
    /// Program used for DCBs evaluation and compensation
    pub program: String,
    /// Constellation to which this compensation applies to
    pub constellation: Constellation,
}

/// PCV compensation description
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PcvCompensation {
    /// URL: source of corrections
    pub url: String,
    /// Program used for PCVs evaluation and compensation
    pub program: String,
    /// Constellation to which this compensation applies to
    pub constellation: Constellation,
}

/// Used when parsing a RINEX file.
/// RINEX files should always start with this token
#[derive(Debug, Clone, Copy)]
pub struct VersionTypeConstellToken {
    /// File Revision
    pub version: Version,
    /// File Format
    pub rinex: RinexType,
    /// Constellation (set to [Constellation::Mixed]
    /// when several can be found. May not apply to 
    /// some formats like [RinexType::Meteo]
    pub constellation: Option<Constellation>,
} 

/// ProgramRunBy is used to define the file production context
pub struct ProgramRunByDateToken {
    /// File Creation Date
    pub date: Epoch,
    /// Operator
    pub run_by: String,c
    /// Program name
    pub program: String,
} 

/// [ObservationTypeof] defines
pub struct ObservationTypeof {
    /// The number of [Observable] to be found
    pub count: u32,
    /// List of [Observable]s
    pub observables: Vec<Observable>,
}

/// Token when parsing Header sections
#[derive(Debug, Clone)]
pub enum Token<'a> {
    /// Comments are encountered in the Header section
    /// and stored in [Header] "as is"
    Comment(&'a str),
    /// File Version definition
    VersionTypeConstell(),
    /// ANTEX calibration method specs
    AntexMethodByDate(AntexMethodByDate),
    /// ANTEX specific Number of frequencies
    AntexNumberFrequencies(u32),
    /// ANTEX specific Phase Center Variation
    AntexPcvType(AntexPcvType),
    /// Antenna specifications, usually found in
    /// ANTEX, NAV and OBS formats
    Antenna(Antenna),
    /// Antenna position offset (ENU)
    AntennaEnuOffset(Point3d),
    /// Special CRINEX compressed header
    CRINEX(CRINEX),
    /// ProgramRunBy 
    ProgramRunBy(ProgramRunBy),
    /// ObservationTypes is very important and describes
    /// The type of observations to follow (and their number)
    ObservationType(ObservationType),
    /// GeodeticMarker
    GeodeticMarker(),
    /// Name of the geodetic marker
    GeodeticMarkerName(&'a str),
    /// Receiver definition
    GnssReceiver(GnssReceiver),
    /// Leap Second counter
    LeapSecond(LeapSecondCounter),
    /// Clock specific [TimeScale] definition.
    /// Defined by "TIME SYSTEM ID"
    ClockTimescale(TimeScale),
    /// Clock Analysis center
    ClockAnalysisCenter(&'a str),
    /// Phase Center compensation context
    /// Defined by "SYS / PCVS APPLIED"
    PhaseCenterCompensation(PCVCompensationSpecs),
    /// Phase Center compensation context
    /// Defined by "SYS / PCVS APPLIED"
    DifferentialCodeBiasCompensation(DCBCompensationSpecs),
    /// IONEX specific number of maps to be found
    IONEXNumMaps(u32),
    /// IONEX specific Epoch of First map.
    /// Very similar to [Self::ObservationTimeofFirst]
    IONEXTimeofFirst(Epoch),
    /// IONEX specific Epoch of First map.
    /// Very similar to [Self::ObservationTimeofLast]
    IONEXTimeofLast(Epoch),
    /// IONEX map definition
    IONEXMapDimensions(u8),
    /// IONEX Base Radius definition
    IONEXBaseRadius()
    IONEXObservables()
    /// Number of Clocks to be found
    ClockNumClocks(u32),
    /// Clock specific number of solutions,
    /// defined by "# OF SOLN STA / TRF"
    ClockNumSolutions(ClockNumSolutions),
    /// Defines approximate location of the GNSS receiver.
    /// Useful, mostly in static surveys
    ApproxPosition(Point3d),
    /// SamplingPeriod specs
    SamplingPeriod(Duration),
    /// True when Receiver Clock Offset is compensated for
    ReceiverClockOffsetCompensation(bool),
    /// Number of [SV]s found in this record
    ObservationNumSat(u32),
    /// Time of First Observation is very important and 
    /// specificies the starting point (in time) _and the [TimeScale]_
    /// used in the following record.
    /// Either [Self::TimeOfFirstObservation] or [Self::TimeOfLastObservation]
    /// needs to be defined for a valid Observation RINEX.
    /// This library can deal with one of them missing (either one).
    ObservationTimeofFirst(Epoch),
    /// Time of First Observation is very important and 
    /// specificies the starting point (in time) _and the [TimeScale]_
    /// used in the following record.
    /// Either [Self::TimeOfFirstObservation] or [Self::TimeOfLastObservation]
    /// needs to be defined for a valid Observation RINEX.
    /// This library can deal with one of them missing (either one).
    ObservationTimeofLast(Epoch),
    /// Wavelength
    WavelengthFactL1L2(WavelengthFactor),
    /// Special Marker specifying that file body is starting on next line.
    EndOfHeader,
}
