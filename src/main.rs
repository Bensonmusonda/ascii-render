use std::fs;

const WIDTH: usize = 80;
const HEIGHT: usize = 40;

// Character ramp from "far" to "near"
const SHADE_RAMP: [char; 5] = ['.', '-', '+', '#', '@'];


#[derive(Clone, Copy, Debug)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Clone, Copy, Debug)]
struct Point2D {
    x: i32,
    y: i32,
}

/// Create a blank grid filled with spaces.
fn new_grid() -> Vec<Vec<char>> {
    vec![vec![' '; WIDTH]; HEIGHT]
}

fn shade_char(z: f32, z_min: f32, z_max: f32) -> char {
    let range = (z_max - z_min).max(1e-6);
    let normalized = ((z - z_min) / range).clamp(0.0, 1.0);
    let idx = (normalized * (SHADE_RAMP.len() - 1) as f32).round() as usize;
    SHADE_RAMP[idx]
}

/// Draw a line between two 3D-projected points, shading by interpolated depth.
fn draw_line_shaded(
    grid: &mut Vec<Vec<char>>,
    a: Point2D,
    b: Point2D,
    z_a: f32,
    z_b: f32,
    z_min: f32,
    z_max: f32,
) {
    let steps = ((a.x - b.x).abs()).max((a.y - b.y).abs()).max(1);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let x = a.x as f32 + (b.x - a.x) as f32 * t;
        let y = a.y as f32 + (b.y - a.y) as f32 * t;
        let z = z_a + (z_b - z_a) * t;
        let (xi, yi) = (x.round() as i32, y.round() as i32);
        if xi >= 0 && xi < WIDTH as i32 && yi >= 0 && yi < HEIGHT as i32 {
            grid[yi as usize][xi as usize] = shade_char(z, z_min, z_max);
        }
    }
}
/// Print the grid to stdout, clearing the screen first.
fn render_grid(grid: &Vec<Vec<char>>) {
    print!("\x1B[2J\x1B[1;1H"); // clear screen, move cursor to top-left
    for row in grid {
        let line: String = row.iter().collect();
        println!("{}", line);
    }
}

/// The 8 vertices of a unit cube, centered at origin.
fn cube_vertices() -> Vec<Vec3> {
    vec![
        Vec3 { x: -1.0, y: -1.0, z: -1.0 },
        Vec3 { x:  1.0, y: -1.0, z: -1.0 },
        Vec3 { x:  1.0, y:  1.0, z: -1.0 },
        Vec3 { x: -1.0, y:  1.0, z: -1.0 },
        Vec3 { x: -1.0, y: -1.0, z:  1.0 },
        Vec3 { x:  1.0, y: -1.0, z:  1.0 },
        Vec3 { x:  1.0, y:  1.0, z:  1.0 },
        Vec3 { x: -1.0, y:  1.0, z:  1.0 },
    ]
}

/// Index pairs into cube_vertices() defining the 12 edges.
fn cube_edges() -> Vec<(usize, usize)> {
    vec![
        // back face
        (0, 1), (1, 2), (2, 3), (3, 0),
        // front face
        (4, 5), (5, 6), (6, 7), (7, 4),
        // connecting edges
        (0, 4), (1, 5), (2, 6), (3, 7),
    ]
}

/// Rotate a point around the Y axis by `angle` radians.
fn rotate_y(p: Vec3, angle: f32) -> Vec3 {
    let (s, c) = angle.sin_cos();
    Vec3 {
        x: p.x * c + p.z * s,
        y: p.y,
        z: -p.x * s + p.z * c,
    }
}

/// Rotate a point around the X axis by `angle` radians.
fn rotate_x(p: Vec3, angle: f32) -> Vec3 {
    let (s, c) = angle.sin_cos();
    Vec3 {
        x: p.x,
        y: p.y * c - p.z * s,
        z: p.y * s + p.z * c,
    }
}

fn project(p: Vec3, screen_width: i32, screen_height: i32) -> Point2D {
    // Scale factor controls how "big" the cube appears.
    // Terminal chars are taller than wide, so we scale x and y differently
    // to avoid a squashed-looking cube.
    let scale_x = screen_width as f32 / 4.0;
    let scale_y = screen_height as f32 / 4.0;

    let screen_x = (p.x * scale_x) as i32 + screen_width / 2;
    let screen_y = (-p.y * scale_y) as i32 + screen_height / 2; // flip y: screen y grows downward

    Point2D { x: screen_x, y: screen_y }
}


