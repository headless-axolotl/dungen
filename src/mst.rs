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

    // Climb up the tree until the parent is found. On the way back
    // reparent all the nodes in the path to the parent, reducing the
    // time for the next query.
    pub fn find_set(&mut self, entity: usize) -> usize {
        if self.parent[entity] == entity {
            return entity;
        }
        self.parent[entity] = self.find_set(self.parent[entity]);
        self.parent[entity]
    }

    // Merge the two sets based on their rank (depth of the corresponding tree).
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
        mut edges,
    } = triangulation;

    let tree = minimum_spanning_tree(&doorways, &mut edges);

    // Find the edges which are not part of the minimum spanning tree.
    // Since the indices in the tree array are sorted we can do this in O(n) as such.
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
    // doorways of different rooms to the corridor array.
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
    // doorways of different rooms to the corridor array.
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock::MockRng;
    use crate::vec::vec2u;

    #[test]
    fn new_disjoint_set() {
        let disjoint_set_size = 5;
        let mut disjoint_set = DisjointSet::new(disjoint_set_size);

        for entity in 0..disjoint_set_size {
            assert_eq!(
                disjoint_set.find_set(entity),
                entity,
                "Entity {} should be in its own set.",
                entity
            );
        }
    }

    #[test]
    fn set_unification() {
        let disjoint_set_size = 8;
        let sets: &[&[usize]] = &[&[0, 1, 2], &[3, 4, 5, 6], &[7]];
        let mut disjoint_set = DisjointSet::new(disjoint_set_size);

        let mut last;
        for set in sets {
            last = set[0];
            for &entity in *set {
                disjoint_set.union_sets(last, entity);
                last = entity;
            }
        }

        for set in sets {
            last = set[0];
            for &entity in *set {
                assert_eq!(
                    disjoint_set.find_set(last),
                    disjoint_set.find_set(entity),
                    "Entities {} and {} should be in the same set.",
                    last,
                    entity
                );
                last = entity;
            }
        }

        for i in 0..sets.len() - 1 {
            for j in i + 1..sets.len() {
                assert_ne!(
                    disjoint_set.find_set(sets[i][0]),
                    disjoint_set.find_set(sets[j][0]),
                    "Entities {} and {} should be in different sets.",
                    sets[i][0],
                    sets[j][0]
                );
            }
        }
    }

    fn doorway(x: usize, y: usize, room_index: usize) -> Doorway {
        Doorway {
            room_index,
            position: vec2u(x, y),
        }
    }

    #[test]
    fn correct_minimum_spanning_tree() {
        let doorways = &[doorway(1, 1, 0), doorway(4, 1, 0), doorway(1, 3, 0)];
        let edges = &mut [(0, 1), (1, 2), (0, 2)];

        let result = minimum_spanning_tree(doorways, edges);
        assert_eq!(
            &result,
            &[0, 1],
            "The minimum spanning tree should be the two smaller edges of \
             the right triangle in the order of their length."
        );
        assert_eq!(
            edges,
            &[(0, 2), (0, 1), (1, 2)],
            "The order of the edges is incorrect."
        );

        // The order of the edges should not matter.
        let edges = &mut [(0, 2), (0, 1), (1, 2)];
        let result = minimum_spanning_tree(doorways, edges);
        assert_eq!(
            &result,
            &[0, 1],
            "The minimum spanning tree should be the two smaller edges of \
             the right triangle in the order of their length."
        );
        assert_eq!(
            edges,
            &[(0, 2), (0, 1), (1, 2)],
            "The order of the edges is incorrect."
        );
    }

    #[test]
    fn corridors_belong_to_different_rooms() {
        let configuration = Configuration {
            reintroduced_corridor_density: (2, 2),
            ..Default::default()
        };
        let triangulation = RoomGraph {
            rooms: vec![],
            doorways: vec![
                doorway(1, 1, 0),
                doorway(4, 1, 0),
                doorway(1, 3, 0),
                doorway(4, 3, 0),
            ],
            edges: vec![(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)],
        };
        let mut mock_rng = MockRng::new(vec![1]);
        let result = pick_corridors(&configuration, triangulation, &mut mock_rng);

        assert_eq!(
            result.edges.len(),
            0,
            "Since all doorways belong to a single room there should be no corridors picked."
        );
    }

    #[test]
    fn only_minimum_spanning_tree_corridors() {
        let configuration = Configuration {
            reintroduced_corridor_density: (0, 1),
            ..Default::default()
        };
        let triangulation = RoomGraph {
            rooms: vec![],
            doorways: vec![
                doorway(1, 1, 1),
                doorway(4, 1, 2),
                doorway(1, 3, 3),
                doorway(4, 3, 4),
            ],
            edges: vec![(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)],
        };
        let mut mock_rng = MockRng::new(vec![1]);
        let result = pick_corridors(&configuration, triangulation, &mut mock_rng);

        assert_eq!(
            result.edges.len(),
            3,
            "There should be exactly 3 edges in the picked corridors."
        );
        for edge in &[(0, 2), (0, 1), (1, 3)] {
            assert!(
                result.edges.contains(edge),
                "A correct edge is missing from the picked edges."
            );
        }
    }

    #[test]
    fn all_valid_corridors() {
        let configuration = Configuration {
            reintroduced_corridor_density: (1, 1),
            ..Default::default()
        };
        let triangulation = RoomGraph {
            rooms: vec![],
            doorways: vec![
                doorway(1, 1, 1),
                doorway(4, 1, 2),
                doorway(1, 3, 3),
                doorway(4, 3, 4),
            ],
            edges: vec![(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)],
        };
        let mut mock_rng = MockRng::new(vec![1]);
        let result = pick_corridors(&configuration, triangulation, &mut mock_rng);

        assert_eq!(
            result.edges.len(),
            5,
            "There should be exactly 5 edges in the picked corridors."
        );
    }
}
