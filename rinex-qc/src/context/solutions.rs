// The type of [Solutions] we can generate from [QcContext]
pub enum Solutions {
    CGGTTS(Cggtts),
    PVT(PVTSolutions),
}