/*
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License found in the LICENSE file in the root
 * directory of this source tree.
 */

use crate::graph::{EdgeType, Node, NodeData, NodeType};
use crate::walk::{expand_checked_nodes, OutgoingEdge, ResolvedNode, WalkVisitor};
use chashmap::CHashMap;
use mercurial_types::{HgChangesetId, HgFileNodeId, HgManifestId};
use mononoke_types::{ChangesetId, ContentId, MPath, MPathHash};
use std::{cmp, collections::HashSet, hash::Hash, ops::Add, sync::Arc};

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct StepStats {
    pub num_direct: usize,
    pub num_direct_new: usize,
    pub num_expanded_new: usize,
    pub visited_of_type: usize,
}

impl Add<StepStats> for StepStats {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            num_direct: self.num_direct + other.num_direct,
            num_direct_new: self.num_direct_new + other.num_direct_new,
            num_expanded_new: self.num_expanded_new + other.num_expanded_new,
            visited_of_type: cmp::max(self.visited_of_type, other.visited_of_type),
        }
    }
}

#[derive(Debug)]
struct WalkStateCHashMap {
    // TODO implement ID interning to u32 or u64 for types in more than one map
    // e.g. ChangesetId, HgChangesetId, HgFileNodeId
    include_node_types: HashSet<NodeType>,
    include_edge_types: HashSet<EdgeType>,
    visited_bcs: CHashMap<ChangesetId, ()>,
    visited_bcs_mapping: CHashMap<ChangesetId, ()>,
    visited_file: CHashMap<ContentId, ()>,
    visited_hg_cs: CHashMap<HgChangesetId, ()>,
    visited_hg_cs_mapping: CHashMap<HgChangesetId, ()>,
    visited_hg_file_envelope: CHashMap<HgFileNodeId, ()>,
    visited_hg_filenode: CHashMap<(Option<MPathHash>, HgFileNodeId), ()>,
    visited_hg_manifest: CHashMap<(Option<MPathHash>, HgManifestId), ()>,
    visit_count: CHashMap<NodeType, usize>,
}

/// If the state did not have this value present, true is returned.
fn record_with_path<K>(
    visited_with_path: &CHashMap<(Option<MPathHash>, K), ()>,
    k: &(Option<MPath>, K),
) -> bool
where
    K: Eq + Hash + Copy,
{
    let (path, id) = k;
    let mpathhash_opt = path.as_ref().map(|m| m.get_path_hash());
    !visited_with_path.insert((mpathhash_opt, *id), ()).is_some()
}

impl WalkStateCHashMap {
    fn new(include_node_types: HashSet<NodeType>, include_edge_types: HashSet<EdgeType>) -> Self {
        Self {
            include_node_types,
            include_edge_types,
            visited_bcs: CHashMap::new(),
            visited_bcs_mapping: CHashMap::new(),
            visited_file: CHashMap::new(),
            visited_hg_cs: CHashMap::new(),
            visited_hg_cs_mapping: CHashMap::new(),
            visited_hg_file_envelope: CHashMap::new(),
            visited_hg_filenode: CHashMap::new(),
            visited_hg_manifest: CHashMap::new(),
            visit_count: CHashMap::new(),
        }
    }

    /// If the set did not have this value present, true is returned.
    fn is_first_visit(&self, outgoing: &OutgoingEdge) -> bool {
        let target_node: &Node = &outgoing.target;
        let k = target_node.get_type();
        &self.visit_count.upsert(k, || 1, |old| *old += 1);

        match &target_node {
            Node::BonsaiChangeset(bcs_id) => self.visited_bcs.insert(*bcs_id, ()).is_none(),
            // TODO - measure if worth tracking - the mapping is cachelib enabled.
            Node::BonsaiHgMapping(bcs_id) => self.visited_bcs_mapping.insert(*bcs_id, ()).is_none(),
            Node::HgBonsaiMapping(hg_cs_id) => {
                self.visited_hg_cs_mapping.insert(*hg_cs_id, ()).is_none()
            }
            Node::HgChangeset(hg_cs_id) => self.visited_hg_cs.insert(*hg_cs_id, ()).is_none(),
            Node::HgManifest(k) => record_with_path(&self.visited_hg_manifest, k),
            Node::HgFileNode(k) => record_with_path(&self.visited_hg_filenode, k),
            Node::HgFileEnvelope(id) => self.visited_hg_file_envelope.insert(*id, ()).is_none(),
            Node::FileContent(content_id) => self.visited_file.insert(*content_id, ()).is_none(),
            _ => true,
        }
    }

    fn retain_edge(&self, outgoing_edge: &OutgoingEdge) -> bool {
        // Retain if a root, or if selected
        outgoing_edge.label.incoming_type().is_none()
            || (self
                .include_node_types
                .contains(&outgoing_edge.target.get_type())
                && self.include_edge_types.contains(&outgoing_edge.label))
    }

    fn visit(
        &self,
        // Option as roots have no source
        source: ResolvedNode,
        mut outgoing: Vec<OutgoingEdge>,
    ) -> ((Node, Option<(StepStats, NodeData)>), Vec<OutgoingEdge>) {
        // Filter things we don't want to enter the WalkVisitor at all.
        outgoing.retain(|e| self.retain_edge(e));
        let num_direct = outgoing.len();

        outgoing.retain(|e| self.is_first_visit(&e));
        let num_direct_new = outgoing.len();

        expand_checked_nodes(&mut outgoing);
        // Make sure we don't expand to types of node and edge not wanted
        outgoing.retain(|e| self.retain_edge(e));

        // Stats
        let num_expanded_new = outgoing.len();
        let node = source.node;
        let node_data = source.data;
        let via = source.via;
        let payload = via.map(|_via| {
            let visited_of_type = self.get_visit_count(&node.get_type());
            let stats = StepStats {
                num_direct,
                num_direct_new,
                num_expanded_new,
                visited_of_type,
            };
            (stats, node_data)
        });
        ((node, payload), outgoing)
    }

    fn get_visit_count(&self, t: &NodeType) -> usize {
        self.visit_count.get(t).map(|v| *v).unwrap_or(0)
    }
}

#[derive(Clone, Debug)]
pub struct WalkState {
    inner: Arc<WalkStateCHashMap>,
}

impl WalkState {
    pub fn new(
        include_node_types: HashSet<NodeType>,
        include_edge_types: HashSet<EdgeType>,
    ) -> Self {
        Self {
            inner: Arc::new(WalkStateCHashMap::new(
                include_node_types,
                include_edge_types,
            )),
        }
    }
}

impl WalkVisitor<(Node, Option<(StepStats, NodeData)>)> for WalkState {
    fn visit(
        &self,
        current: ResolvedNode,
        outgoing_edge: Vec<OutgoingEdge>,
    ) -> ((Node, Option<(StepStats, NodeData)>), Vec<OutgoingEdge>) {
        self.inner.visit(current, outgoing_edge)
    }
}
