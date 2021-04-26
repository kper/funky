/// the exploded graph representation of IR code
pub mod graph;
/// this module converts the exploded graph into a .tex file for display
pub mod tikz;

/// this module converts the exploded graph into a .tex file for display
/// sparsed
pub mod tikz2;
/// contains the flow functions which be used to build up the graph
pub mod flowfuncs;

/// Keeps node information about the program
pub mod state;
/// Different tabulation strategies
pub mod tabulation;