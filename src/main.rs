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

fn shade_char(z: f32) -> char {
    // Closer to camera (more negative z, since camera looks down +z) = brighter.
    // Clamp and normalize z into [0.0, 1.0]
    let normalized = ((z + 2.0) / 4.0).clamp(0.0, 1.0);
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
) {
    let steps = ((a.x - b.x).abs()).max((a.y - b.y).abs()).max(1);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let x = a.x as f32 + (b.x - a.x) as f32 * t;
        let y = a.y as f32 + (b.y - a.y) as f32 * t;
        let z = z_a + (z_b - z_a) * t;
        let (xi, yi) = (x.round() as i32, y.round() as i32);
        if xi >= 0 && xi < WIDTH as i32 && yi >= 0 && yi < HEIGHT as i32 {
            grid[yi as usize][xi as usize] = shade_char(z);
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

fn main() {
    let vertices = cube_vertices();
    let edges = cube_edges();

    let mut angle_x: f32 = 0.0;
    let mut angle_y: f32 = 0.0;

    loop {
        let mut grid = new_grid();

        // Rotate all vertices, keep both the 2D projection and the z for shading
        let rotated: Vec<Vec3> = vertices
            .iter()
            .map(|v| rotate_y(rotate_x(*v, angle_x), angle_y))
            .collect();

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
            );
        }

        render_grid(&grid);

        angle_x += 0.05;
        angle_y += 0.03;

        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}