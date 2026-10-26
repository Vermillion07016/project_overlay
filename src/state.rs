use std::{collections::HashMap, os::raw::c_void};
use anyhow::Result;
use project_overlay::{Vertex, create_quad_vertices};
use wgpu::{Adapter, Backends, BindGroup, BindGroupLayout, BlendState, Color, ColorTargetState, ColorWrites, CommandEncoderDescriptor, CurrentSurfaceTexture, Device, DeviceDescriptor, ExperimentalFeatures, Face, Features, FragmentState, FrontFace, Instance, InstanceDescriptor, Limits, MultisampleState, Operations, PolygonMode, PresentMode, PrimitiveState, PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RequestAdapterOptions, ShaderModule, StoreOp, Surface, SurfaceColorSpace, SurfaceConfiguration, TextureFormat, TextureSampleType, TextureUsages, TextureViewDescriptor, TextureViewDimension, VertexState, util::DeviceExt};
use windows::{core::Interface, Win32::{Foundation::HWND, Graphics::{Dxgi::IDXGIDevice, DirectComposition::{DCompositionCreateDevice, IDCompositionDevice, IDCompositionTarget, IDCompositionVisual}}}};
use windows_sys::Win32::Foundation::RECT;

use crate::{sprite::Sprite, texture::Texture};

