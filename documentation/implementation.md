
# Implementation

## Performance

Even though the goal of this dungeon generator is to pre-generate a dungeon for
a game and not do it dynamically during play, which means that speed is not too
important, I have chosen more memory and time optimal solutions to problems I
have encountered during development.

## Project Organisation

Procedures and structures which are related to them are kept in separate
modules (files). The code for the binary application is in the designated `bin`
directory and since it also includes multiple modules it is in its own
subdirectory.

## Generation Pipeline

### Room and Doorway generation

Generating a 2D dungeon begins with generating the rooms in the dungeon. Given
are the dimensions of the map area and some configuration which determines the
minimal and maximal dimensions of the rooms. Also given are the minimum offset
between rooms and the minimum offset of the doorways to the corners of the
doorway. The former number ensures that we can generate a corridor between every
two doorways and thhe latter number is an aesthetic choice. The configuration
also contains a number which determines how many times we can fail to generate a
random room before we terminate the room generation algorithm (possibly before
generating the desired number of rooms). The bigger this number is, the higher
the denser the room placement will be when we choose a relatively big number of
target rooms. This algorithm simply chooses a random spot in the map, limited by
the aforementioned configuration numbers and random dimensions. Then it checks
whether the room overlaps with any other room (taking into account the minimum
offset which in the code is referred to as padding). This is not the most
efficient way to generate the rooms. Its time complexity is O(FN^2) where F is
the maximum fail count and N is the desired (i.e. "target") number of rooms.

### Corridor Edges

The generated rooms and doorways are then passed to the Bowyer-Watson
triangulation algorithm. The doorways are kept in a separate array from the
rooms which makes it easier to iterate through them. The complexity of this
procedure is O(N^2). The information about the circumcircles is not stored
separately as suggested in the Wikipedia article, however, calculating whether a
point is in a circumcircle of a triangle is not too inefficient.

The result of this procedure is a "room graph". This graph is very similar to
the structure passed as input to the triangulation procedure. Where it differs,
is the addition of an array of edges. We do not need to keep a more complex
structure as the next step, namely calculating a Minimum Spanning Tree, does not
need it. In fact, we need an array of edges in order to sort it as one of the
steps of the Kruskall algorithm (whose time complexity is O(|E|log|V|)). Finding
a MST is not the only way to trim the graph and still have every a graph where
every room is connected to every other through a path.

Finding the MST is the first step in the procedure of picking corridors which
need to be "carved". Afterwards, edges between doorways which belong to the same
room are removed. This does not change the connectivity between separate rooms.
In addition, some portion of the edges which do not participate in the MST and
which also do not belong to the same room are randomly "reintroduced" to make
the generated dungeon more interesting. In the resulting array of edges, those
which belonged to the MST appear first. This is somewhat important later.

### Carving Corridors between Doorways

The next step after the choice of edges is to turn them into corridors. Until
now the procedures have worked with an abstraction of the map area which does
not include the actual tiles. During this step i.e. the generation of the grid,
this is done. To make the job of the pathfinding algorithm easier, the grid
generator creates a border of "blocker" tiles around the edges of the gird,
which are unreachable. The grid generation algorithm then places room tiles as
specified by the room structure and also surrounds them by a border of blockers.

The choice for this dungeon generator is that the border between two "empty"
tiles (belonging to a room and a corridor for example) should be a "full" tile
as opposed to putting a thin wall between them. This is easier to imagine as the
full tiles being wooden blocks and the empty tiles being holes in the grid. This
is the reason behind the blocker tiles.

As the next step the grid generation algorithm places the doorways next to the
rooms. In this way it overrides any blocker tiles in the way. Afterwards for
each corridor edge in the room graph, one by one, a pathfinding algorithm is run
to generate a path which will later be "carved" in the grid. There are some
additional rules which the pathfinding algorithm must follow in order to
generate a correct corridor in the opinion of this specific generator. A valid
corridor only goes in the cardinal directions of the map (i.e. respecting the
grid lines). A set of valid corridors do not create a 2x2 set of tiles which are
all corridors. There are a handful of ways this is guaranteed. The first is that
a path will not directly make a 2x2 which is done by selectively adding the
neighbors of the current tiles to the queue of the A* pathfinding algorithm
which is used. The selection is done based on the parents of the current tile
and the state of the tiles. The second way this is guaranteed is the placement
of special marker tiles and the aforementioned blocker tiles. A special marker
tile is unreachable from any other special marker tile. These marker tiles
(referred to as "corridor neighbors" in code) are placed by the corridor carving
procedure which the grid generation procedure calls after the pathfinding
algorithm has found a path between two doorways. When placing a corridor tile
over a wall tile the carving procedure replaces all its neighbors (tiles which
share a side with this one) with marker tiles if they are wall tiles. If a
marker tile would be placed on top of another marker tile, this means that the
corridor has made a turn i.e. if we place a corridor in the inner side of the
turn we would create a 2x2 so a blocker is placed instead.

The rule that the pathfinding algorithm cannot go from a marked tile to another
marked tile is a bit too restrictive since if we have two parallel corridors
which are placed two tiles away, then they would make a pocket of marker tiles
between them. This pocket however, should be "legal" to cross in the
perpendicular direction of the placed corridors.

When all corridors are carved or when the pathfinding algorithm fails to find a
path, the grid generation algorithm terminates, returning the result. The grid
is returned in the form of one dimensional array and an integer specifying the
width of the grid. This is done to avoid extra memory allocations. Going between
rows of the grid is done by jumping by the width of the grid to the left (north)
or to the right (south).

The behaviour of pathfinding algorithm can also be modified using the
configuration. In the configuration there are defined different costs for
different kinds of paths through the grid. The first one determines the cost of
going through a corridor, the second one determines the cost of going in a
straight line (which is checked by confirming whether the parent of the current
node is in the same direction - vertical or horizontal, as the currently
examined neighbor) and the third one is the standard cost (i.e. every other
case). The heuristic of the pathfinding algorithm uses the Manhattan distance
between the current tile and the target tile multiplied by the lowest cost,
which makes it admissible and consistent. These properties are helpful as the
priority queue used in the patfinding algorithm (implemented as a binary heap)
does not support updating the keys of the nodes. Due to the heuristic
duplicates, although not a problem since their presence can be detected, will
not be inserted in the priority queue.

More extreme cost values sometimes make the pathfinding algorithm fail.

TBA:
- regenerate dungeon with default settings, when failure to generate a
corridor occurs.
- maze generator in specific rooms.

### GUI application

The GUI application uses raylib and imgui to draw the generated dungeon and a
user interface to a window on the screen. Copies of the generated structures are
stored so as to allow visualisation of the different steps of the algorithm. The
application also utilises a second thread to generate the dungeons, since for
larger maps it takes a significant amount of time. If multithreading were not
used, the application would freeze the main thread when generating a
particularly large map and/or when using a configuration which causes a larger
number of and more complex corridors to be generated.

## Improvements

Improvements can be done to the triangulation algorithm as stated in the
Wikipedia algorithm

It should be possible to modify the behaviour of the pathfinding algorithm and
the placement of marker tiles so as to allow for two corridors to run in
parallel two tiles away and still be able to cross them perpendicularly.

## Source

The source for the complexity analysis is Wikipedia. Individual articles are
listed in the specification document.

## LLM Report

No LLMs were used in the making of this project.

