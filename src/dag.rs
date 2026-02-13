use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

#[derive(PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
pub struct DeviceId(u32);

#[derive(PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
pub enum WireType {
    Normal,
    Negated,
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
pub struct Wire {
    pub from: DeviceId,
    pub to: DeviceId,
    pub wire_type: WireType,
}

pub struct IllegalWireError;

#[derive(Clone, Serialize, Deserialize)]
pub struct Dag {
    id_counter: u32,
    wires: Vec<Wire>,
    topological_order: Vec<DeviceId>,
    transitive_closure: Vec<(DeviceId, DeviceId)>,
}

impl Dag {
    pub fn new() -> Self {
        Dag {
            id_counter: 0,
            wires: Vec::new(),
            transitive_closure: Vec::new(),
            topological_order: Vec::new(),
        }
    }

    pub fn devices(&self) -> impl Iterator<Item = &DeviceId> {
        self.topological_order.iter()
    }

    pub fn wires(&self) -> impl Iterator<Item = &Wire> {
        self.wires.iter()
    }

    pub fn incoming(&self, child: DeviceId) -> impl Iterator<Item = &Wire> {
        self.wires
            .iter()
            .filter_map(move |wire| if wire.to == child { Some(wire) } else { None })
    }

    pub fn contains_device(&self, d: DeviceId) -> bool {
        self.topological_order.contains(&d)
    }

    pub fn contains_wire(&self, from: DeviceId, to: DeviceId) -> bool {
        for wire in self.wires.iter() {
            if wire.from == from && wire.to == to {
                return true;
            }
        }
        false
    }

    pub fn is_reachable(&self, from: DeviceId, to: DeviceId) -> bool {
        for (tc_from, tc_to) in self.transitive_closure.iter() {
            if *tc_from == from && *tc_to == to {
                return true;
            }
        }
        false
    }

    pub fn add_device(&mut self) -> DeviceId {
        let id = DeviceId(self.id_counter);
        self.id_counter += 1;

        // a new vertex can go anywhere in the topo order
        self.topological_order.push(id);

        id
    }

    pub fn add_wire(
        &mut self,
        from: DeviceId,
        to: DeviceId,
        edge_type: WireType,
    ) -> Result<(), IllegalWireError> {
        if self.contains_wire(from, to) {
            Ok(())
        } else if self.is_reachable(to, from) {
            // edge would create cycle
            Err(IllegalWireError)
        } else if self.contains_device(from) && self.contains_device(to) {
            let e = Wire {
                from,
                to,
                wire_type: edge_type,
            };
            self.wires.push(e);
            self.recompute_caches();
            Ok(())
        } else {
            // edge includes nonexistent vertices
            Err(IllegalWireError)
        }
    }

    pub fn remove_device(&mut self, d: DeviceId) {
        self.wires.retain(|w| w.from != d && w.to != d);
        self.topological_order.retain(|x| *x != d);

        self.recompute_caches();
    }

    pub fn remove_wire(&mut self, from: DeviceId, to: DeviceId) {
        self.wires.retain(|x| x.from != from || x.to != to);

        self.recompute_caches();
    }

    /// The Dag keeps a cached copy of its own topological ordering and
    /// transitive closure because these structures are accessed way more often
    /// than the graph is modified.
    ///
    /// This function recomputes those cached structures and should be called
    /// whenever the graph topology changes.
    fn recompute_caches(&mut self) {
        self.recompute_topological_order();
        self.recompute_transitive_closure();
    }

    fn recompute_transitive_closure(&mut self) {
        // initialize closure as adjacency map
        let mut closure = HashMap::new();
        for v in self.topological_order.iter() {
            closure.insert(*v, HashSet::new());
        }

        // iterate backwards and accumulate closures from children to parents
        for v in self.topological_order.iter().rev() {
            closure.get_mut(v).unwrap().insert(*v);
            let v_reachable = closure[v].clone();

            for edge in self.wires.iter() {
                if *v == edge.to {
                    closure
                        .get_mut(&edge.from)
                        .unwrap()
                        .extend(v_reachable.clone());
                }
            }
        }

        // convert from adjacency map to edge list
        self.transitive_closure = closure
            .iter()
            .flat_map(|(v, ws)| ws.iter().map(|w| (*v, *w)))
            .collect();
    }

    fn recompute_topological_order(&mut self) {
        // count incoming edges to each vertex
        let mut incoming = HashMap::new();
        for vid in self.topological_order.iter() {
            incoming.insert(vid.to_owned(), 0);
        }
        for edge in self.wires.iter() {
            *incoming.get_mut(&edge.to).unwrap() += 1;
        }

        // maintain set of all vertices with no incoming edges (source vertices)
        let mut sources = HashSet::new();
        for (vid, count) in incoming.iter() {
            if *count == 0 {
                sources.insert(vid.to_owned());
            }
        }

        let mut topo = Vec::with_capacity(self.topological_order.len());
        while !sources.is_empty() {
            // pop some vertex with no incoming edges
            let v = sources.iter().next().unwrap().clone();
            sources.remove(&v);

            // make it next in the topological ordering
            topo.push(v);

            // remove outgoing edges from that vertex, and add any new sources
            for edge in self.wires.iter() {
                if edge.from == v {
                    *incoming.get_mut(&edge.to).unwrap() -= 1;

                    if incoming[&edge.to] == 0 {
                        sources.insert(edge.to);
                    }
                }
            }
        }

        self.topological_order = topo;
    }
}
