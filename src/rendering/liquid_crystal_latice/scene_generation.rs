use crate::rendering::*;

fn generate_rectangle_vertices(
    index: u32,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    center_x: f32,
    center_y: f32,
) -> Vec<OrbitingVertex> {
    vec![
        // First triangle of the rectangle
        OrbitingVertex {
            vertex: Vertex {
                position: [x0, y0, 0.0],
                tex_coords: [0.0, 0.0],
            },
            center: [center_x, center_y],
            index,
        }, // Bottom-left
        OrbitingVertex {
            vertex: Vertex {
                position: [x1, y0, 0.0],
                tex_coords: [1.0, 0.0],
            },
            center: [center_x, center_y],
            index,
        }, // Bottom-right
        OrbitingVertex {
            vertex: Vertex {
                position: [x0, y1, 0.0],
                tex_coords: [0.0, 1.0],
            },
            center: [center_x, center_y],
            index,
        }, // Top-left
        // Second triangle of the rectangle
        OrbitingVertex {
            vertex: Vertex {
                position: [x0, y1, 0.0],
                tex_coords: [0.0, 1.0],
            },
            center: [center_x, center_y],
            index,
        }, // Top-left
        OrbitingVertex {
            vertex: Vertex {
                position: [x1, y0, 0.0],
                tex_coords: [1.0, 0.0],
            },
            center: [center_x, center_y],
            index,
        }, // Bottom-right
        OrbitingVertex {
            vertex: Vertex {
                position: [x1, y1, 0.0],
                tex_coords: [1.0, 1.0],
            },
            center: [center_x, center_y],
            index,
        }, // Top-right
    ]
}

pub fn generate_vertex_buffer(rows: usize, cols: usize) -> Vec<OrbitingVertex> {
    let mut vertices = Vec::with_capacity(rows * cols * 6);

    let row_step = 2.0 / rows as f32;
    let col_step = 2.0 / cols as f32;
    let spacing_x = 0.0; //col_step / 3.0;
    let spacing_y = 0.0; //row_step / 3.0;
    let mut index = 0;

    for row in 0..rows {
        let y0 = -1.0 + row as f32 * row_step + spacing_y / 2.0;
        let y1 = y0 + row_step - spacing_y;
        let center_y = (y0 + y1) / 2.0;

        for col in 0..cols {
            let x0 = -1.0 + col as f32 * col_step + spacing_x / 2.0;
            let x1 = x0 + col_step - spacing_x;
            let center_x = (x0 + x1) / 2.0;

            vertices.extend(generate_rectangle_vertices(
                index, x0, y0, x1, y1, center_x, center_y,
            ));
            index += 1;
        }
    }

    vertices
}
