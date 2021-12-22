/// `NavigationFrameError` describes
/// navigation frames specific errors
enum NavigationFrameError {
    
}

/// `NavigationBlock` describes
/// a block of navigation messages
struct NavigationBlock {
    frames: Vec<NavigationFrame>,
}

impl std::FromStr for NavigationBlock {
    type Err = NavigationFrameError;
    /// Builds a NavigationBlock from extracted content
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        
    }
}

/// `NavigationFrame` describes
/// Rinex body frames when
/// Rinex::Header::type::NavigationMessage
struct NavigationFrame {
}
