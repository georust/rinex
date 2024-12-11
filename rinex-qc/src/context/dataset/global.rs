/// Global data identifier, allows identifing a specific set within
/// multiple [GlobalData]
enum GlobalDataIdenfier {
    // Identified by production agency.
    Agency(String),
    /// Identified by local area
    LocalArea(String),
}

/// Global data that may apply to both [DataSet] and enhanced capabilities.
/// This type of data is usually valid worlwide
enum GlobalData {
    #[cfg(feature = "sp3")]
    SP3(SP3),
    IONEX(Rinex),
    BroadcastNavigation(Rinex),
}

struct GlobalDataSet {
    data: HashMap<GlobalIdentifier, GlobalData>,
}