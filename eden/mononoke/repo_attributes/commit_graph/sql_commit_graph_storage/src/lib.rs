/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

//! SQL Commit Graph Storage
//!
//! Database-backed implementation of the commit graph storage.
#![allow(unused)]

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use commit_graph_types::edges::ChangesetEdges;
use commit_graph_types::edges::ChangesetNode;
use commit_graph_types::edges::ChangesetNodeParents;
use commit_graph_types::storage::CommitGraphStorage;
use commit_graph_types::storage::Prefetch;
use commit_graph_types::ChangesetParents;
use context::CoreContext;
use context::PerfCounterType;
use mononoke_types::ChangesetId;
use mononoke_types::ChangesetIdPrefix;
use mononoke_types::ChangesetIdsResolvedFromPrefix;
use mononoke_types::Generation;
use mononoke_types::RepositoryId;
use rendezvous::RendezVous;
use rendezvous::RendezVousOptions;
use rendezvous::RendezVousStats;
use rendezvous::TunablesRendezVousController;
use sql::Connection;
use sql::SqlConnections;
use sql_construct::SqlConstruct;
use sql_construct::SqlConstructFromMetadataDatabaseConfig;
use sql_ext::mononoke_queries;
use vec1::vec1;
use vec1::Vec1;

#[cfg(test)]
mod tests;

pub struct SqlCommitGraphStorageBuilder {
    connections: SqlConnections,
}

impl SqlConstruct for SqlCommitGraphStorageBuilder {
    const LABEL: &'static str = "commit_graph";

    const CREATION_QUERY: &'static str = include_str!("../schemas/sqlite-commit-graph.sql");

    fn from_sql_connections(connections: SqlConnections) -> Self {
        Self { connections }
    }
}

impl SqlConstructFromMetadataDatabaseConfig for SqlCommitGraphStorageBuilder {}

impl SqlCommitGraphStorageBuilder {
    pub fn build(self, opts: RendezVousOptions, repo_id: RepositoryId) -> SqlCommitGraphStorage {
        SqlCommitGraphStorage {
            repo_id,
            read_connection: RendezVousConnection::new(
                self.connections.read_connection,
                "read",
                opts,
            ),
            read_master_connection: RendezVousConnection::new(
                self.connections.read_master_connection,
                "read_master",
                opts,
            ),
            write_connection: self.connections.write_connection,
        }
    }
}

#[derive(Clone)]
struct RendezVousConnection {
    fetch_single: RendezVous<ChangesetId, ChangesetEdges>,
    conn: Connection,
}

impl RendezVousConnection {
    fn new(conn: Connection, name: &str, opts: RendezVousOptions) -> Self {
        Self {
            conn,
            fetch_single: RendezVous::new(
                TunablesRendezVousController::new(opts),
                Arc::new(RendezVousStats::new(format!(
                    "commit_graph.fetch_single.{}",
                    name
                ))),
            ),
        }
    }
}

pub struct SqlCommitGraphStorage {
    repo_id: RepositoryId,
    write_connection: Connection,
    read_connection: RendezVousConnection,
    read_master_connection: RendezVousConnection,
}

