use crate::binary_heap::Heap;
use crate::grid::Tile;

const CORRIDOR_COST: usize = 1;
const STANDARD_COST: usize = 3;
const _: () = assert!(
    CORRIDOR_COST <= STANDARD_COST,
    "The cost through a corridor should be less than or equal to the standard cost."
);

/// Absolute difference between two unsigned integers
pub fn diff(a: usize, b: usize) -> usize {
    if a < b { b - a } else { a - b }
}

/// Manhatan distance between two indices representing coordinates in a grid.
fn manhattan(width: usize, a: usize, b: usize) -> usize {
    diff(a / width, b / width) + diff(a % width, b % width)
}

/// A* pathfinding algorithm to carve corridors in the grid. It follows a couple of rules to
/// guarantee that the corridors follow a specific shape. Since this procedure will be called
/// multiple times, there is no need to allocate the needed structures more than once.
///
/// The heuristic used is the manhatan distance from this cell to the end multiplied by the
/// corridor cost (which should be less than the standard cost). We want the algorithm to prefer to
/// go through corridors instead of carving new ones. The heuristic is admissible and consistent as
/// we assume the best case of traveling only through corridors till the end.
#[allow(clippy::too_many_arguments)]
pub fn a_star(
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
    open_set.insert(manhattan(width, start, end) * CORRIDOR_COST, 0);

    while let Some((_, mut current)) = open_set.extract_min() {
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

            let cost = if matches!(tiles[neighbor], Corridor) {
                CORRIDOR_COST
            } else {
                STANDARD_COST
            };

            let tentative_g_score = g_scores[current] + cost;
            if tentative_g_score < g_scores[neighbor] {
                parent[neighbor] = current;
                g_scores[neighbor] = tentative_g_score;
                let f_score = tentative_g_score + manhattan(width, neighbor, end) * CORRIDOR_COST;
                open_set.insert(f_score, neighbor);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
}
