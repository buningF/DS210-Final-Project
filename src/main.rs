use graphcluster::build_graph;
use graphcluster::cluster_graph_k_mean;
use std::collections::HashMap;
use std::vec;

const MAX_ROW_READ_COUNT: i32 = 4000;

fn load_csv(filename: &str) -> Vec<(u64, u64)> {
    // read csv
    let mut rows: Vec<Vec<String>> = vec![];
    let mut reader: csv::Reader<std::fs::File> =
        csv::Reader::from_path(filename).expect("no such file");
    let mut i = 0;
    for result in reader.records() {
        let record: csv::StringRecord = result.unwrap();
        let row: Vec<String> = record
            .iter()
            .map(|field: &str| field.to_string())
            .collect::<Vec<_>>();
        //println!("row is {} {}", row[0], row[1]);
        rows.push(row);
        i = i + 1;
        if i > MAX_ROW_READ_COUNT {
            break;
        }
    }

    let mut exist_map = HashMap::<u64, ()>::new();
    for _row in rows.iter() {
        let id: u64 = _row[0].parse::<u64>().unwrap();
        exist_map.insert(id, ());
    }

    let mut edges: Vec<(u64, u64)> = vec![];
    for row in rows.iter() {
        let id: u64 = row[0].parse::<u64>().unwrap();
        let clean_row = row[1].replace("[", "").replace("]", "").replace("'", "");
        if clean_row.is_empty() {
            continue;
        }
        let friends: Vec<u64> = clean_row
            .trim()
            .split(", ")
            .into_iter()
            .map(|s| s.parse::<u64>().unwrap())
            .collect();
        for friend in friends.iter() {
            // only take that both in data.
            if exist_map.get(friend).is_some() {
                let fid = friend.clone();
                edges.push((id, fid));
            }
        }
    }
    println!("total point edges {}", edges.len());
    return edges;
}

fn run_twitter_friend(filename: &str) {
    let edges = load_csv(filename);
    let g = build_graph(edges);
    println!(
        "graph node_count {} edge_count {}",
        g.node_count(),
        g.edge_count()
    );
    // retry N times
    let retry: usize = 2;
    for _ in 0..retry {
        let clusters = cluster_graph_k_mean(&g, 3);
        for (i, cluster) in clusters.iter().enumerate() {
            println!("cluster {} has size {}", i, cluster.len())
        }
    }
}

fn main() {
    run_twitter_friend("./data/twitter.clean.4k.csv");
}
