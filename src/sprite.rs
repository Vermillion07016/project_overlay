pub struct Sprite {
    pub texture_name: String, // Hangi texture kullanılacak? (örn: "ikon.png")
    pub x: f32,               // Ekrana göre X pikseli
    pub y: f32,               // Ekrana göre Y pikseli
    pub width: f32,  // Sprite'ın kendi genişliği
    pub height: f32, // Sprite'ın kendi yüksekliği
    pub vertex_buffer: wgpu::Buffer, // Bu sprite'a özel vertex buffer
    pub index_buffer: wgpu::Buffer,   // Bu sprite'a özel index buffer
}