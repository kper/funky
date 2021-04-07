/// the exploded graph representation of IR code
pub mod graph;
/// this module converts the exploded graph into a .tex file for display
pub mod tikz;

/// convert the IR into an ifds instance
pub mod convert;

/// contains the flow functions which be used to build up the graph
pub mod flowfuncs;