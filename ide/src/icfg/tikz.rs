use crate::icfg::graph2::{Edge, Graph};
use log::debug;

pub const TAU: usize = 1;

pub fn render_to(graph: &Graph) -> String {
    let mut str_vars = String::new();

    let mut index = 0;
    let mut functions: Vec<_> = graph.functions.iter().map(|(_, f)| f).collect();

    functions.sort_by(|a, b| b.name.cmp(&a.name));

    for function in functions {
        let function_name = &function.name;
        debug!("Drawing function {}", function_name);

        let facts = graph.facts.iter().filter(|x| &x.function == function_name);
        let notes = graph.notes.iter().filter(|x| &x.function == function_name);

        for fact in facts {
            debug!("Drawing fact");

            str_vars.push_str(&format!(
                "\\node[circle,fill,inner sep=1pt,label=left:{}] ({}) at ({}, {}) {{}};\n",
                fact.belongs_to_var.replace("%", "\\%"),
                fact.id,
                index + fact.track,
                fact.pc,
            ));
        }

        for note in notes {
            debug!("Drawing note");

            str_vars.push_str(&format!(
                "\\node[font=\\tiny] (note_{}) at ({}, {}) {{{}}};\n",
                note.id,
                index as f64 - 1.5,   //x
                note.pc as f64 - 0.5, //y
                note.note.replace("%", "").replace("\"", "").escape_debug(),
            ));
        }

        index += function.definitions + TAU + 1;
    }

    for edge in graph.edges.iter() {
        match edge {
            Edge::Normal { from, to , curved } => {
                if *curved {
                    str_vars.push_str(&format!("\t\t\\path[->, bend right] ({}) edge ({});\n", from.id, to.id));
                }
                else {
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
        }
    }

    template(str_vars)
}

fn template(inject: String) -> String {
    format!(
        "   \\documentclass{{article}}
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
