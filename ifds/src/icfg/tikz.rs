use crate::counter::Counter;
use crate::icfg::graph::{Edge, Fact, Graph};
use crate::icfg::state::State;
use log::debug;
use std::cmp::Ordering;

const TAU: usize = 1;

struct FactWrapper<'a> {
    id: usize,
    fact: &'a Fact,
}

impl<'a> FactWrapper<'a> {
    pub fn get(&'a self) -> &'a Fact {
        self.fact
    }
}

/// Convert the given graph into string in .tex format.
pub fn render_to(graph: &Graph, state: &State) -> String {
    let mut str_vars = String::new();

    let mut index = 0;
    let mut functions: Vec<_> = state.functions.iter().map(|(_, f)| f).collect();

    functions.sort_by(|a, b| b.name.cmp(&a.name));

    let mut counter = Counter::default();

    let mut facts: Vec<_> = graph
        .edges
        .iter()
        .map(|x| x.get_from())
        .chain(graph.edges.iter().map(|x| x.to()))
        .collect();

    fn chain_ordering(o1: Ordering, o2: Ordering) -> Ordering {
        match o1 {
            Ordering::Equal => o1,
            _ => o2,
        }
    }

    facts.sort_by(|a, b| {
        chain_ordering(
            chain_ordering(
                b.function.cmp(&a.function),
                b.belongs_to_var.cmp(&a.belongs_to_var),
            ),
            b.next_pc.cmp(&a.next_pc),
        )
    });
    facts.dedup();

    let facts = facts
        .into_iter()
        .map(|x| FactWrapper {
            id: counter.get(),
            fact: x,
        })
        .collect::<Vec<_>>();

    for function in functions {
        let function_name = &function.name;
        debug!("Drawing function {}", function_name);

        let facts = facts.iter().filter(|x| &x.get().function == function_name);

        let notes = state.notes.iter().filter(|x| &x.function == function_name);

        let mut vars : Vec<_> = facts.clone().map(|x| &x.get().belongs_to_var).collect();
        vars.sort_by(|a, b| {
            b.cmp(a)
        });
        vars.dedup();

        for fact in facts {
            debug!("Drawing fact");

            str_vars.push_str(&format!(
                "\\node[circle,fill,inner sep=1pt,label=left:${}$] ({}) at ({}, {}) {{ }};\n",
                fact.get().belongs_to_var.replace("%", "\\%"),
                fact.id,
                index + vars.iter().position(|x| x == &&fact.get().belongs_to_var).unwrap(),
                fact.get().next_pc,
            ));
        }

        for note in notes {
            debug!("Drawing note");

            str_vars.push_str(&format!(
                "\\node[font=\\tiny] (note_{}) at ({}, {}) {{{}}};\n",
                note.id,
                index as f64 - 1.5,   //x
                note.pc as f64 + 0.5, //y
                note.note.replace("%", "").replace("\"", "").escape_debug(),
            ));
        }

        index += function.definitions + TAU + 1;
    }

    let get_fact_id = |fact| {
        let pos = facts.iter().position(|x| x.get() == fact).unwrap();
        facts.get(pos).unwrap().id
    };

    for edge in graph.edges.iter() {
        match edge {
            Edge::Normal { from, to, curved } => {
                if *curved {
                    str_vars.push_str(&format!(
                        "\t\t\\path[->, bend right] ({}) edge ({});\n",
                        get_fact_id(from),
                        get_fact_id(to)
                    ));
                } else {
                    str_vars.push_str(&format!(
                        "\t\t\\path[->] ({}) edge ({});\n",
                        get_fact_id(from),
                        get_fact_id(to)
                    ));
                }
            }
            Edge::Call { from, to } => {
                str_vars.push_str(&format!(
                    "\t\t\\path[->, green] ({}) [bend left] edge node {{ }} ({});\n",
                    get_fact_id(from),
                    get_fact_id(to)
                ));
            }
            Edge::CallToReturn { from, to } => {
                str_vars.push_str(&format!(
                    "\t\t\\path[->] ({}) edge node {{ }} ({});\n",
                    get_fact_id(from),
                    get_fact_id(to)
                ));
            }
            Edge::Return { from, to } => {
                str_vars.push_str(&format!(
                    "\t\t\\path[->, red] ({}) [bend right] edge  node {{ }} ({});\n",
                    get_fact_id(from),
                    get_fact_id(to)
                ));
            }
            Edge::Path { from, to } => {
                if from != to {
                    str_vars.push_str(&format!(
                        "\t\t\\path[->, blue] ({}) [bend right] edge  node {{ }} ({});\n",
                        get_fact_id(from),
                        get_fact_id(to)
                    ));
                } else {
                    str_vars.push_str(&format!(
                        "\t\t\\path[->, blue] ({}) [loop right] edge  node {{ }} ({});\n",
                        get_fact_id(from),
                        get_fact_id(to)
                    ));
                }
            }
            Edge::Summary { from, to } => {
                str_vars.push_str(&format!(
                    "\t\t\\path[->, red] ({}) [bend left] edge  node {{ }} ({});\n",
                    get_fact_id(from),
                    get_fact_id(to)
                ));
            }
        }
    }

    template(str_vars)
}

fn template(inject: String) -> String {
    format!(
        "   \\documentclass{{standalone}}
            \\usepackage{{pgf, tikz}}
            \\usetikzlibrary{{arrows, automata}}
            \\begin{{document}}
                \\begin{{tikzpicture}}
                    [node distance=3cm]
                    \\tikzstyle{{every state}}=[
                        draw = black,
                        thick,
                        fill = white,
                        minimum size = 4mm
                    ]

                    {}

                \\end{{tikzpicture}}
            \\end{{document}}",
        inject
    )
}
