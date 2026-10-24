use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2]
}
impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub fn create_quad_vertices(x: f32, y: f32, w: f32, h: f32, screen_width: f32, screen_height: f32) -> (Vec<Vertex>, Vec<u16>) {
    // Piksel koordinatlarını -1 ile +1 arasına (NDC) çeviriyoruz:
    let x0 = (x / screen_width) * 2.0 - 1.0;
    let y0 = 1.0 - (y / screen_height) * 2.0;
    let x1 = ((x + w) / screen_width) * 2.0 - 1.0;
    let y1 = 1.0 - ((y + h) / screen_height) * 2.0;

    let vertices = vec![
        Vertex { position: [x0, y0], tex_coords: [0.0, 0.0] }, // Sol Üst
        Vertex { position: [x0, y1], tex_coords: [0.0, 1.0] }, // Sol Alt
        Vertex { position: [x1, y1], tex_coords: [1.0, 1.0] }, // Sağ Alt
        Vertex { position: [x1, y0], tex_coords: [1.0, 0.0] }, // Sağ Üst
    ];

    // İki üçgenden oluşan standart bir quad index dizisi
    let indices = vec![0, 1, 2, 0, 2, 3];

    (vertices, indices)
}