mononoke_queries! {
    write InsertChangeset(
        repo_id: RepositoryId,
        cs_id: ChangesetId,
        gen: u64,
        skip_tree_depth: u64,
        p1_linear_depth: u64,
        parent_count: usize,
        p1_parent: Option<ChangesetId>,
        merge_ancestor: Option<ChangesetId>,
        skip_tree_parent: Option<ChangesetId>,
        skip_tree_skew_ancestor: Option<ChangesetId>,
        p1_linear_skew_ancestor: Option<ChangesetId>
    ) {
        insert_or_ignore,
        "
        {insert_or_ignore} INTO commit_graph_edges (
            repo_id,
            cs_id,
            gen,
            skip_tree_depth,
            p1_linear_depth,
            parent_count,
            p1_parent,
            merge_ancestor,
            skip_tree_parent,
            skip_tree_skew_ancestor,
            p1_linear_skew_ancestor
        ) VALUES (
            {repo_id},
            {cs_id},
            {gen},
            {skip_tree_depth},
            {p1_linear_depth},
            {parent_count},
            (SELECT cs.id FROM commit_graph_edges cs WHERE cs.repo_id = {repo_id} AND cs.cs_id = {p1_parent}),
            (SELECT cs.id FROM commit_graph_edges cs WHERE cs.repo_id = {repo_id} AND cs.cs_id = {merge_ancestor}),
            (SELECT cs.id FROM commit_graph_edges cs WHERE cs.repo_id = {repo_id} AND cs.cs_id = {skip_tree_parent}),
            (SELECT cs.id FROM commit_graph_edges cs WHERE cs.repo_id = {repo_id} AND cs.cs_id = {skip_tree_skew_ancestor}),
            (SELECT cs.id FROM commit_graph_edges cs WHERE cs.repo_id = {repo_id} AND cs.cs_id = {p1_linear_skew_ancestor})
        )
        "
    }

    write InsertChangesetsNoEdges(values: (
        repo_id: RepositoryId,
        cs_id: ChangesetId,
        gen: u64,
        skip_tree_depth: u64,
        p1_linear_depth: u64,
        parent_count: usize,
    )) {
        insert_or_ignore,
        "
        {insert_or_ignore} INTO commit_graph_edges (
            repo_id,
            cs_id,
            gen,
            skip_tree_depth,
            p1_linear_depth,
            parent_count
        ) VALUES {values}
        "
    }

    // Fix edges for changesets previously added with InsertChangesetsNoEdges
    write FixEdges(values: (
        repo_id: RepositoryId,
        cs_id: ChangesetId,
        // We need the depths otherwise we get an error on sqlite. Though this won't be used because we
        // always replace the edges only.
        gen: u64,
        skip_tree_depth: u64,
        p1_linear_depth: u64,
        parent_count: usize,
        p1_parent: Option<u64>,
        merge_ancestor: Option<u64>,
        skip_tree_parent: Option<u64>,
        skip_tree_skew_ancestor: Option<u64>,
        p1_linear_skew_ancestor: Option<u64>
    )) {
        none,
        mysql("INSERT INTO commit_graph_edges
            (repo_id, cs_id, gen, skip_tree_depth, p1_linear_depth, parent_count,
                p1_parent, merge_ancestor, skip_tree_parent, skip_tree_skew_ancestor, p1_linear_skew_ancestor)
        VALUES {values}
        ON DUPLICATE KEY UPDATE
            p1_parent = VALUES(p1_parent),
            merge_ancestor = VALUES(merge_ancestor),
            skip_tree_parent = VALUES(skip_tree_parent),
            skip_tree_skew_ancestor = VALUES(skip_tree_skew_ancestor),
            p1_linear_skew_ancestor = VALUES(p1_linear_skew_ancestor)")
        sqlite("INSERT INTO commit_graph_edges
            (repo_id, cs_id, gen, skip_tree_depth, p1_linear_depth, parent_count,
                p1_parent, merge_ancestor, skip_tree_parent, skip_tree_skew_ancestor, p1_linear_skew_ancestor)
        VALUES {values}
        ON CONFLICT(repo_id, cs_id) DO UPDATE SET
            p1_parent = excluded.p1_parent,
            merge_ancestor = excluded.merge_ancestor,
            skip_tree_parent = excluded.skip_tree_parent,
            skip_tree_skew_ancestor = excluded.skip_tree_skew_ancestor,
            p1_linear_skew_ancestor = excluded.p1_linear_skew_ancestor")
    }

    read SelectManyIds(repo_id: RepositoryId, >list cs_ids: ChangesetId) -> (ChangesetId, u64) {
        "SELECT cs.cs_id, cs.id FROM commit_graph_edges cs WHERE cs.repo_id = {repo_id} AND cs.cs_id IN {cs_ids}"
    }

    write InsertMergeParents(values: (id: u64, parent_num: usize, parent: u64)) {
        insert_or_ignore,
        "{insert_or_ignore} INTO commit_graph_merge_parents (id, parent_num, parent) VALUES {values}"
    }

    read SelectManyChangesets(repo_id: RepositoryId, >list cs_ids: ChangesetId) -> (
        ChangesetId, // cs_id
        Option<u64>, // gen
        Option<u64>, // skip_tree_depth
        Option<u64>, // p1_linear_depth
        Option<usize>, // parent_count
        Option<ChangesetId>, // merge_ancestor
        Option<u64>, // merge_ancestor_gen
        Option<u64>, // merge_ancestor_skip_tree_depth
        Option<u64>, // merge_ancestor_p1_linear_depth
        Option<ChangesetId>, // skip_tree_parent
        Option<u64>, // skip_tree_parent_gen
        Option<u64>, // skip_tree_parent_skip_tree_depth
        Option<u64>, // skip_tree_parent_p1_linear_depth
        Option<ChangesetId>, // skip_tree_skew_ancestor
        Option<u64>, // skip_tree_skew_ancestor_gen
        Option<u64>, // skip_tree_skew_ancestor_skip_tree_depth
        Option<u64>, // skip_tree_skew_ancestor_p1_linear_depth
        Option<ChangesetId>, // p1_linear_skew_ancestor
        Option<u64>, // p1_linear_skew_ancestor_gen
        Option<u64>, // p1_linear_skew_ancestor_skip_tree_depth
        Option<u64>, // p1_linear_skew_ancestor_p1_linear_depth
        usize, // parent_num
        Option<ChangesetId>, // parent
        Option<u64>, // parent_gen
        Option<u64>, // parent_skip_tree_depth
        Option<u64>, // parent_p1_linear_depth
    ) {
        "
        SELECT
            cs0.cs_id AS cs_id,
            NULL AS gen,
            NULL AS skip_tree_depth,
            NULL AS p1_linear_depth,
            NULL AS parent_count,
            NULL AS merge_ancestor,
            NULL AS merge_ancestor_gen,
            NULL AS merge_ancestor_skip_tree_depth,
            NULL AS merge_ancestor_p1_linear_depth,
            NULL AS skip_tree_parent,
            NULL AS skip_tree_parent_gen,
            NULL AS skip_tree_parent_skip_tree_depth,
            NULL AS skip_tree_parent_p1_linear_depth,
            NULL AS skip_tree_skew_ancestor,
            NULL AS skip_tree_skew_ancestor_gen,
            NULL AS skip_tree_skew_ancestor_skip_tree_depth,
            NULL AS skip_tree_skew_ancestor_p1_linear_depth,
            NULL AS p1_linear_skew_ancestor,
            NULL AS p1_linear_skew_ancestor_gen,
            NULL AS p1_linear_skew_ancestor_skip_tree_depth,
            NULL AS p1_linear_skew_ancestor_p1_linear_depth,
            commit_graph_merge_parents.parent_num AS parent_num,
            cs1.cs_id AS parent,
            cs1.gen AS parent_gen,
            cs1.skip_tree_depth AS parent_skip_tree_depth,
            cs1.p1_linear_depth AS parent_p1_linear_depth
        FROM commit_graph_merge_parents
        INNER JOIN commit_graph_edges cs0 ON cs0.id = commit_graph_merge_parents.id
        INNER JOIN commit_graph_edges cs1 ON cs1.id = commit_graph_merge_parents.parent
        WHERE cs0.repo_id = {repo_id} AND cs0.cs_id IN {cs_ids} AND cs1.repo_id = {repo_id} AND cs0.parent_count >= 2

        UNION

        SELECT
            cs0.cs_id AS cs_id,
            cs0.gen AS gen,
            cs0.skip_tree_depth AS skip_tree_depth,
            cs0.p1_linear_depth AS p1_linear_depth,
            cs0.parent_count AS parent_count,
            cs_merge_ancestor.cs_id AS merge_ancestor,
            cs_merge_ancestor.gen AS merge_ancestor_gen,
            cs_merge_ancestor.skip_tree_depth AS merge_ancestor_skip_tree_depth,
            cs_merge_ancestor.p1_linear_depth AS merge_ancestor_p1_linear_depth,
            cs_skip_tree_parent.cs_id AS skip_tree_parent,
            cs_skip_tree_parent.gen AS skip_tree_parent_gen,
            cs_skip_tree_parent.skip_tree_depth AS skip_tree_parent_skip_tree_depth,
            cs_skip_tree_parent.p1_linear_depth AS skip_tree_parent_p1_linear_depth,
            cs_skip_tree_skew_ancestor.cs_id AS skip_tree_skew_ancestor,
            cs_skip_tree_skew_ancestor.gen AS skip_tree_skew_ancestor_gen,
            cs_skip_tree_skew_ancestor.skip_tree_depth AS skip_tree_skew_ancestor_skip_tree_depth,
            cs_skip_tree_skew_ancestor.p1_linear_depth AS skip_tree_skew_ancestor_p1_linear_depth,
            cs_p1_linear_skew_ancestor.cs_id AS p1_linear_skew_ancestor,
            cs_p1_linear_skew_ancestor.gen AS p1_linear_skew_ancestor_gen,
            cs_p1_linear_skew_ancestor.skip_tree_depth AS p1_linear_skew_ancestor_skip_tree_depth,
            cs_p1_linear_skew_ancestor.p1_linear_depth AS p1_linear_skew_ancestor_p1_linear_depth,
            0 AS parent_num,
            cs_p1_parent.cs_id AS parent,
            cs_p1_parent.gen AS parent_gen,
            cs_p1_parent.skip_tree_depth AS parent_skip_tree_depth,
            cs_p1_parent.p1_linear_depth AS parent_p1_linear_depth
        FROM commit_graph_edges cs0
        LEFT JOIN commit_graph_edges cs_p1_parent ON cs_p1_parent.id = cs0.p1_parent
        LEFT JOIN commit_graph_edges cs_merge_ancestor ON cs_merge_ancestor.id = cs0.merge_ancestor
        LEFT JOIN commit_graph_edges cs_skip_tree_parent ON cs_skip_tree_parent.id = cs0.skip_tree_parent
        LEFT JOIN commit_graph_edges cs_skip_tree_skew_ancestor ON cs_skip_tree_skew_ancestor.id = cs0.skip_tree_skew_ancestor
        LEFT JOIN commit_graph_edges cs_p1_linear_skew_ancestor ON cs_p1_linear_skew_ancestor.id = cs0.p1_linear_skew_ancestor
        WHERE cs0.repo_id = {repo_id} and cs0.cs_id IN {cs_ids}

        ORDER BY parent_num ASC
        "
    }

    read SelectChangesetsInRange(repo_id: RepositoryId, min_id: ChangesetId, max_id: ChangesetId, limit: usize) -> (ChangesetId) {
        "
        SELECT cs_id
        FROM commit_graph_edges
        WHERE repo_id = {repo_id} AND {min_id} <= cs_id AND cs_id <= {max_id}
        ORDER BY cs_id ASC
        LIMIT {limit}
        "
    }
}

impl SqlCommitGraphStorage {
    fn collect_changeset_edges(
        fetched_edges: &[(
            ChangesetId,         // cs_id
            Option<u64>,         // gen
            Option<u64>,         // skip_tree_depth
            Option<u64>,         // p1_linear_depth
            Option<usize>,       // parent_count
            Option<ChangesetId>, // merge_ancestor
            Option<u64>,         // merge_ancestor_gen
            Option<u64>,         // merge_ancestor_skip_tree_depth
            Option<u64>,         // merge_ancestor_p1_linear_depth
            Option<ChangesetId>, // skip_tree_parent
            Option<u64>,         // skip_tree_parent_gen
            Option<u64>,         // skip_tree_parent_skip_tree_depth
            Option<u64>,         // skip_tree_parent_p1_linear_depth
            Option<ChangesetId>, // skip_tree_skew_ancestor
            Option<u64>,         // skip_tree_skew_ancestor_gen
            Option<u64>,         // skip_tree_skew_ancestor_skip_tree_depth
            Option<u64>,         // skip_tree_skew_ancestor_p1_linear_depth
            Option<ChangesetId>, // p1_linear_skew_ancestor
            Option<u64>,         // p1_linear_skew_ancestor_gen
            Option<u64>,         // p1_linear_skew_ancestor_skip_tree_depth
            Option<u64>,         // p1_linear_skew_ancestor_p1_linear_depth
            usize,               // parent_num
            Option<ChangesetId>, // parent
            Option<u64>,         // parent_gen
            Option<u64>,         // parent_skip_tree_depth
            Option<u64>,         // parent_p1_linear_depth
        )],
    ) -> HashMap<ChangesetId, ChangesetEdges> {
        let option_fields_to_option_node =
            |cs_id, generation, skip_tree_depth, p1_linear_depth| match (
                cs_id,
                generation,
                skip_tree_depth,
                p1_linear_depth,
            ) {
                (Some(cs_id), Some(generation), Some(skip_tree_depth), Some(p1_linear_depth)) => {
                    Some(ChangesetNode {
                        cs_id,
                        generation: Generation::new(generation),
                        skip_tree_depth,
                        p1_linear_depth,
                    })
                }
                _ => None,
            };
        let mut cs_id_to_cs_edges = HashMap::new();
        for row in fetched_edges.iter() {
            match *row {
                (
                    cs_id,
                    Some(gen),
                    Some(skip_tree_depth),
                    Some(p1_linear_depth),
                    Some(parent_count),
                    merge_ancestor,
                    merge_ancestor_gen,
                    merge_ancestor_skip_tree_depth,
                    merge_ancestor_p1_linear_depth,
                    skip_tree_parent,
                    skip_tree_parent_gen,
                    skip_tree_parent_skip_tree_depth,
                    skip_tree_parent_p1_linear_depth,
                    skip_tree_skew_ancestor,
                    skip_tree_skew_ancestor_gen,
                    skip_tree_skew_ancestor_skip_tree_depth,
                    skip_tree_skew_ancestor_p1_linear_depth,
                    p1_linear_skew_ancestor,
                    p1_linear_skew_ancestor_gen,
                    p1_linear_skew_ancestor_skip_tree_depth,
                    p1_linear_skew_ancestor_p1_linear_depth,
                    ..,
                ) => {
                    cs_id_to_cs_edges
                        .entry(cs_id)
                        .or_insert_with(|| ChangesetEdges {
                            node: ChangesetNode {
                                cs_id,
                                generation: Generation::new(gen),
                                skip_tree_depth,
                                p1_linear_depth,
                            },
                            parents: ChangesetNodeParents::new(),
                            merge_ancestor: option_fields_to_option_node(
                                merge_ancestor,
                                merge_ancestor_gen,
                                merge_ancestor_skip_tree_depth,
                                merge_ancestor_p1_linear_depth,
                            ),
                            skip_tree_parent: option_fields_to_option_node(
                                skip_tree_parent,
                                skip_tree_parent_gen,
                                skip_tree_parent_skip_tree_depth,
                                skip_tree_parent_p1_linear_depth,
                            ),
                            skip_tree_skew_ancestor: option_fields_to_option_node(
                                skip_tree_skew_ancestor,
                                skip_tree_skew_ancestor_gen,
                                skip_tree_skew_ancestor_skip_tree_depth,
                                skip_tree_skew_ancestor_p1_linear_depth,
                            ),
                            p1_linear_skew_ancestor: option_fields_to_option_node(
                                p1_linear_skew_ancestor,
                                p1_linear_skew_ancestor_gen,
                                p1_linear_skew_ancestor_skip_tree_depth,
                                p1_linear_skew_ancestor_p1_linear_depth,
                            ),
                        });
                }
                _ => continue,
            }
        }

        for row in fetched_edges {
            match *row {
                (
                    cs_id,
                    ..,
                    parent_num,
                    Some(parent),
                    Some(parent_gen),
                    Some(parent_skip_tree_depth),
                    Some(parent_p1_linear_depth),
                ) => {
                    if let Some(edge) = cs_id_to_cs_edges.get_mut(&cs_id) {
                        // Only insert if we have the correct next parent
                        if edge.parents.len() == parent_num {
                            edge.parents.push(ChangesetNode {
                                cs_id: parent,
                                generation: Generation::new(parent_gen),
                                skip_tree_depth: parent_skip_tree_depth,
                                p1_linear_depth: parent_p1_linear_depth,
                            })
                        }
                    }
                }
                _ => continue,
            }
        }

        cs_id_to_cs_edges
    }

    async fn fetch_many_edges_impl(
        &self,
        ctx: &CoreContext,
        cs_ids: &[ChangesetId],
        _prefetch: Prefetch,
        rendezvous: &RendezVousConnection,
    ) -> Result<HashMap<ChangesetId, ChangesetEdges>> {
        if cs_ids.is_empty() {
            // This is actually NECESSARY, because SQL doesn't deal well with
            // querying empty arrays
            return Ok(HashMap::new());
        }

        let ret = rendezvous
            .fetch_single
            .dispatch(ctx.fb.clone(), cs_ids.iter().copied().collect(), || {
                let conn = rendezvous.conn.clone();
                let repo_id = self.repo_id.clone();

                move |cs_ids| async move {
                    let cs_ids = cs_ids.into_iter().collect::<Vec<_>>();

                    let fetched_edges =
                        SelectManyChangesets::query(&conn, &repo_id, cs_ids.as_slice()).await?;
                    Ok(Self::collect_changeset_edges(&fetched_edges))
                }
            })
            .await?;

        Ok(ret
            .into_iter()
            .filter_map(|(cs_id, cs_edge)| cs_edge.map(|cs_edge| (cs_id, cs_edge)))
            .collect())
    }
}

#[async_trait]
impl CommitGraphStorage for SqlCommitGraphStorage {
    fn repo_id(&self) -> RepositoryId {
        self.repo_id
    }

    async fn add_many(&self, ctx: &CoreContext, many_edges: Vec1<ChangesetEdges>) -> Result<usize> {
        // We need to be careful because there might be dependencies among the edges
        // Part 1 - Add all nodes without any edges, so we generate ids for them
        let transaction = self.write_connection.start_transaction().await?;
        let cs_no_edges = many_edges
            .iter()
            .map(|e| {
                (
                    self.repo_id,
                    e.node.cs_id,
                    e.node.generation.value(),
                    e.node.skip_tree_depth,
                    e.node.p1_linear_depth,
                    e.parents.len(),
                )
            })
            .collect::<Vec<_>>();
        let (transaction, result) = InsertChangesetsNoEdges::query_with_transaction(
            transaction,
            cs_no_edges
                .iter()
                // This does &(TypeA, TypeB, ...) -> (&TypeA, &TypeB, ...)
                .map(|(a, b, c, d, e, f)| (a, b, c, d, e, f))
                .collect::<Vec<_>>()
                .as_slice(),
        )
        .await?;
        let modified = result.affected_rows();
        if modified == 0 {
            // Early return, everything is already stored
            return Ok(0);
        }
        // Part 2 - Collect all changesets we need the ids from, and query them
        // using the same transaction
        let mut need_ids = HashSet::new();
        for edges in &many_edges {
            need_ids.insert(edges.node.cs_id);
            edges.merge_ancestor.map(|u| need_ids.insert(u.cs_id));
            edges.skip_tree_parent.map(|u| need_ids.insert(u.cs_id));
            edges
                .skip_tree_skew_ancestor
                .map(|u| need_ids.insert(u.cs_id));
            edges
                .p1_linear_skew_ancestor
                .map(|u| need_ids.insert(u.cs_id));
            for u in &edges.parents {
                need_ids.insert(u.cs_id);
            }
        }
        let (transaction, cs_to_ids) = if !need_ids.is_empty() {
            // Use the same transaction to make sure we see the new values
            let (transaction, result) = SelectManyIds::query_with_transaction(
                transaction,
                &self.repo_id,
                need_ids.into_iter().collect::<Vec<_>>().as_slice(),
            )
            .await?;
            (transaction, result.into_iter().collect())
        } else {
            (transaction, HashMap::new())
        };
        // Part 3 - Fix edges on all changesets we previously inserted
        let get_id = |node: &ChangesetNode| {
            cs_to_ids
                .get(&node.cs_id)
                .copied()
                .with_context(|| format!("Failed to fetch id for changeset {}", node.cs_id))
        };
        let maybe_get_id = |maybe_node: Option<&ChangesetNode>| maybe_node.map(get_id).transpose();
        let rows = match many_edges
            .iter()
            .map(|e| {
                Ok((
                    self.repo_id,
                    e.node.cs_id,
                    e.node.generation.value(),
                    e.node.skip_tree_depth,
                    e.node.p1_linear_depth,
                    e.parents.len(),
                    maybe_get_id(e.parents.first())?,
                    maybe_get_id(e.merge_ancestor.as_ref())?,
                    maybe_get_id(e.skip_tree_parent.as_ref())?,
                    maybe_get_id(e.skip_tree_skew_ancestor.as_ref())?,
                    maybe_get_id(e.p1_linear_skew_ancestor.as_ref())?,
                ))
            })
            .collect::<Result<Vec<_>>>()
        {
            Ok(rows) => rows,
            Err(err) => {
                transaction.rollback().await?;
                return Err(err);
            }
        };

        let (transaction, _) = FixEdges::query_with_transaction(
            transaction,
            rows.iter()
                .map(|(a, b, c, d, e, f, g, h, i, j, k)| (a, b, c, d, e, f, g, h, i, j, k))
                .collect::<Vec<_>>()
                .as_slice(),
        )
        .await?;

        let merge_parent_rows = many_edges
            .iter()
            .flat_map(|edges| {
                edges
                    .parents
                    .iter()
                    .enumerate()
                    .skip(1)
                    .map(|(parent_num, node)| Ok((get_id(&edges.node)?, parent_num, get_id(node)?)))
            })
            .collect::<Result<Vec<_>>>()?;

        let (transaction, result) = InsertMergeParents::query_with_transaction(
            transaction,
            merge_parent_rows
                .iter()
                .map(|(a, b, c)| (a, b, c))
                .collect::<Vec<_>>()
                .as_slice(),
        )
        .await?;

        // All good, nodes were added and correctly updated, let's commit.
        transaction.commit().await?;
        ctx.perf_counters()
            .increment_counter(PerfCounterType::SqlWrites);

        Ok(modified.try_into()?)
    }

    async fn add(&self, ctx: &CoreContext, edges: ChangesetEdges) -> Result<bool> {
        let merge_parent_cs_id_to_id: HashMap<ChangesetId, u64> = if edges.parents.len() >= 2 {
            ctx.perf_counters()
                .increment_counter(PerfCounterType::SqlReadsReplica);
            SelectManyIds::query(
                &self.read_connection.conn,
                &self.repo_id,
                &edges
                    .parents
                    .iter()
                    .map(|node| node.cs_id)
                    .collect::<Vec<_>>(),
            )
            .await?
            .into_iter()
            .collect()
        } else {
            Default::default()
        };

        let transaction = self.write_connection.start_transaction().await?;

        let (transaction, result) = InsertChangeset::query_with_transaction(
            transaction,
            &self.repo_id,
            &edges.node.cs_id,
            &edges.node.generation.value(),
            &edges.node.skip_tree_depth,
            &edges.node.p1_linear_depth,
            &edges.parents.len(),
            &edges.parents.get(0).map(|node| node.cs_id),
            &edges.merge_ancestor.map(|node| node.cs_id),
            &edges.skip_tree_parent.map(|node| node.cs_id),
            &edges.skip_tree_skew_ancestor.map(|node| node.cs_id),
            &edges.p1_linear_skew_ancestor.map(|node| node.cs_id),
        )
        .await?;

        match result.last_insert_id() {
            Some(last_insert_id) if result.affected_rows() == 1 => {
                let merge_parent_rows = edges
                    .parents
                    .iter()
                    .enumerate()
                    .skip(1)
                    .map(|(parent_num, node)| {
                        Ok((
                            last_insert_id,
                            parent_num,
                            *merge_parent_cs_id_to_id
                                .get(&node.cs_id)
                                .ok_or_else(|| anyhow!("Failed to fetch id for {}", node.cs_id))?,
                        ))
                    })
                    .collect::<Result<Vec<_>>>()?;

                let (transaction, result) = InsertMergeParents::query_with_transaction(
                    transaction,
                    merge_parent_rows
                        .iter()
                        .map(|(a, b, c)| (a, b, c))
                        .collect::<Vec<_>>()
                        .as_slice(),
                )
                .await?;

                transaction.commit().await?;
                ctx.perf_counters()
                    .increment_counter(PerfCounterType::SqlWrites);

                Ok(true)
            }
            _ => {
                transaction.rollback().await?;
                Ok(false)
            }
        }
    }

    async fn fetch_edges(
        &self,
        ctx: &CoreContext,
        cs_id: ChangesetId,
    ) -> Result<Option<ChangesetEdges>> {
        Ok(self
            .fetch_many_edges(ctx, &[cs_id], Prefetch::None)
            .await?
            .remove(&cs_id))
    }

    async fn fetch_edges_required(
        &self,
        ctx: &CoreContext,
        cs_id: ChangesetId,
    ) -> Result<ChangesetEdges> {
        self.fetch_many_edges_required(ctx, &[cs_id], Prefetch::None)
            .await?
            .remove(&cs_id)
            .ok_or_else(|| anyhow!("Missing changeset from sql commit graph storage: {}", cs_id))
    }

    async fn fetch_many_edges(
        &self,
        ctx: &CoreContext,
        cs_ids: &[ChangesetId],
        prefetch: Prefetch,
    ) -> Result<HashMap<ChangesetId, ChangesetEdges>> {
        ctx.perf_counters()
            .increment_counter(PerfCounterType::SqlReadsReplica);
        self.fetch_many_edges_impl(ctx, cs_ids, prefetch, &self.read_connection)
            .await
    }

    async fn fetch_many_edges_required(
        &self,
        ctx: &CoreContext,
        cs_ids: &[ChangesetId],
        prefetch: Prefetch,
    ) -> Result<HashMap<ChangesetId, ChangesetEdges>> {
        ctx.perf_counters()
            .increment_counter(PerfCounterType::SqlReadsReplica);
        let mut edges = self
            .fetch_many_edges_impl(ctx, cs_ids, prefetch, &self.read_connection)
            .await?;
        let unfetched_ids: Vec<ChangesetId> = cs_ids
            .iter()
            .filter(|id| !edges.contains_key(id))
            .copied()
            .collect();
        let unfetched_ids = if !unfetched_ids.is_empty() {
            // Let's go to master with the remaining edges
            ctx.perf_counters()
                .increment_counter(PerfCounterType::SqlReadsMaster);
            let extra_edges = self
                .fetch_many_edges_impl(ctx, &unfetched_ids, prefetch, &self.read_master_connection)
                .await?;
            edges.extend(extra_edges);
            cs_ids
                .iter()
                .filter(|id| !edges.contains_key(id))
                .copied()
                .collect()
        } else {
            unfetched_ids
        };
        if !unfetched_ids.is_empty() {
            anyhow::bail!(
                "Missing changesets from sql commit graph storage: {}",
                unfetched_ids
                    .into_iter()
                    .map(|id| format!("{}, ", id))
                    .collect::<String>()
            );
        }
        Ok(edges)
    }

    async fn find_by_prefix(
        &self,
        ctx: &CoreContext,
        cs_prefix: ChangesetIdPrefix,
        limit: usize,
    ) -> Result<ChangesetIdsResolvedFromPrefix> {
        ctx.perf_counters()
            .increment_counter(PerfCounterType::SqlReadsReplica);
        let mut fetched_ids = SelectChangesetsInRange::query(
            &self.read_connection.conn,
            &self.repo_id,
            &cs_prefix.min_bound(),
            &cs_prefix.max_bound(),
            &(limit + 1),
        )
        .await?
        .into_iter()
        .map(|(cs_id,)| cs_id)
        .collect::<Vec<_>>();

        Ok(ChangesetIdsResolvedFromPrefix::from_vec_and_limit(
            fetched_ids,
            limit,
        ))
    }
}
