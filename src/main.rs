// src/main.rs

#[derive(Clone, Copy, Debug)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
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

fn main() {
    let vertices = cube_vertices();
    let edges = cube_edges();

    // Sanity check for now — Step 3/4 will replace this with real rendering.
    let angle_x: f32 = 0.4;
    let angle_y: f32 = 0.6;

    for (i, v) in vertices.iter().enumerate() {
        let rotated = rotate_y(rotate_x(*v, angle_x), angle_y);
        println!("v{}: {:?} -> {:?}", i, v, rotated);
    }

    println!("\nEdges: {:?}", edges);
}