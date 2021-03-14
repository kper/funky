use crate::icfg::graph::{Edge, Fact, Graph};
use log::debug;

pub fn render_to(graph: &Graph) {
    let mut str_vars = String::new();

    let mut index = 0;

    /*
    for (function_name, vars) in graph.vars.iter() {
        for var in vars.iter() {
            str_vars.push_str(&format!(
                "\\node[circle,fill,inner sep=1pt] ({}_{}) at ({}, 0) {{}};\n",
                function_name, var.id.replace("%", ""), index
            ));
            index += 1;
        }
    }*/

    let max_epoch = graph.get_max_epoch().expect("No max epoch");

    for epoch in 0..=max_epoch {
        let str_epoch = draw_epoch(graph, epoch, max_epoch);
        str_vars.push_str(&str_epoch);
    }

    for edge in graph.edges.iter() {
        match edge {
            Edge::Normal { from, to} => {
                str_vars.push_str(&format!("\t\t\\path[->] ({}) edge ({});\n", from.id, to.id));
            }
            /*Edge::Call { from, to } => {
                str_vars.push_str(&format!("\t\t\\path[->] ({}) edge node[above] {{ call }} ({});\n", from.id, to.id));
            }*/
            _ => {}
        } 
    }

    println!("{}", template(str_vars));
}

fn draw_epoch(graph: &Graph, epoch: usize, max: usize) -> String {
    debug!("Drawing epoch {}", epoch);
    let mut str_epoch = String::new();
    let mut index = 0;

    for fact in graph.facts.iter().filter(|x| x.epoch == epoch) {
        debug!("Drawing fact {:?}", fact);
        if epoch != 0 {
            str_epoch.push_str(&format!(
                "\t\t\t\\node[circle,fill,inner sep=1pt] ({}) at ({}, {}) {{}};\n",
                fact.id, index, max - epoch
            ));
        } else {
            str_epoch.push_str(&format!(
                "\\node[circle,fill,inner sep=1pt,label={}] ({}) at ({}, {}) {{}};\n",
                fact.belongs_to_var.replace("%", "\\%"), fact.id, index, max - epoch
            ));
        }

        index += 1;
    }

    str_epoch
}

fn template(inject: String) -> String {
    format!(
        "   \\documentclass{{standalone}}
            \\usepackage{{pgf, tikz}}
            \\usetikzlibrary{{arrows, automata}}
            \\begin{{document}}
                \\begin{{tikzpicture}}
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
