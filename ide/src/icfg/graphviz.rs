use std::borrow::Cow;
use std::io::Write;

use dot::LabelText;

use crate::icfg::graph::{Edge, Fact, Graph, Variable};

pub fn render_to<W: Write>(graph: &Graph, output: &mut W) {
    dot::render(graph, output).unwrap()
}

impl<'a> dot::Labeller<'a, Fact, Edge> for Graph {
    //TODO name of the graph
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new("example1").unwrap()
    }

    fn node_id(&'a self, n: &Fact) -> dot::Id<'a> {
        dot::Id::new(format!("Fact{}", n.id)).unwrap()
    }

    fn node_label(&'a self, n: &Fact) -> LabelText<'a> {
        dot::LabelText::html(format!("{}", n.note))
    }

    fn edge_color(&'a self, e: &Edge) -> Option<LabelText<'a>> {
        match e {
            Edge::Call { .. } => {
                Some(LabelText::LabelStr(Cow::Borrowed("firebrick1")))
            }
            _ => None,
        }
    }
}

impl<'a> dot::GraphWalk<'a, Fact, Edge> for Graph {
    fn nodes(&self) -> dot::Nodes<'a, Fact> {
        // (assumes that |N| * 2 \approxeq |E|)
        let mut nodes = Vec::with_capacity(self.edges.len() * 2);
        for edge in &self.edges {
            match edge {
                Edge::Normal { from, to } => {
                    nodes.push(from.clone());
                    nodes.push(to.clone());
                }
                Edge::Call { from, to } => {
                    nodes.push(from.clone());
                    nodes.push(to.clone());
                }
                _ => {}
            }
        }
        nodes.sort();
        nodes.dedup();
        Cow::Owned(nodes)
    }

    fn edges(&'a self) -> dot::Edges<'a, Edge> {
        let edges = &self.edges;
        Cow::Borrowed(&edges[..])
    }

    fn source(&self, e: &Edge) -> Fact {
        match e {
            Edge::Normal { from, to: _to } => from.clone(),
            Edge::Call { from, to: _to } => from.clone(),
            _ => unimplemented!("Please add"),
        }
    }

    fn target(&self, e: &Edge) -> Fact {
        match e {
            Edge::Normal { from: _from, to } => to.clone(),
            Edge::Call { from: _from, to } => to.clone(),
            _ => unimplemented!("Please add"),
        }
    }
}
