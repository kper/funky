use crate::icfg::graph::{Edge, Graph};
use log::debug;

const TAU: usize = 1;

/// Convert the given graph into string in .tex format.
pub fn render_to(graph: &Graph) -> String {
    let mut str_vars = String::new();

    let mut index = 0;
    let mut functions: Vec<_> = graph.functions.iter().map(|(_, f)| f).collect();

    functions.sort_by(|a, b| b.name.cmp(&a.name));

    for function in functions {
        let function_name = &function.name;
        debug!("Drawing function {}", function_name);

        //let facts = graph.facts.iter().filter(|x| &x.function == function_name);
        let mut facts: Vec<_> = graph
            .edges
            .iter()
            .map(|x| x.get_from())
            .chain(graph.edges.iter().map(|x| x.to()))
            .filter(|x| &x.function == function_name)
            .collect();

        facts.sort_by(|a, b| b.id.cmp(&a.id));
        facts.dedup();

        let notes = graph.notes.iter().filter(|x| &x.function == function_name);

        for fact in facts {
            debug!("Drawing fact");

            str_vars.push_str(&format!(
                "\\node[circle,fill,inner sep=1pt,label=left:{}] ({}) at ({}, {}) {{ }};\n",
                fact.belongs_to_var.replace("%", "\\%"),
                fact.id,
                index + fact.track,
                fact.next_pc,
                //fact.id
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

    for edge in graph.edges.iter() {
        match edge {
            Edge::Normal { from, to, curved } => {
                if *curved {
                    str_vars.push_str(&format!(
                        "\t\t\\path[->, bend right] ({}) edge ({});\n",
                        from.id, to.id
                    ));
                } else {
                    str_vars.push_str(&format!("\t\t\\path[->] ({}) edge ({});\n", from.id, to.id));
                }
            }
            Edge::Call { from, to } => {
                str_vars.push_str(&format!(
                    "\t\t\\path[->, green] ({}) [bend left] edge node {{ }} ({});\n",
                    from.id, to.id
                ));
            }
            Edge::CallToReturn { from, to } => {
                str_vars.push_str(&format!(
                    "\t\t\\path[->] ({}) edge node {{ }} ({});\n",
                    from.id, to.id
                ));
            }
            Edge::Return { from, to } => {
                str_vars.push_str(&format!(
                    "\t\t\\path[->, red] ({}) [bend right] edge  node {{ }} ({});\n",
                    from.id, to.id
                ));
            }
            Edge::Path { from, to } => {
                if from != to {
                    str_vars.push_str(&format!(
                        "\t\t\\path[->, blue] ({}) [bend right] edge  node {{ }} ({});\n",
                        from.id, to.id
                    ));
                } else {
                    str_vars.push_str(&format!(
                        "\t\t\\path[->, blue] ({}) [loop right] edge  node {{ }} ({});\n",
                        from.id, to.id
                    ));
                }
            }
            Edge::Summary { from, to } => {
                str_vars.push_str(&format!(
                    "\t\t\\path[->, red] ({}) [bend left] edge  node {{ }} ({});\n",
                    from.id, to.id
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