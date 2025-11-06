use crate::{
    Configuration,
    rng::Rng,
    room::{Doorway, RoomGraph},
};

#[derive(Debug)]
pub struct DisjointSet {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl DisjointSet {
    pub fn new(entity_count: usize) -> Self {
        Self {
            parent: (0..entity_count).collect(),
            rank: vec![0; entity_count],
        }
    }

    pub fn find_set(&mut self, entity: usize) -> usize {
        if self.parent[entity] == entity {
            return entity;
        }
        self.parent[entity] = self.find_set(self.parent[entity]);
        self.parent[entity]
    }

    pub fn union_sets(&mut self, mut entity_a: usize, mut entity_b: usize) {
        entity_a = self.find_set(entity_a);
        entity_b = self.find_set(entity_b);
        if entity_a == entity_b {
            return;
        }
        if self.rank[entity_a] < self.rank[entity_b] {
            std::mem::swap(&mut entity_a, &mut entity_b);
        }
        self.parent[entity_b] = entity_a;
        if self.rank[entity_a] == self.rank[entity_b] {
            self.rank[entity_a] += 1;
        }
    }
}

/// Employs Kruskal's algorithm to find a minimum spanning tree of the triangulation. Returns the
/// indices of edges which are part of the minimum spanning tree.
pub fn minimum_spanning_tree(doorways: &[Doorway], edges: &mut [(usize, usize)]) -> Vec<usize> {
    let mut edge_indices: Vec<usize> = vec![];

    edges.sort_by_key(|edge| {
        (doorways[edge.0].position - doorways[edge.1].position).length_sqr() as usize
    });

    let mut disjoint_set = DisjointSet::new(doorways.len());
    for (edge_index, edge) in edges.iter().enumerate() {
        if disjoint_set.find_set(edge.0) == disjoint_set.find_set(edge.1) {
            continue;
        }
        edge_indices.push(edge_index);
        disjoint_set.union_sets(edge.0, edge.1);
    }

    edge_indices
}

/// Picks the final corridors from the triangulation by creating a MST and reintroducing some of
/// the edges of the triangulation back.
pub fn pick_corridors<R: Rng>(
    configuration: &Configuration,
    triangulation: RoomGraph,
    rng: &mut R,
) -> RoomGraph {
    let RoomGraph {
        rooms,
        doorways,
        mut edges
    } = triangulation;

    let tree = minimum_spanning_tree(&doorways, &mut edges);

    // Find the edges which are not part of the minimum spanning tree.
    // Since the indices in the tree list are sorted we can do this in O(n) as such.
    let mut residual_edges: Vec<usize> = vec![];
    let mut tree_edge_index: usize = 0;
    for edge_index in 0..edges.len() {
        if tree_edge_index < tree.len() && edge_index == tree[tree_edge_index] {
            tree_edge_index += 1;
            continue;
        }
        residual_edges.push(edge_index);
    }

    let mut corridors: Vec<(usize, usize)> = vec![];

    // Add edges from the minimum spanning tree which connect
    // doorways of different rooms to the corridor list.
    let mut edge: (usize, usize);
    for edge_index in tree {
        edge = edges[edge_index];
        if doorways[edge.0].room_index != doorways[edge.1].room_index {
            corridors.push(edge);
        }
    }

    // Even though the rand library provides a facility to shuffle a sequence,
    // that function calls different methods on the random number generators,
    // however, for the purposes of testing I need to be able to mock them.
    //
    // Randomly add edges not in the minimum spanning tree which connect
    // doorways of different rooms to the corridor list.
    for residual_edge_index in residual_edges {
        edge = edges[residual_edge_index];
        if doorways[edge.0].room_index == doorways[edge.1].room_index {
            continue;
        }
        let filter_number = rng.random_range(1..=configuration.reintroduced_corridor_density.1);
        if filter_number <= configuration.reintroduced_corridor_density.0 {
            corridors.push(edge);
        }
    }

    RoomGraph {
        rooms,
        doorways,
        edges: corridors,
    }
}

