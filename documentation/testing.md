
# Testing

## Coverage

Unit test coverage is reported using the crate
[`cargo-tarpaulin`](https://crates.io/crates/cargo-tarpaulin).
This tool is not entirely accurate so even though it reports test coverage below
100%, some of the lines that are reportedly skipped cannot be as lines before
them are not skipped. This can be more clearly seen by generating the **html**
output by running `cargo tarpaulin --engine llvm --out Html`. Running just the
tests without code coverage can be done by running `cargo test`.

Unit tests which test the functionality of a given module live in the same file
as that module. They are in the submodule `test` which is marked by
`#[cfg(test)]`.

As of the last commit the coverage is: 95.91%.

## Methodology

The name of the test functions give hints as to what specific thing I am
testing. In addition, I have placed some comments inside the tests to provide
further insight as to how and why I am testing a given procedure.

### Binary heap

Tested here are insertion and removal of elements as well as other procedures
that are supported on this data structure such as clearing and checking whether
the structure is empty. Tests are performed before and after clearing the
structure to check whether a correct state is maintained.

### A*

Two tests here check whether utility procedures work correctly. After that
several representative tests check for the correct behaviour of the procedure
which implements A*. The first test checks whether the procedure panics (aborts)
when the edges of the map are reachable (in practice we don't want them to be).
The procedures after that check whether the algorithm finds a path of correct
length. Different paths are tested such as a straight path, a path which must go
around a wall, a map where the other point is unreachable in which case we
expect the path to be empty and a map in which it is more affordable to go
through a corridor which would have already been placed by the grid generation
procedure.

### Minimum spanning tree and picking of corridors

Tested here is the Disjoint Set data structure which is needed by the minimum
spanning tree algorithm. Two tests are performed. The first checks whether the
structure is initialised correctly after it is created. The second tests the
functionality of operations on the structure. Chosen are sequences of entities
whose sets should be merged. They are merged in an order such that every branch
of the procedure is explored. Checked afterwards is that each entity is in the
same set that it was merged in and that entities from different sets are in fact
in different sets in the data structure.

The minimum spanning tree is tested with a simple triangle graph. Tested are two
orderings of the input edges which should not matter. The output is checked to
be the two smaller sides of the triangle.

The corridor picking procedure is tested with a mock random number generator
with preset numbers. The tests performed check different options of the config -
no edges are reintroduced as corridors and all edges are reintroduced as
corridors. Tested is also that the final set of edges does not contain any edges
whose ends belong to the same room (i.e. a corridor connecting to the same
room).

### Grid generation

Tested here is the serialization and deserialization of the Grid structure which
consists of a width specifier and an array which represents the flattened
matrix of the map.

Tests here are also performed on the procedure which carves corridors through
the grid. Since the grid serialization and deserialization is already tested the
initialisation of the test is done with deserializing strings instead of listing
the variants of the Tile enum. In the singular test that is given we can check
whether the corridor placement procedure handles every needed case correctly
such as not replaceing blocker tiles, placing corridor neighbor tiles in the
correct spots and turning them into blocker tiles when a corridor makes a turn
for example.

Two grid generation tests are performed then. One of them checks whether the
grid generation procedure makes the edges of the map unreachable by placing
blocker tiles in a one-wide line across the border of the grid. Checked is also
whether the rooms and their blocker tiles are also placed correctly. Since the
A* algorithm and corridor placement procedure are already tested, there is no
need to perform a test with more complex room/doorway/corridor placement.

### Room generation

Tested here is the utility function which checks whether two rooms (rectangles)
are a certain distance away from each other. The tests include an overlapping
case, a non-overlapping case and a case where the two rectangles are in contact
but have no overlapping area in which they are considered to be not overlapping.

Tests are also included for the placement of doorways. Two test variants are
performed with two mock random number generators. One always picks the lowest
number in the range and the other the highest. After the doorways are generated
it is checked whether the correct number of doorways is generated and whether
they are a given distance away from the corners of the outline of the room (an
aesthetic choice).

The room generation is tested with a restrictive map size to ensure that the
algorithm only places valid rooms i.e. ones which are a given distance away from
the edges of the map and ones which do not overlap. Different termination
conditions are tested such as reaching the maximum number of failures allowed.

### Triangulation

Two simple and one bigger unit tests are performed here. The first simple test
checks whether the procedure correctly constructs a triangle from three points
and the other whether the procedure correctly constructs two triagnles from four
points. By extension these tests also check whether the helper points which are
placed around the rectangle of the map and any edges connected to them are
removed from the list. The bigger test checks the triangulation of more points.

### Vector operations

There are two procedures that are tested here. The first is the procedure which
checks whether a point is in a circumcircle. Tested are the cases when the point
is in the circumcircle and when it is not. In addition a check is performed to
confirm that the procedure works with larger numbers since it avoids division
and performing square roots for efficiency's sake.

The second procedure tetsed is a simple conversion from the 2D vector structure
to an index in a flattened matrix array. This test is more of a sanity check.

### Manual testing

Using the visualisation tools more extreme configuration values were tested. In
addition, more of the significant bugs were easier to detect visually using the
GUI application. The maze generation was tested manually.

