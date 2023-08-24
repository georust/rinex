#![allow(dead_code)]

/// Important articles and references that proved useful when designing this library
pub enum Bibliography {
    /// J. Lesouple, 2019: *Estimation Parcimonieuse de Biais Multitrajets pour Systemes GNSS*.
    /// Pseudo range calculation method on page 50.  
    /// Kepler Solver on page 159.   
    /// Elevation and Azimuth angles determination, page 160.
    /// [DOI](http://perso.recherche.enac.fr/~julien.lesouple/fr/publication/thesis/THESIS.pdf?fbclid=IwAR3WlHm0eP7ygRzywbL07Ig-JawvsdCEdvz1umJJaRRXVO265J9cp931YyI)
    JLe19
    /// ESA NAVIPedia: *Combining Pairs of signals and clock definitions*
    /// [DOI](https://gssc.esa.int/navipedia/index.php/Combining_pairs_of_signals_and_clock_definition)
    ESAGnssCombination,
    /// ASCE Appendix 3: *Calculation of Satellite Position from Ephemeris Data*
    /// [DOI](https://ascelibrary.org/doi/pdf/10.1061/9780784411506.ap03)
    AsceAppendix3,
}
