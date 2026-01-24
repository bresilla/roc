use crate::shared::tf2_subscriber::TfEdgeKind;
use crate::shared::tf2_subscriber::TfEdgeTransform;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize)]
pub struct TfEdgeExport {
    pub parent: String,
    pub child: String,
    pub kind: String,
    pub translation: [f64; 3],
    pub rotation: [f64; 4],
}

#[derive(Debug, Clone, Serialize)]
pub struct TfGraphExport {
    pub frames: Vec<String>,
    pub edges: Vec<TfEdgeExport>,
}

pub fn export_graph(edges: Vec<((String, String), TfEdgeTransform, TfEdgeKind)>) -> TfGraphExport {
    let mut frames = BTreeSet::new();
    let mut out_edges = Vec::new();
    for ((p, c), tf, kind) in edges {
        frames.insert(p.clone());
        frames.insert(c.clone());
        out_edges.push(TfEdgeExport {
            parent: p,
            child: c,
            kind: match kind {
                TfEdgeKind::Static => "static".to_string(),
                TfEdgeKind::Dynamic => "dynamic".to_string(),
            },
            translation: [tf.tx, tf.ty, tf.tz],
            rotation: [tf.qw, tf.qx, tf.qy, tf.qz],
        });
    }

    TfGraphExport {
        frames: frames.into_iter().collect(),
        edges: out_edges,
    }
}

pub fn export_dot(graph: &TfGraphExport) -> String {
    let mut s = String::new();
    s.push_str("digraph tf {\n");
    s.push_str("  rankdir=LR;\n");

    // Nodes
    for f in &graph.frames {
        s.push_str(&format!("  \"{}\";\n", f));
    }

    // Edges
    for e in &graph.edges {
        let label = format!(
            "{} t=[{:.3},{:.3},{:.3}] q=[{:.3},{:.3},{:.3},{:.3}]",
            e.kind,
            e.translation[0],
            e.translation[1],
            e.translation[2],
            e.rotation[0],
            e.rotation[1],
            e.rotation[2],
            e.rotation[3]
        );
        s.push_str(&format!(
            "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
            e.parent, e.child, label
        ));
    }

    s.push_str("}\n");
    s
}

pub fn build_parent_children_map(
    edges: &[TfEdgeExport],
) -> (
    BTreeMap<String, Vec<TfEdgeExport>>,
    BTreeMap<String, Vec<TfEdgeExport>>,
) {
    let mut children: BTreeMap<String, Vec<TfEdgeExport>> = BTreeMap::new();
    let mut parents: BTreeMap<String, Vec<TfEdgeExport>> = BTreeMap::new();

    for e in edges {
        children
            .entry(e.parent.clone())
            .or_default()
            .push(e.clone());
        parents.entry(e.child.clone()).or_default().push(e.clone());
    }

    (parents, children)
}
