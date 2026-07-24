# ASCII 3D Renderer

A terminal-based 3D wireframe renderer written in Rust. Loads OBJ models (or
falls back to a built-in cube), rotates them in real time, and renders them
as animated ASCII art with depth-based shading — no external crates required.

## Features

- Pure `std` implementation — no dependencies
- OBJ file loading (vertices + faces, supports `v`/`f` lines and `v/vt/vn` indices)
- Automatic bounding-box normalization so models of any scale/origin render
  centered and consistently sized
- Real-time rotation around X and Y axes
- Orthographic projection to a terminal character grid
- Per-frame depth shading using a character ramp (`.` `-` `+` `#` `@`) that
  normalizes contrast to each frame's actual depth range

## Usage

Run with the built-in cube:
```bash
cargo run
```

Run with your own OBJ file:
```bash
cargo run -- path/to/model.obj
```

Press `Ctrl+C` to stop.

## How it works

1. **Load** — vertices and edges are read from an OBJ file (faces are
   decomposed into deduplicated edges), or a hardcoded cube is used as a
   fallback.
2. **Normalize** — the model is centered on its bounding-box midpoint and
   scaled to fit roughly within `[-1, 1]`, so any model renders at a
   consistent size regardless of its original units or origin offset.
3. **Rotate** — each frame, vertices are rotated around the X and Y axes
   using standard rotation matrices.
4. **Project** — rotated 3D points are projected orthographically onto a 2D
   character grid sized to the terminal.
5. **Shade** — edges are drawn between projected points using linear
   interpolation, with depth (z) interpolated along each line and mapped to
   a character ramp. The depth range is recomputed each frame from the
   rotated model, so contrast stays strong regardless of the model's scale.
6. **Render** — the character grid is printed to stdout each frame, with the
   screen cleared between frames via ANSI escape codes.

## Known limitations

- Wireframe only — no face filling or backface culling yet, so dense meshes
  can look silhouette-like from some angles
- No perspective projection (orthographic only)
- No color (ANSI color support could be a future addition)

## Possible next steps

- Backface culling for cleaner reads on closed meshes
- Perspective projection
- Keyboard-controlled rotation instead of continuous auto-spin
- ANSI color output