
# Dungen

## Overview

This is a 2D dungeon generator written in the programming language Rust. The
program will receive as an input settings and features related to the generation
of the dungeon such as the size of the map. The output of the program is a 2D
map of walls and empty cells which will constitute rooms and corridors between
those rooms.

Since this is not a core feature, the output will be presented in an ASCII
text format.

## Algorithms

The core of the dungeon generation is creating rectangular rooms aligned to the
axis of the grid and connecting them with corridors. Chosing which rooms will be
connected is done based on a Delaunay triangulation of points on the rooms using
Bowyer-Watson's algorithm. After the triangulation a Minimum Spanning Tree will
be constructed using the Disjoint Set Union version of Kruskal's algorithm. This
tree will then be modified by reintroducing randomly edges from the
triangulation. A* will be used to create the corridors between the rooms. The
priority queue in A* will be binary heap.

List of possible features that may be added to the dungeon after generation:
- Mazes in parts of rooms which are of a given size through iterative randomized
Kruskal.

# Course info

I am part of the Bachelor's Programme in Science. I have sufficient knowledge to
peer-review projects written in Python, C# and by extension, most likely Java
and C/C++.

# Sources

- [Bowyer-Watson's algorithm](https://en.wikipedia.org/wiki/Bowyer%E2%80%93Watson_algorithm)
- [A* algorithm](https://en.wikipedia.org/wiki/A*_search_algorithm)
- [Kruskal's algorithm](https://en.wikipedia.org/wiki/Kruskal%27s_algorithm)