pub struct State {
    surface_config: SurfaceConfiguration,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    render_pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: BindGroupLayout,
    textures: HashMap<String, (Texture, BindGroup)>,
    sprites: Vec<Sprite>,
    _dcomp: IDCompositionDevice,
    _dcomp_target: IDCompositionTarget,
    _dcomp_visual: IDCompositionVisual,
}
impl State {
    pub fn new(hwnd: *const c_void) -> Result<Self> {
        println!("\n[INIT] DX12 Grafik Alt Sistemi Başlatılıyor...");
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::DX12,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: wgpu::BackendOptions {
                dx12: wgpu::Dx12BackendOptions {
                    presentation_system: wgpu::Dx12SwapchainKind::DxgiFromVisual,
                    ..Default::default()
                },
                ..Default::default()
            },
            display: None
        }); 

        let dcomp: IDCompositionDevice = unsafe { DCompositionCreateDevice(None::<&IDXGIDevice>)? };
        let dcomp_target = unsafe { dcomp.CreateTargetForHwnd(HWND(hwnd as *mut c_void), true)? };
        let dcomp_visual = unsafe { dcomp.CreateVisual()? };
        unsafe { dcomp_target.SetRoot(&dcomp_visual)? };

        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::CompositionVisual(dcomp_visual.as_raw()))?
        };
        let adapter = pollster::block_on(async {
            instance.request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: true,
            }).await
        }).map_err(|_| anyhow::anyhow!("Uygun bir DX12 Adaptörü (GPU) bulunamadı!"))?;

        let info = adapter.get_info();
        println!("  ├─ [GPU Seçildi] : {} (Tür: {:?})", info.name, info.device_type);
        println!("  ├─ [Sürücü]      : {}", info.driver);

        let (device, queue) = pollster::block_on(async {
            adapter.request_device(&DeviceDescriptor {
                label: Some("Overlay Main Device"),
                required_features: Features::empty(),
                experimental_features: ExperimentalFeatures::disabled(),
                required_limits: Limits::defaults(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            }).await
        })?;
        println!("  ├─ [Logical Device & Queue] Başarıyla oluşturuldu.");

        let (shader, config, surface_format) = Self::create_surface_configs(&device, &surface, &adapter);
        let texture_bind_group_layout = Self::create_diffuse_things(&device)?;

        let render_pipeline = Self::create_pipeline(&device, &texture_bind_group_layout, &shader, &config);
        surface.configure(&device, &config);
        unsafe { dcomp.Commit()? };

        println!("[SUCCESS] DX12 altyapısı tamamen ayağa kalktı.\n");

        Ok(Self { 
            surface, device, queue, render_pipeline, 
            surface_config: config, textures: HashMap::new(), 
            texture_bind_group_layout, sprites: Vec::new(),
            _dcomp: dcomp, _dcomp_target: dcomp_target, _dcomp_visual: dcomp_visual
        })
    }
    fn create_pipeline(device: &Device, texture_bind_group_layout: &BindGroupLayout, shader: &ShaderModule, config: &SurfaceConfiguration) -> RenderPipeline {
        let render_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[Some(texture_bind_group_layout)],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor { 
            label: Some("Render Pipeline"), 
            layout: Some(&render_pipeline_layout), 
            vertex: VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(Vertex::desc())],
                compilation_options: Default::default()
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default()
            }),
            primitive: PrimitiveState { 
                topology: PrimitiveTopology::TriangleList, 
                strip_index_format: None, 
                front_face: FrontFace::Ccw, 
                cull_mode: Some(Face::Back), 
                unclipped_depth: false, 
                polygon_mode: PolygonMode::Fill, 
                conservative: false 
            }, 
            depth_stencil: None, 
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false
            }, 
            multiview_mask: None, 
            cache: None 
        });

        render_pipeline
    }
    fn create_surface_configs(device: &Device, surface: &Surface, adapter: &Adapter) -> (ShaderModule, SurfaceConfiguration, TextureFormat) {
        let surface_caps = surface.get_capabilities(adapter);
        println!("  ├─ [Alpha Modları] : {:?}", surface_caps.alpha_modes);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: 1920,
            height: 1080,
            present_mode: surface_caps.present_modes.iter().find(|mode| **mode == PresentMode::Fifo).map(|a| *a).unwrap_or(surface_caps.present_modes[0]),
            alpha_mode: surface_caps.alpha_modes.iter().copied().find(|m| *m == wgpu::CompositeAlphaMode::PreMultiplied).unwrap_or(surface_caps.alpha_modes[0]),
            view_formats: Vec::new(),
            desired_maximum_frame_latency: 2,
            color_space: SurfaceColorSpace::Auto,
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        
        (shader, config, surface_format)
    }
    fn create_diffuse_things(device: &Device) -> Result<BindGroupLayout> {
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { 
            label: Some("texture_bind_group_layout"), 
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { 
                        sample_type: TextureSampleType::Float { filterable: true }, 
                        view_dimension: TextureViewDimension::D2, 
                        multisampled: false 
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(
                        wgpu::SamplerBindingType::Filtering
                    ),
                    count: None
                }
            ] 
        });

        Ok(texture_bind_group_layout)
    }
    fn load_texture(&mut self, name: &str, bytes: &[u8]) -> Result<()> {
        let texture = Texture::from_bytes(&self.device, &self.queue, bytes, name)?;

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{}_bind_group", name)),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        });
        self.textures.insert(name.to_owned(), (texture, bind_group));
        Ok(())
    }
    pub fn add_sprite(&mut self, name: &str, x: f32, y: f32, w: f32, h: f32, bytes: &[u8]) -> Result<()> {
        let (vertices, indices) = create_quad_vertices(
            x, y, w, h, 
            self.surface_config.width as f32, 
            self.surface_config.height as f32
        );

        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        self.sprites.push(Sprite {
            texture_name: name.to_string(), width: w, height: h,
            x, y, vertex_buffer, index_buffer,
        });

        self.load_texture(name, bytes)?;
        Ok(())
    }
    pub fn resize(&mut self, rect: RECT) {
        let (w, h) = ((rect.right - rect.left) as u32, (rect.bottom - rect.top) as u32);

        if w > 0 && h > 0 {
            self.surface_config.width = w;
            self.surface_config.height = h;
            self.surface.configure(&self.device, &self.surface_config);

            let sw = w as f32;
            let sh = h as f32;

            for sprite in &mut self.sprites {
                let (vertices, _indices) = create_quad_vertices(
                    sprite.x, sprite.y, sprite.width, sprite.height, sw, sh
                );

                sprite.vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Resized Sprite Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
            }
        }
    }
    pub fn render(&mut self) -> Result<(), &'static str> {
        let output = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(texture) => texture,
            CurrentSurfaceTexture::Suboptimal(texture) => texture,
            CurrentSurfaceTexture::Timeout |
            CurrentSurfaceTexture::Occluded |
            CurrentSurfaceTexture::Validation => return Ok(()),
            CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.surface_config);
                return Ok(());
            },
            CurrentSurfaceTexture::Lost => {
                eprintln!("Lost device");
                std::process::exit(-1);
            }
        };

        let view = output.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });
        
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Clear(Color { 
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0 
                        }),
                        store: StoreOp::Store
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None
            });

            render_pass.set_pipeline(&self.render_pipeline);
            for sprite in &self.sprites {
                if let Some((_, bind_group)) = self.textures.get(&sprite.texture_name) {
                    render_pass.set_bind_group(0, bind_group, &[]);
                } else {

                }

                render_pass.set_vertex_buffer(0, sprite.vertex_buffer.slice(..));
                render_pass.set_index_buffer(sprite.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..6,0,0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(output);

        Ok(())
    }
}