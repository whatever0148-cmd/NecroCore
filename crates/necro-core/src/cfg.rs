use crate::ir::{BlockId, NecroIR, Terminator};
use std::collections::{BTreeMap, BTreeSet};

/// Control-flow graph over the blocks of a [`NecroIR`] program.
#[derive(Clone, Debug)]
pub struct Cfg {
    pub n: usize,
    pub entry: usize,
    pub edges: Vec<(usize, usize)>,
    pub succ: Vec<Vec<usize>>,
    pub pred: Vec<Vec<usize>>,
}

impl Cfg {
    /// Builds the graph with a one-level PERFORM context: a paragraph end falls
    /// through to the next paragraph only when the paragraph was entered inline
    /// (first paragraph, GO TO, or fallthrough), and returns to a PERFORM's
    /// continuation only when that PERFORM's range ends at this paragraph. Edges
    /// are the projection of the reached (block, context) transitions, so a
    /// paragraph that is only ever PERFORMed does not leak into its successor.
    pub fn build(ir: &NecroIR) -> Cfg {
        const INLINE: usize = usize::MAX;
        let n = ir.blocks.len();
        let mut edges: BTreeSet<(usize, usize)> = BTreeSet::new();
        let mut reached: BTreeSet<(usize, usize)> = BTreeSet::new();
        let mut work: Vec<(usize, usize)> = Vec::new();
        // contexts in which each Perform block was reached
        let mut perform_ctx: BTreeMap<usize, BTreeSet<usize>> = BTreeMap::new();
        // ParaEnd blocks of paragraph p reached in context p (ready to return)
        let mut ends_reached: BTreeMap<usize, BTreeSet<usize>> = BTreeMap::new();
        let mut performs_ending_at: BTreeMap<usize, Vec<(usize, usize)>> = BTreeMap::new(); // end_para -> (perform block, ret)
        let mut call_rets: Vec<usize> = Vec::new();
        for b in &ir.blocks {
            match &b.term {
                Terminator::Perform { end_para, ret, .. } => performs_ending_at.entry(*end_para).or_default().push((b.id.0, ret.0)),
                Terminator::Call { ret, .. } => call_rets.push(ret.0),
                _ => {}
            }
        }
        let entry = ir.entry.0;
        reached.insert((entry, INLINE));
        work.push((entry, INLINE));
        while let Some((b, ctx)) = work.pop() {
            if b >= n {
                continue;
            }
            let mut add = |from: usize, to: usize, c: usize, work: &mut Vec<(usize, usize)>| {
                if to < n {
                    edges.insert((from, to));
                    if reached.insert((to, c)) {
                        work.push((to, c));
                    }
                }
            };
            match &ir.blocks[b].term {
                Terminator::Jump(t) => add(b, t.0, ctx, &mut work),
                Terminator::Branch { then_to, else_to, .. } => {
                    add(b, then_to.0, ctx, &mut work);
                    add(b, else_to.0, ctx, &mut work);
                }
                Terminator::Perform { entry: e, end_para, ret } => {
                    let fresh = perform_ctx.entry(b).or_default().insert(ctx);
                    add(b, e.0, *end_para, &mut work);
                    if fresh {
                        if let Some(ends) = ends_reached.get(end_para) {
                            for &pe in ends.clone().iter() {
                                add(pe, ret.0, ctx, &mut work);
                            }
                        }
                    }
                }
                Terminator::ParaEnd { para, next } => {
                    if ctx == *para {
                        ends_reached.entry(*para).or_default().insert(b);
                        if let Some(list) = performs_ending_at.get(para) {
                            for &(pb, ret) in list {
                                if let Some(ctxs) = perform_ctx.get(&pb) {
                                    for &c in ctxs.clone().iter() {
                                        add(b, ret, c, &mut work);
                                    }
                                }
                            }
                        }
                    } else {
                        add(b, next.0, ctx, &mut work);
                    }
                }
                Terminator::Call { target, ret } => {
                    add(b, target.0, ctx, &mut work);
                    add(b, ret.0, ctx, &mut work);
                }
                Terminator::Return => {
                    for &r in &call_rets {
                        add(b, r, ctx, &mut work);
                    }
                }
                Terminator::Halt => {}
            }
        }
        let edges: Vec<(usize, usize)> = edges.into_iter().collect();
        let mut succ = vec![Vec::new(); n];
        let mut pred = vec![Vec::new(); n];
        for &(a, b) in &edges {
            succ[a].push(b);
            pred[b].push(a);
        }
        Cfg { n, entry, edges, succ, pred }
    }

    /// Blocks reachable from the entry.
    pub fn reachable(&self) -> Vec<bool> {
        let mut seen = vec![false; self.n];
        let mut stack = vec![self.entry];
        while let Some(x) = stack.pop() {
            if seen[x] {
                continue;
            }
            seen[x] = true;
            for &s in &self.succ[x] {
                if !seen[s] {
                    stack.push(s);
                }
            }
        }
        seen
    }

    /// Tarjan strongly connected components. Only components that contain a
    /// cycle (size > 1 or a self edge) are returned: those are the loops.
    pub fn loops(&self) -> Vec<Vec<usize>> {
        let n = self.n;
        let mut index = vec![usize::MAX; n];
        let mut low = vec![0usize; n];
        let mut on = vec![false; n];
        let mut stack: Vec<usize> = Vec::new();
        let mut out: Vec<Vec<usize>> = Vec::new();
        let mut counter = 0usize;
        // iterative Tarjan
        for root in 0..n {
            if index[root] != usize::MAX {
                continue;
            }
            let mut work: Vec<(usize, usize)> = vec![(root, 0)];
            index[root] = counter;
            low[root] = counter;
            counter += 1;
            stack.push(root);
            on[root] = true;
            while let Some(&mut (v, ref mut i)) = work.last_mut() {
                if *i < self.succ[v].len() {
                    let w = self.succ[v][*i];
                    *i += 1;
                    if index[w] == usize::MAX {
                        index[w] = counter;
                        low[w] = counter;
                        counter += 1;
                        stack.push(w);
                        on[w] = true;
                        work.push((w, 0));
                    } else if on[w] {
                        low[v] = low[v].min(index[w]);
                    }
                } else {
                    work.pop();
                    if let Some(&(u, _)) = work.last() {
                        low[u] = low[u].min(low[v]);
                    }
                    if low[v] == index[v] {
                        let mut comp = Vec::new();
                        loop {
                            let w = stack.pop().unwrap();
                            on[w] = false;
                            comp.push(w);
                            if w == v {
                                break;
                            }
                        }
                        comp.sort_unstable();
                        let cyclic = comp.len() > 1 || self.succ[v].contains(&v);
                        if cyclic {
                            out.push(comp);
                        }
                    }
                }
            }
        }
        out.sort();
        out
    }

    pub fn edges_named(&self, ir: &NecroIR) -> Vec<(String, String)> {
        self.edges
            .iter()
            .map(|&(a, b)| (ir.blocks[a].label.clone(), ir.blocks[b].label.clone()))
            .collect()
    }

    pub fn block_id(i: usize) -> BlockId {
        BlockId(i)
    }
}
