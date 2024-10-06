use std::collections::BTreeMap;

#[derive(Copy, Clone, Debug)]
pub enum Combination {
    /// The [GeometryFree] (GF) combination
    GeometryFree,
    /// The [IonosphereFree) (IF) combination
    IonosphereFree,
    /// The [WideLane] (Wl) combination
    WideLane,
    /// The [NarrowLane] (Nl) combination
    NarrowLane,
    /// The [MelbourneWubbena] (MW) special combination
    MelbourneWubbena,
}

pub trait Combine<K, V> {
    /// Form desired [Combination] from [Self]
    fn combine(&self, combination: Combination) -> BTreeMap<K, V>;
}
