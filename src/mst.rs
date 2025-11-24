use crate::{Configuration, rng::Rng, room::RoomGraph};

#[derive(Debug)]
pub struct DisjointSet {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl DisjointSet {
    /// Creates a new DisjointSet structure, reserving space for a given number of entities.
    pub fn new(entity_count: usize) -> Self {
        Self {
            parent: (0..entity_count).collect(),
            rank: vec![0; entity_count],
        }
    }

    /// Climb up the tree until the parent is found. On the way back reparent all the nodes in the
    /// path to the parent, reducing the time for the next query.
    pub fn find_set(&mut self, entity: usize) -> usize {
        if self.parent[entity] == entity {
            return entity;
        }
        self.parent[entity] = self.find_set(self.parent[entity]);
        self.parent[entity]
    }

    /// Merge the two sets based on their rank (depth of the corresponding tree).
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
pub fn minimum_spanning_tree<P: Clone + Into<raylib::math::Vector2>>(
    points: &[P],
    edges: &mut [(usize, usize)],
) -> Vec<usize> {
    let mut edge_indices: Vec<usize> = vec![];

    edges.sort_by_key(|edge| {
        (points[edge.0].clone().into() - points[edge.1].clone().into()).length_sqr() as usize
    });

    let mut disjoint_set = DisjointSet::new(points.len());
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
    use crate::mock::doorway;
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
                disjoint_set.union_sets(entity, last);
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

    #[test]
    fn correct_minimum_spanning_tree() {
        let points = &[vec2u(1, 1), vec2u(4, 1), vec2u(1, 3)];
        let edges = &mut [(0, 1), (1, 2), (0, 2)];

        let result = minimum_spanning_tree(points, edges);
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
        let result = minimum_spanning_tree(points, edges);
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

    use rand::distr::Distribution;
    use raylib::math::Vector2;

    /// Uses Prim's algorithm on a full graph to find the minimum spanning tree. Returns the
    /// points, the full graph and the MST in order to test Kruskal's algorithm.
    #[allow(clippy::type_complexity)]
    fn minimum_spanning_tree_prim(
        vertex_count: usize,
    ) -> (Vec<Vector2>, Vec<(usize, usize)>, Vec<(usize, usize)>) {
        let uniform = rand::distr::Uniform::new(0, 1000).unwrap();
        let mut rng = rand::rng();

        let mut points: Vec<Vector2> = Vec::with_capacity(vertex_count);
        let mut edge_list: Vec<(usize, usize)> =
            Vec::with_capacity((vertex_count * (vertex_count + 1)) / 2);
        let mut mst: Vec<(usize, usize)> = Vec::with_capacity(vertex_count - 1);
        for vertex in 0..vertex_count {
            points.push(vec2u(uniform.sample(&mut rng), uniform.sample(&mut rng)));
            for previous in 0..vertex_count {
                edge_list.push((vertex, previous));
            }
        }

        let mut in_tree = vec![false; vertex_count];
        // Pairs of (distance, parent_index).
        let mut min_edge = vec![(usize::MAX, usize::MAX); vertex_count];
        min_edge[0] = (0, usize::MAX);
        let mut next;
        let mut distance;
        for _ in 0..vertex_count {
            next = vertex_count;
            for vertex in 0..vertex_count {
                if !in_tree[vertex]
                    && (next == vertex_count || min_edge[vertex].0 < min_edge[next].0)
                {
                    next = vertex;
                }
            }

            if next == vertex_count {
                break;
            }

            in_tree[next] = true;
            if min_edge[next].1 != usize::MAX {
                mst.push(crate::triangulation::make_edge(next, min_edge[next].1));
            }

            for to in 0..vertex_count {
                distance = (points[next] - points[to]).length_sqr() as usize;
                if distance < min_edge[to].0 {
                    min_edge[to] = (distance, next);
                }
            }
        }

        mst.sort_by_key(|edge| (points[edge.0] - points[edge.1]).length_sqr() as usize);

        (points, edge_list, mst)
    }

    fn test_kruskal_with_prim(vertex_count: usize) {
        let (points, mut edges, mst_prim) = minimum_spanning_tree_prim(vertex_count);
        let mst_kruskal_indices = minimum_spanning_tree(&points, &mut edges);
        let mst_kruskal = mst_kruskal_indices
            .iter()
            .map(|index| edges[*index])
            .collect::<Vec<_>>();
        assert_eq!(
            mst_prim.len(),
            mst_kruskal.len(),
            "Minimum spanning trees should match."
        );
        for i in 0..mst_prim.len() {
            let length_prim = (points[mst_prim[i].0] - points[mst_prim[i].1]).length_sqr();
            let length_kruskal = (points[mst_kruskal[i].0] - points[mst_kruskal[i].1]).length_sqr();
            assert_eq!(
                length_prim, length_kruskal,
                "Minimum spanning trees should match."
            );
        }
    }

    #[test]
    fn kruskal_with_prim_tests() {
        test_kruskal_with_prim(10);
        test_kruskal_with_prim(100);
        test_kruskal_with_prim(1000);
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
