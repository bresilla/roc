use crate::shared::tf2_subscriber::{TfEdgeKind, TfEdgeTransform};
use std::collections::{BTreeMap, VecDeque};

#[derive(Debug, Clone, Copy)]
pub struct TfTransform {
    pub tx: f64,
    pub ty: f64,
    pub tz: f64,
    pub qx: f64,
    pub qy: f64,
    pub qz: f64,
    pub qw: f64,
}

impl TfTransform {
    pub fn identity() -> Self {
        Self {
            tx: 0.0,
            ty: 0.0,
            tz: 0.0,
            qx: 0.0,
            qy: 0.0,
            qz: 0.0,
            qw: 1.0,
        }
    }
}

fn quat_mul(a: (f64, f64, f64, f64), b: (f64, f64, f64, f64)) -> (f64, f64, f64, f64) {
    let (ax, ay, az, aw) = a;
    let (bx, by, bz, bw) = b;
    (
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    )
}

fn quat_conj(q: (f64, f64, f64, f64)) -> (f64, f64, f64, f64) {
    (-q.0, -q.1, -q.2, q.3)
}

fn quat_rotate(q: (f64, f64, f64, f64), v: (f64, f64, f64)) -> (f64, f64, f64) {
    // v' = q * (v,0) * q_conj
    let (vx, vy, vz) = v;
    let vq = (vx, vy, vz, 0.0);
    let t = quat_mul(q, vq);
    let r = quat_mul(t, quat_conj(q));
    (r.0, r.1, r.2)
}

fn normalize_quat(q: (f64, f64, f64, f64)) -> (f64, f64, f64, f64) {
    let n = (q.0 * q.0 + q.1 * q.1 + q.2 * q.2 + q.3 * q.3).sqrt();
    if n == 0.0 {
        return (0.0, 0.0, 0.0, 1.0);
    }
    (q.0 / n, q.1 / n, q.2 / n, q.3 / n)
}

fn compose(a: TfTransform, b: TfTransform) -> TfTransform {
    // a_then_b: p' = Ra * p + ta, then b: p'' = Rb * p' + tb
    // => p'' = (Rb*Ra)p + (Rb*ta + tb)
    let qa = (a.qx, a.qy, a.qz, a.qw);
    let qb = (b.qx, b.qy, b.qz, b.qw);
    let q = normalize_quat(quat_mul(qb, qa));
    let ta_rot = quat_rotate(qb, (a.tx, a.ty, a.tz));
    TfTransform {
        tx: ta_rot.0 + b.tx,
        ty: ta_rot.1 + b.ty,
        tz: ta_rot.2 + b.tz,
        qx: q.0,
        qy: q.1,
        qz: q.2,
        qw: q.3,
    }
}

fn invert(t: TfTransform) -> TfTransform {
    let q = normalize_quat((t.qx, t.qy, t.qz, t.qw));
    let q_inv = quat_conj(q);
    let trans_inv = quat_rotate(q_inv, (-t.tx, -t.ty, -t.tz));
    TfTransform {
        tx: trans_inv.0,
        ty: trans_inv.1,
        tz: trans_inv.2,
        qx: q_inv.0,
        qy: q_inv.1,
        qz: q_inv.2,
        qw: q_inv.3,
    }
}

#[derive(Debug, Clone)]
pub struct TfGraph {
    // Directed adjacency: from -> Vec<(to, tf, kind)>
    adj: BTreeMap<String, Vec<(String, TfTransform, TfEdgeKind)>>,
}

impl TfGraph {
    pub fn from_edges(edges: Vec<((String, String), TfEdgeTransform, TfEdgeKind)>) -> Self {
        let mut adj: BTreeMap<String, Vec<(String, TfTransform, TfEdgeKind)>> = BTreeMap::new();
        for ((parent, child), tf, kind) in edges {
            let t = TfTransform {
                tx: tf.tx,
                ty: tf.ty,
                tz: tf.tz,
                qx: tf.qx,
                qy: tf.qy,
                qz: tf.qz,
                qw: tf.qw,
            };
            adj.entry(parent.clone())
                .or_default()
                .push((child.clone(), t, kind));
            // Add inverse edge so lookups can traverse both ways.
            adj.entry(child)
                .or_default()
                .push((parent, invert(t), kind));
        }
        Self { adj }
    }

    pub fn lookup(&self, from: &str, to: &str) -> Option<(TfTransform, TfEdgeKind)> {
        if from == to {
            return Some((TfTransform::identity(), TfEdgeKind::Static));
        }

        // BFS to find shortest path.
        let mut q = VecDeque::new();
        let mut prev: BTreeMap<String, (String, TfTransform, TfEdgeKind)> = BTreeMap::new();
        q.push_back(from.to_string());
        prev.insert(
            from.to_string(),
            (String::new(), TfTransform::identity(), TfEdgeKind::Static),
        );

        while let Some(cur) = q.pop_front() {
            let Some(neigh) = self.adj.get(&cur) else {
                continue;
            };
            for (next, t_edge, kind) in neigh {
                if prev.contains_key(next) {
                    continue;
                }
                prev.insert(next.clone(), (cur.clone(), *t_edge, *kind));
                if next == to {
                    q.clear();
                    break;
                }
                q.push_back(next.clone());
            }
        }

        if !prev.contains_key(to) {
            return None;
        }

        // Reconstruct and compose transforms.
        let mut cur = to.to_string();
        let mut composed = TfTransform::identity();
        let mut overall_kind = TfEdgeKind::Static;
        while cur != from {
            let (p, t_edge, kind) = prev.get(&cur).unwrap().clone();
            composed = compose(t_edge, composed);
            if kind == TfEdgeKind::Dynamic {
                overall_kind = TfEdgeKind::Dynamic;
            }
            cur = p;
        }

        Some((composed, overall_kind))
    }
}