/// Load vertices and edges from a simple OBJ file.
/// Supports `v x y z` and `f i j k ...` (triangles or polygons, 1-based indices).
/// Ignores normals, UVs, materials, etc.
fn load_obj(path: &str) -> (Vec<Vec3>, Vec<(usize, usize)>) {
    let contents = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read OBJ file '{}': {}", path, e));

    let mut vertices = Vec::new();
    let mut edge_set = std::collections::HashSet::new();

    for line in contents.lines() {
        let line = line.trim();
        if line.starts_with("v ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            // parts[0] == "v", parts[1..4] == x y z
            let x: f32 = parts[1].parse().unwrap();
            let y: f32 = parts[2].parse().unwrap();
            let z: f32 = parts[3].parse().unwrap();
            vertices.push(Vec3 { x, y, z });
        } else if line.starts_with("f ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            // Each face part may look like "3", "3/1", or "3/1/2" (v/vt/vn) — take only the vertex index.
            let indices: Vec<usize> = parts[1..]
                .iter()
                .map(|p| {
                    let vertex_idx = p.split('/').next().unwrap();
                    vertex_idx.parse::<usize>().unwrap() - 1 // OBJ is 1-indexed
                })
                .collect();

            // Turn the face (triangle or polygon) into edges, deduped.
            let n = indices.len();
            for i in 0..n {
                let a = indices[i];
                let b = indices[(i + 1) % n];
                let edge = if a < b { (a, b) } else { (b, a) };
                edge_set.insert(edge);
            }
        }
    }

    let edges: Vec<(usize, usize)> = edge_set.into_iter().collect();
    (vertices, edges)
}

/// Center vertices around their bounding-box center and scale them to fit
/// within a standard [-1, 1] range, similar to our hardcoded cube.
fn normalize_vertices(vertices: &mut Vec<Vec3>) {
    if vertices.is_empty() {
        return;
    }

    // Find bounding box
    let mut min = vertices[0];
    let mut max = vertices[0];

    for v in vertices.iter() {
        min.x = min.x.min(v.x);
        min.y = min.y.min(v.y);
        min.z = min.z.min(v.z);
        max.x = max.x.max(v.x);
        max.y = max.y.max(v.y);
        max.z = max.z.max(v.z);
    }

    // Center of the bounding box
    let center = Vec3 {
        x: (min.x + max.x) / 2.0,
        y: (min.y + max.y) / 2.0,
        z: (min.z + max.z) / 2.0,
    };

    // Largest dimension across all axes, used as the scale reference
    let extent = ((max.x - min.x).max(max.y - min.y).max(max.z - min.z)).max(1e-6);
    let scale = 2.0 / extent; // fit into roughly [-1, 1]

    for v in vertices.iter_mut() {
        v.x = (v.x - center.x) * scale;
        v.y = (v.y - center.y) * scale;
        v.z = (v.z - center.z) * scale;
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let (mut vertices, edges) = if args.len() > 1 {
        load_obj(&args[1])
    } else {
        (cube_vertices(), cube_edges())
    };

    normalize_vertices(&mut vertices);

    let mut angle_x: f32 = 0.0;
    let mut angle_y: f32 = 0.0;

    loop {
        let mut grid = new_grid();

        let rotated: Vec<Vec3> = vertices
            .iter()
            .map(|v| rotate_y(rotate_x(*v, angle_x), angle_y))
            .collect();

        // NEW: compute this frame's depth range
        let z_values: Vec<f32> = rotated.iter().map(|v| v.z).collect();
        let z_min = z_values.iter().cloned().fold(f32::INFINITY, f32::min);
        let z_max = z_values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        let screen_points: Vec<Point2D> = rotated
            .iter()
            .map(|v| project(*v, WIDTH as i32, HEIGHT as i32))
            .collect();

        for &(i, j) in &edges {
            draw_line_shaded(
                &mut grid,
                screen_points[i],
                screen_points[j],
                rotated[i].z,
                rotated[j].z,
                z_min,
                z_max,
            );
        }

        render_grid(&grid);

        angle_x += 0.05;
        angle_y += 0.03;

        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}