use crate::icfg::graph2::{Edge, Fact, Graph};
use log::debug;

pub const TAU: usize = 1;

pub fn render_to(graph: &Graph) {
    let mut str_vars = String::new();

    //let max_pc = graph.facts().expect("No max epoch");

    /*
    for epoch in 0..=max_epoch {
        let str_epoch = draw_epoch(graph, epoch, max_epoch);
        str_vars.push_str(&str_epoch);
    }*/

    let mut index = 0;
    for (function_name, function) in graph.functions.iter() {
        debug!("Drawing function {}", function_name);

        let facts = graph.facts.iter().filter(|x| &x.function == function_name);
        let max_pc = facts.clone().map(|x| x.pc).max().unwrap_or(0);

        for fact in facts {
            debug!("Drawing fact");

            str_vars.push_str(&format!(
                "\\node[circle,fill,inner sep=1pt,label=left:{}] ({}) at ({}, {}) {{}};\n",
                fact.belongs_to_var.replace("%", "\\%"),
                fact.id,
                index + fact.track,
                max_pc - fact.pc,
            ));
        }

        index += function.definitions + TAU;        
    }

    /*
    for edge in graph.edges.iter() {
        match edge {
            Edge::Normal { from, to } => {
                str_vars.push_str(&format!("\t\t\\path[->] ({}) edge ({});\n", from.id, to.id));
            }
            Edge::Call { from, to } => {
                str_vars.push_str(&format!("\t\t\\path[->, green] ({}) [bend left] edge node {{ }} ({});\n", from.id, to.id));
            }
            Edge::CallToReturn { from, to } => {
                str_vars.push_str(&format!("\t\t\\path[->] ({}) edge node {{ }} ({});\n", from.id, to.id));
            }
            Edge::Return { from, to } => {
                str_vars.push_str(&format!("\t\t\\path[->, red] ({}) [bend right] edge  node {{ }} ({});\n", from.id, to.id));
            }

            _ => {}
        }
    }
    */

    println!("{}", template(str_vars));
}

fn draw_epoch(graph: &Graph, epoch: usize, max: usize) -> String {
    debug!("Drawing epoch {}", epoch);

    let mut str_epoch = String::new();
    let mut index = 0;
    /*

    let mut last_note = String::new();
    for fact in graph.facts.iter().filter(|x| x.epoch == epoch) {
        debug!("Drawing fact {:?}", fact);

        str_epoch.push_str(&format!(
            "\\node[circle,fill,inner sep=1pt,label=left:{}] ({}) at ({}, {}) {{}};\n",
            fact.belongs_to_var.replace("%", "\\%"),
            //fact.note.replace("\"", "\\\"").replace("%", "\\%"),
            fact.id,
            fact.scope + index,
            max - epoch,
        ));

        if last_note != fact.note {
            str_epoch.push_str(&format!(
                "\\node[circle, font=\\tiny] at ({}, {}) {{{}}};\n",
                //fact.belongs_to_var.replace("%", "\\%"),
                //fact.id,
                (fact.scope + index) as f64 - 1.5,
                (max - epoch) as f64 - 0.5,
                fact.note.replace("\"", "\\\"").replace("%", "\\%"),
            ));
            last_note = fact.note.clone();
        }

        index += 1;
    }
    */

    str_epoch
}

fn template(inject: String) -> String {
    format!(
        "   \\documentclass{{article}}
            \\usepackage{{pgf, tikz}}
            \\usetikzlibrary{{arrows, automata}}
            \\begin{{document}}
                \\begin{{tikzpicture}}
                    [node distance=2cm]
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
