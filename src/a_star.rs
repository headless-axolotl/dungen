use crate::Configuration;
use crate::binary_heap::Heap;
use crate::grid::Tile;

/// Absolute difference between two unsigned integers
pub fn diff(a: usize, b: usize) -> usize {
    if a < b { b - a } else { a - b }
}

/// Manhatan distance between two indices representing coordinates in a grid.
fn manhattan(width: usize, a: usize, b: usize) -> usize {
    diff(a / width, b / width) + diff(a % width, b % width)
}

fn make_square(width: usize, mut nodes: [usize; 4]) -> bool {
    nodes.sort();
    nodes[0] + 1 == nodes[1] && nodes[0] + width == nodes[2] && nodes[0] + width + 1 == nodes[3]
}

/// A* pathfinding algorithm to carve corridors in the grid. It follows a couple of rules to
/// guarantee that the corridors follow a specific shape. Since this procedure will be called
/// multiple times, there is no need to allocate the needed structures more than once.
///
/// The heuristic used is the manhatan distance from this cell to the end multiplied by the minimum
/// cost (with respect of all types of cost). The heuristic is admissible and consistent as we
/// assume the best case of traveling only through the cheapest possible path.
#[allow(clippy::too_many_arguments)]
pub fn a_star(
    configuration: &Configuration,
    start: usize,
    end: usize,
    width: usize,
    tiles: &[Tile],
    open_set: &mut Heap<usize, usize>,
    g_scores: &mut Vec<usize>,
    parent: &mut Vec<usize>,
    path: &mut Vec<usize>,
) {
    use Tile::*;

    let corridor_cost = configuration.corridor_cost;
    let straight_cost = configuration.straight_cost;
    let standard_cost = configuration.standard_cost;
    let min_cost = corridor_cost.min(straight_cost).min(standard_cost);

    // Initialize the structures.
    open_set.clear();
    // Expect the grids to be small enough so that half of usize::MAX is effectively infinity.
    g_scores.clear();
    g_scores.resize(tiles.len(), usize::MAX / 2);
    parent.clear();
    for i in 0..tiles.len() {
        parent.push(i);
    }
    path.clear();

    g_scores[start] = 0;
    open_set.insert(manhattan(width, start, end) * min_cost, start);

    while let Some((f_cost, mut current)) = open_set.extract_min() {
        let g_cost = f_cost - manhattan(width, current, end) * min_cost;
        if g_cost > g_scores[current] {
            continue;
        }

        if current == end {
            path.push(current);
            while current != parent[current] {
                current = parent[current];
                path.push(current);
            }
            return;
        }

        // We expect that the grid is such that it is guaranteed that the edges are unreachable.
        for neighbor in [
            current + width, // south
            current - width, // north
            current + 1,     // east
            current - 1,     // west
        ] {
            // We cannot go through rooms or blockers.
            if matches!(tiles[neighbor], Blocker | Room) {
                continue;
            }
            // We cannot go to another corridor neighbor if we also are a corridor neighbor. This
            // covers one of the cases where the corridor would make a 2x2 square which is one of
            // the rules the corridors must follow.
            if matches!(tiles[current], CorridorNeighbor)
                && matches!(tiles[neighbor], CorridorNeighbor)
            {
                continue;
            }
            // If the path makes a square, this is an invalid neighbor.
            if make_square(
                width,
                [neighbor, current, parent[current], parent[parent[current]]],
            ) {
                continue;
            }

            let cost = if matches!(tiles[neighbor], Corridor) {
                corridor_cost
            } else if diff(current, parent[current]) == diff(neighbor, current) {
                straight_cost
            } else {
                standard_cost
            };

            let tentative_g_score = g_scores[current] + cost;
            if tentative_g_score < g_scores[neighbor] {
                parent[neighbor] = current;
                g_scores[neighbor] = tentative_g_score;
                let f_score = tentative_g_score + manhattan(width, neighbor, end) * min_cost;
                open_set.insert(f_score, neighbor);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::grid::Grid;
    use crate::vec::{to_index, vec2u};

    #[test]
    fn unsigned_int_difference() {
        assert_eq!(diff(1, 2), 1, "Absolute difference is not correct.");
        assert_eq!(diff(2, 1), 1, "Absolute difference is not correct.");
    }

    #[test]
    fn manhattan_is_corerct() {
        let width = 10;
        let row_a = 3;
        let col_a = 4;
        let row_b = 8;
        let col_b = 1;
        assert_eq!(
            manhattan(width, row_a * width + col_a, row_b * width + col_b),
            row_b - row_a + col_a - col_b,
            "Manhattan distance is not correct."
        )
    }

    fn make_structures() -> (Heap<usize, usize>, Vec<usize>, Vec<usize>, Vec<usize>) {
        (Heap::with_capacity(1000), vec![], vec![], vec![])
    }

    #[test]
    #[should_panic]
    fn grid_edge_should_be_unreachable() {
        let configuration = Configuration::default();
        let Grid { width, tiles } = Grid::from(
            "\
            #####\n\
            ##%##\n\
            #d%d#\n\
            ##%##\n\
            #####\n",
        );
        let start = to_index(vec2u(1, 2), width);
        let end = to_index(vec2u(3, 2), width);
        let (mut open_set, mut g_scores, mut parent, mut path) = make_structures();
        a_star(
            &configuration,
            start,
            end,
            width,
            &tiles,
            &mut open_set,
            &mut g_scores,
            &mut parent,
            &mut path,
        );
    }

    #[test]
    fn no_path() {
        let configuration = Configuration::default();
        let Grid { width, tiles } = Grid::from(
            "\
            %%%%%\n\
            %#%#%\n\
            %d%d%\n\
            %#%#%\n\
            %%%%%\n",
        );
        let start = to_index(vec2u(1, 2), width);
        let end = to_index(vec2u(3, 2), width);
        println!("{:?}", tiles[start]);
        println!("{:?}", tiles[start]);
        let (mut open_set, mut g_scores, mut parent, mut path) = make_structures();
        a_star(
            &configuration,
            start,
            end,
            width,
            &tiles,
            &mut open_set,
            &mut g_scores,
            &mut parent,
            &mut path,
        );
        assert_eq!(path.len(), 0, "A path was found when there was not one.");
    }

    #[test]
    fn straight() {
        let configuration = Configuration::default();
        let Grid { width, tiles } = Grid::from(
            "\
            %%%%%%%%%%%\n\
            %#########%\n\
            %#########%\n\
            %d#######d%\n\
            %#########%\n\
            %#########%\n\
            %%%%%%%%%%%\n",
        );
        let start = to_index(vec2u(1, 3), width);
        let end = to_index(vec2u(9, 3), width);
        let (mut open_set, mut g_scores, mut parent, mut path) = make_structures();
        a_star(
            &configuration,
            start,
            end,
            width,
            &tiles,
            &mut open_set,
            &mut g_scores,
            &mut parent,
            &mut path,
        );
        assert_eq!(path.len(), 9, "Path length was incorrect.");
    }

    #[test]
    fn with_wall() {
        let configuration = Configuration::default();
        let Grid { width, tiles } = Grid::from(
            "\
            %%%%%%%%%%%\n\
            %#########%\n\
            %####%####%\n\
            %d###%###d%\n\
            %####%####%\n\
            %####%####%\n\
            %%%%%%%%%%%\n",
        );
        let start = to_index(vec2u(1, 3), width);
        let end = to_index(vec2u(9, 3), width);
        let (mut open_set, mut g_scores, mut parent, mut path) = make_structures();
        a_star(
            &configuration,
            start,
            end,
            width,
            &tiles,
            &mut open_set,
            &mut g_scores,
            &mut parent,
            &mut path,
        );
        assert_eq!(path.len(), 13, "Path length was incorrect.");
    }

    #[test]
    fn modified_costs() {
        // The algorithm should now choose to go through the already placed corridor.
        let configuration = Configuration {
            straight_cost: 9,
            standard_cost: 10,
            ..Default::default()
        };
        let Grid { width, tiles } = Grid::from(
            "\
            %%%%%%%%%%%\n\
            %#########%\n\
            %#########%\n\
            %d#######d%\n\
            %##@@@@@##%\n\
            %#@ccccc@#%\n\
            %%%%%%%%%%%\n",
        );
        let start = to_index(vec2u(1, 3), width);
        let end = to_index(vec2u(9, 3), width);
        let (mut open_set, mut g_scores, mut parent, mut path) = make_structures();
        a_star(
            &configuration,
            start,
            end,
            width,
            &tiles,
            &mut open_set,
            &mut g_scores,
            &mut parent,
            &mut path,
        );
        assert_eq!(path.len(), 13, "Path length was incorrect.");
    }
}
