use petgraph::algo::dijkstra;
use petgraph::prelude::*;
use petgraph::Graph;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::vec;

pub type Cluster = Vec<NodeIndex>;
type DistanceMapMap = HashMap<NodeIndex, HashMap<NodeIndex, i32>>;

fn graph_get_center(dmm: &DistanceMapMap, cluster: &Cluster) -> NodeIndex {
    let mut center = cluster[0];
    let mut distance = std::i32::MAX;
    for node in cluster {
        let distances_map = dmm.get(node).unwrap();
        let d: Vec<i32> = cluster
            .iter()
            .map(|n| distances_map.get(n))
            .map(|x| x.unwrap_or(&MAX_DISTANCE))
            .map(|&x| x)
            .collect();
        let ds = d.iter().sum();
        if ds < distance {
            center = *node;
            distance = ds;
        }
    }
    return center;
}

const MAX_DISTANCE: i32 = 100;
fn graph_get_distance(dmm: &DistanceMapMap, a: &NodeIndex, b: &NodeIndex) -> i32 {
    let dm = dmm.get(a);
    if dm.is_none() {
        return MAX_DISTANCE;
    }
    let d = dm.unwrap().get(b);
    if d.is_none() {
        return MAX_DISTANCE;
    }
    return d.unwrap().clone();
}

pub fn cluster_graph_k_mean(graph: &Graph<u64, (), Undirected>, k: usize) -> Vec<Cluster> {
    let mut rng = thread_rng();
    let mut clusters: Vec<Cluster> = Vec::new();

    let mut nodes: Vec<NodeIndex> = graph.node_indices().collect();
    nodes.shuffle(&mut rng);
    for i in 0..k {
        clusters.push(vec![nodes[i]]);
    }

    // only need init once.
    let mut node_dijkstra_map_map: DistanceMapMap = HashMap::new();
    for node in nodes.iter() {
        let distance_map = dijkstra(&graph, *node, None, |_e| 1);
        node_dijkstra_map_map.insert(*node, distance_map);
    }

    // calculate each node
    for (ni, node) in nodes.iter().enumerate() {
        if ni < k {
            continue;
        }
        let mut closest_cluster: Option<usize> = None;
        let mut closest_distance: i32 = std::i32::MAX;

        for (i, cluster) in clusters.iter().enumerate() {
            let center_node = graph_get_center(&node_dijkstra_map_map, cluster);
            // log for node
            if ni % 100 == 0 {
                println!(
                    "calcuate center for node {} each cluster {} got {}",
                    ni, i, graph[center_node]
                );
            }
            let dist: i32 = graph_get_distance(&node_dijkstra_map_map, node, &center_node);
            if dist < closest_distance {
                closest_cluster = Some(i);
                closest_distance = dist;
            }
        }

        if let Some(i) = closest_cluster {
            clusters[i].push(*node);
        }
    }

    return clusters;
}

pub fn build_graph(edges: Vec<(u64, u64)>) -> Graph<u64, (), Undirected> {
    let mut graph: Graph<u64, (), Undirected> = Graph::new_undirected();
    let mut node_indices = vec![];
    for &(source, target) in &edges {
        let source_index: petgraph::stable_graph::NodeIndex =
            if let Some(index) = node_indices.iter().position(|&i| i == source) {
                NodeIndex::new(index)
            } else {
                let index = graph.add_node(source);
                node_indices.push(source);
                index
            };
        let target_index = if let Some(index) = node_indices.iter().position(|&i| i == target) {
            NodeIndex::new(index)
        } else {
            let index = graph.add_node(target);
            node_indices.push(target);
            index
        };
        graph.add_edge(source_index, target_index, ());
    }
    return graph;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cmp::min, u32::MAX, vec};

    #[test]
    fn test_for_build_graph() {
        let edges: Vec<(u64, u64)> = vec![(1, 2), (1, 3), (2, 4), (3, 4), (4, 5), (5, 1), (5, 2)];
        let g = build_graph(edges);
        assert_eq!(g.node_count(), 5);
        assert_eq!(g.edge_count(), 7);
    }

    #[test]
    fn test_cluster_graph_k_mean() {
        let edges: Vec<(u64, u64)> = vec![(1, 2), (1, 3), (2, 3), (4, 5)];
        let g = build_graph(edges);
        let clusters = cluster_graph_k_mean(&g, 2);
        let mut a: Vec<u64> = clusters[0].iter().map(|&n| g[n]).into_iter().collect();
        let mut b: Vec<u64> = clusters[1].iter().map(|&n| g[n]).into_iter().collect();

        println!("test_cluster_graph_k_mean a clusters {:#?} b {:#?}", a, b);
        if a.len() < b.len() {
            (a, b) = (b, a)
        }
        assert_eq!(a.len() + b.len(), 5);
        //assert_eq!(a.len(), 3); // [1, 2, 3]
        //assert_eq!(b.len(), 2); // [4, 5]
    }
}
