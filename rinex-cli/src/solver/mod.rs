use rinex_qc::QcContext;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Error)]
pub(crate) enum Error {
    #[error("provided context is either unsufficient or invalid for any position solving")]
    Unfeasible,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum SolverType {
    /// SPP : code based
    SPPSolver,
    /// PPP : phase + code based, the ultimate
    PPPSolver,
}

impl std::fmt::Display for SolverType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::SPPSolver => write!(fmt, "SPP"),
            Self::PPPSolver => write!(fmt, "PPP"),
        }
    }
}

impl SolverType {
    fn from(ctx: &QcContext) -> Result<Self, Error> {
        if ctx.primary_data().is_observation_rinex() {
            if ctx.has_sp3() {
                Ok(Self::PPPSolver)
            } else {
                if ctx.has_navigation_data() {
                    Ok(Self::SPPSolver)
                } else {
                    Err(Error::Unfeasible)
                }
            }
        } else {
            Err(Error::Unfeasible)
        }
    }
}

pub(crate) struct Solver<'a> {
    pub solver: SolverType,
    context: &'a QcContext,
}

impl<'a> Solver<'a> {
    pub fn from(context: &'a QcContext) -> Result<Self, Error> {
        Ok(Self {
            context,
            solver: SolverType::from(context)?,
        })
    }
    pub fn ppp(&self) -> bool {
        self.solver == SolverType::PPPSolver
    }
}
