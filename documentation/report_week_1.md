
# Report Week 1
- Chose a topic (generating dungeons) and researched algorithms (0.75h).
- Created the repository and initialized several of the required documentation
files. (0.75h)
- Tested how test coverage works in Rust. (0.5h)
- Wrote a procedure which checks whether a point is in the circumcircle of a
given triangle. (0.75h)
- Wrote the room generation procedures. (0.75h)
- Wrote the triangulation procedure and visualisation. (4h)

Queries:
- I'd like to know whether the libraries I am planning to use are allowed. They
can be found in the file Cargo.toml under the section **dependencies**.
- While observing the generated triangulation, I found out that the total
polygon is not always convex. I can fix that by first generating a convex hull
and trivially triangulating that (instead of using a super triangle) and then
performing the iterative addition of the triangles. My question is whether this
is worth it.

