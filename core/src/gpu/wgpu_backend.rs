//! WGPU-based GPU renderer for fractals
//!
//! This module implements GPU-accelerated fractal rendering using WGPU compute shaders.
//!
//! # Shader Composition Architecture
//!
//! Shaders are built from two parts at load time:
//! - `common.wgsl` - Shared infrastructure (FractalParams struct, bindings, coordinate
//!   mapping, complex math utilities, and the compute entry point)
//! - `*_kernel.wgsl` - Per-fractal iteration logic (must define `iterate_fractal()`)
//!
//! The common template contains a `// {{FRACTAL_KERNEL}}` marker that gets replaced
//! with the kernel source. This eliminates duplication of coordinate mapping, complex
//! math, and dispatch boilerplate across fractals.
//!
//! # Tiled Rendering
//!
//! For large exports that exceed GPU buffer limits (typically 256 MB for most GPUs),
//! the renderer automatically uses tiled rendering:
//!
//! 1. Image is split into tiles that fit within GPU buffer limits
//! 2. Each tile is rendered independently on the GPU
//! 3. Tiles are assembled into a complete iteration count buffer
//! 4. Filtering (if enabled) is applied to the complete assembled image
//! 5. Colormap is applied to produce the final RGBA image

use super::{FractalRenderer, RenderConfig, GPU_MAX_SAFE_ITERATIONS};
use crate::fractals::Fractal;
use crate::perf_log;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

/// Unified GPU parameters for all fractal shaders (must match WGSL FractalParams layout)
///
/// Per-fractal data is passed through param_0/param_1/param_2 slots (order matches parameters()):
/// - Mandelbrot/Powerbrot: param_0 = power (default 2.0)
/// - Insideout Dragon: param_0 = escape_radius (default 4.0)
/// - Julia Set: param_0 = c_real, param_1 = c_imag, param_2 = power (default 2.0)
/// - Zubieta: param_0 = c_real, param_1 = c_imag
/// - Sin Julia: param_0 = c_real, param_1 = c_imag, param_2 = escape_radius (default 50.0)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuFractalParams {
    center_x: f32,
    center_y: f32,
    zoom: f32,
    max_iter: u32,
    width: u32,
    height: u32,
    param_0: f32,
    param_1: f32,
    param_2: f32,
    _padding: [u32; 3], // Ensure 16-byte alignment
}

/// GPU parameters for orbit accumulation shaders (must match WGSL OrbitParams layout).
///
/// Layout: center, zoom, dimensions, orbit params, padding, then map data arrays.
/// WGSL `array<f32, 8>` has stride 4 and alignment 4, so no inter-element padding.
/// Total: 12 + 4(pad) + 8*3 = 40 f32s = 160 bytes. Round to 16-byte alignment = 160.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuOrbitParams {
    center_x: f32,
    center_y: f32,
    zoom: f32,
    width: u32,
    height: u32,
    samples_per_thread: u32,
    burn_in: u32,
    seed_base: u32,
    num_maps: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
    // 8 map c_real values
    map_re: [f32; 8],
    // 8 map c_imag values
    map_im: [f32; 8],
    // 8 cumulative probabilities
    cum_prob: [f32; 8],
}

/// WGPU-based GPU renderer
pub struct WgpuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipelines: HashMap<String, ComputePipeline>,
    max_buffer_size: u64,
}

struct ComputePipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl WgpuRenderer {
    /// Calculate optimal tile size that fits within GPU buffer limits
    ///
    /// Returns (tile_width, tile_height) that will fit within max_buffer_size
    fn calculate_tile_size(&self) -> (u32, u32) {
        // Each pixel needs 4 bytes (u32 for iteration count)
        let bytes_per_pixel = std::mem::size_of::<u32>() as u64;
        
        // Leave 10% headroom for other buffers (params, staging, etc.)
        let usable_buffer_size = (self.max_buffer_size as f64 * 0.9) as u64;
        
        // Calculate max pixels per tile
        let max_pixels = usable_buffer_size / bytes_per_pixel;
        
        // Use square tiles for simplicity (easier to manage aspect ratio)
        let tile_size = (max_pixels as f64).sqrt() as u32;
        
        // Round down to multiple of 16 for workgroup alignment
        let tile_size = (tile_size / 16) * 16;
        
        (tile_size, tile_size)
    }
    
    /// Compose a complete WGSL shader from the common template + a fractal kernel
    ///
    /// The common template (common.wgsl) contains a `// {{FRACTAL_KERNEL}}` marker
    /// that is replaced with the fractal-specific iteration logic.
    fn compose_shader(kernel_source: &str) -> String {
        let common = include_str!("shaders/common.wgsl");
        common.replace("// {{FRACTAL_KERNEL}}", kernel_source)
    }
    
    /// Load and compile a fractal compute shader by name
    ///
    /// Composes the common template with the given kernel source,
    /// creates the shader module, bind group layout, and compute pipeline.
    fn load_shader(&mut self, name: &str, kernel_source: &str) -> Result<(), String> {
        let full_source = Self::compose_shader(kernel_source);
        
        let label_shader = format!("{} Compute Shader", name);
        let label_layout = format!("{} Bind Group Layout", name);
        let label_pipeline_layout = format!("{} Pipeline Layout", name);
        let label_pipeline = format!("{} Compute Pipeline", name);
        
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&label_shader),
            source: wgpu::ShaderSource::Wgsl(full_source.into()),
        });
        
        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&label_layout),
            entries: &[
                // Uniform buffer for parameters
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Storage buffer for output
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&label_pipeline_layout),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let pipeline = self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(&label_pipeline),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });
        
        self.pipelines.insert(
            name.to_string(),
            ComputePipeline {
                pipeline,
                bind_group_layout,
            },
        );
        
        Ok(())
    }
    
    /// Render a fractal on GPU (single tile)
    ///
    /// Generic render method that works for any loaded fractal shader.
    /// If the image exceeds GPU buffer limits, automatically delegates to tiled rendering.
    fn render_fractal(&self, pipeline_name: &str, config: &RenderConfig) -> Result<Vec<u32>, String> {
        let pipeline = self.pipelines.get(pipeline_name)
            .ok_or_else(|| format!("{} shader not loaded", pipeline_name))?;
        
        // Safety: clamp iterations to prevent TDR (Windows GPU timeout)
        let effective_max_iter = if config.max_iter > GPU_MAX_SAFE_ITERATIONS {
            eprintln!(
                "[GPU-SAFETY] Clamping iterations from {} to {} to avoid GPU timeout (TDR). \
                 Use CPU mode for higher iterations.",
                config.max_iter, GPU_MAX_SAFE_ITERATIONS
            );
            GPU_MAX_SAFE_ITERATIONS
        } else {
            config.max_iter
        };
        
        // Check buffer size against device limits - if too large, use tiled rendering
        let output_size = (config.width * config.height) as usize;
        let required_buffer_size = (output_size * std::mem::size_of::<u32>()) as u64;
        
        if required_buffer_size > self.max_buffer_size {
            return self.render_fractal_tiled(pipeline_name, config);
        }
        
        // Build unified params from config
        let params = GpuFractalParams {
            center_x: config.center_x as f32,
            center_y: config.center_y as f32,
            zoom: config.zoom as f32,
            max_iter: effective_max_iter,
            width: config.width,
            height: config.height,
            param_0: config.fractal_params.first().copied().unwrap_or(0.0) as f32,
            param_1: config.fractal_params.get(1).copied().unwrap_or(0.0) as f32,            param_2: config.fractal_params.get(2).copied().unwrap_or(0.0) as f32,
            _padding: [0; 3],        };
        
        let label_params = format!("{} Params Buffer", pipeline_name);
        let label_bind = format!("{} Bind Group", pipeline_name);
        let label_pass = format!("{} Compute Pass", pipeline_name);
        
        // Create uniform buffer for parameters
        let params_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&label_params),
            contents: bytemuck::cast_slice(&[params]),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        
        // Create output buffer
        let output_buffer_size = (output_size * std::mem::size_of::<u32>()) as u64;
        perf_log!("[GPU-MEM] Render {}x{}: output={:.1} MB, staging={:.1} MB (total GPU alloc ~{:.1} MB)",
            config.width, config.height,
            output_buffer_size as f64 / (1024.0 * 1024.0),
            output_buffer_size as f64 / (1024.0 * 1024.0),
            2.0 * output_buffer_size as f64 / (1024.0 * 1024.0));

        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: output_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        
        // Create staging buffer for reading results
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: output_buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create bind group
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&label_bind),
            layout: &pipeline.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: output_buffer.as_entire_binding(),
                },
            ],
        });
        
        // Encode and submit compute pass
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Compute Encoder"),
        });
        
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some(&label_pass),
                timestamp_writes: None,
            });
            
            compute_pass.set_pipeline(&pipeline.pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            
            // Dispatch workgroups (16x16 threads per workgroup)
            let workgroup_size = 16;
            let workgroups_x = (config.width + workgroup_size - 1) / workgroup_size;
            let workgroups_y = (config.height + workgroup_size - 1) / workgroup_size;
            compute_pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
        }
        
        // Copy output to staging buffer
        encoder.copy_buffer_to_buffer(
            &output_buffer,
            0,
            &staging_buffer,
            0,
            output_buffer_size,
        );
        
        self.queue.submit(Some(encoder.finish()));
        
        // Read results from staging buffer
        // Wrap in catch_unwind to handle GPU device lost (TDR timeout) gracefully
        // instead of crashing the entire application
        let read_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let buffer_slice = staging_buffer.slice(..);
            let (sender, receiver) = std::sync::mpsc::channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                let _ = sender.send(result);
            });
            
            // Poll device until buffer is ready
            self.device.poll(wgpu::Maintain::Wait);
            
            let map_result = receiver.recv()
                .map_err(|e| format!("Failed to receive buffer mapping: {}", e))?;
            map_result.map_err(|e| format!("Failed to map buffer: {:?}", e))?;
            
            // Copy data from mapped buffer
            let data = buffer_slice.get_mapped_range();
            let result: Vec<u32> = bytemuck::cast_slice(&data).to_vec();
            
            drop(data);
            staging_buffer.unmap();
            
            Ok(result)
        }));
        
        match read_result {
            Ok(inner_result) => inner_result,
            Err(panic_info) => {
                let msg = if let Some(s) = panic_info.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                    s.to_string()
                } else {
                    "Unknown GPU panic".to_string()
                };
                Err(format!(
                    "GPU device lost (likely TDR timeout). Iterations ({}) may be too high for GPU. \
                     Falling back to CPU. Error: {}",
                    config.max_iter, msg
                ))
            }
        }
    }
    
    /// Render a fractal using tiled approach for large images
    ///
    /// Splits the render into tiles that fit within GPU buffer limits,
    /// renders each tile with render_fractal(), then assembles them.
    fn render_fractal_tiled(&self, pipeline_name: &str, config: &RenderConfig) -> Result<Vec<u32>, String> {
        let (tile_width, tile_height) = self.calculate_tile_size();
        
        // Calculate number of tiles needed
        let tiles_x = (config.width + tile_width - 1) / tile_width;
        let tiles_y = (config.height + tile_height - 1) / tile_height;
        let total_tiles = tiles_x * tiles_y;
        
        perf_log!("[GPU-TILE] Rendering {}x{} image with {} tiles ({}x{} tiles, each {}x{} pixels)",
            config.width, config.height, total_tiles, tiles_x, tiles_y, tile_width, tile_height);
        
        // Allocate complete output buffer
        let output_size = (config.width * config.height) as usize;
        let mut complete_buffer = vec![0u32; output_size];
        
        // Calculate zoom scale for tiles (what portion of the view each tile represents)
        let zoom_scale_x = config.width as f64 / tile_width as f64;
        let zoom_scale_y = config.height as f64 / tile_height as f64;
        
        // Render each tile
        for tile_y in 0..tiles_y {
            for tile_x in 0..tiles_x {
                // Calculate tile dimensions (may be smaller at edges)
                let this_tile_width = if tile_x == tiles_x - 1 {
                    config.width - tile_x * tile_width
                } else {
                    tile_width
                };
                let this_tile_height = if tile_y == tiles_y - 1 {
                    config.height - tile_y * tile_height
                } else {
                    tile_height
                };
                
                // Calculate the center coordinates for this tile in complex plane
                let pixel_width = 1.0 / (config.zoom * config.width as f64);
                let pixel_height = 1.0 / (config.zoom * config.height as f64);
                
                // Calculate offset from image center (in pixels)
                let tile_center_x_pixel = (tile_x * tile_width + this_tile_width / 2) as f64 
                                        - (config.width as f64 / 2.0);
                let tile_center_y_pixel = (tile_y * tile_height + this_tile_height / 2) as f64 
                                        - (config.height as f64 / 2.0);
                
                // Convert pixel offset to complex plane offset
                let offset_x = tile_center_x_pixel * pixel_width;
                let offset_y = tile_center_y_pixel * pixel_height;
                
                // Tile center in complex plane
                let tile_center_x = config.center_x + offset_x;
                let tile_center_y = config.center_y + offset_y;
                
                // Calculate zoom for this tile
                let tile_zoom_x = config.zoom * zoom_scale_x;
                let tile_zoom_y = config.zoom * zoom_scale_y;
                let tile_zoom = tile_zoom_x.max(tile_zoom_y);
                
                // Create config for this tile
                let tile_config = RenderConfig {
                    center_x: tile_center_x,
                    center_y: tile_center_y,
                    zoom: tile_zoom,
                    max_iter: config.max_iter,
                    width: this_tile_width,
                    height: this_tile_height,
                    fractal_params: config.fractal_params.clone(),
                };
                
                // Render this tile
                let tile_buffer = self.render_fractal(pipeline_name, &tile_config)?;
                
                // Copy tile data into complete buffer
                for ty in 0..this_tile_height {
                    for tx in 0..this_tile_width {
                        let tile_idx = (ty * this_tile_width + tx) as usize;
                        let output_x = tile_x * tile_width + tx;
                        let output_y = tile_y * tile_height + ty;
                        let output_idx = (output_y * config.width + output_x) as usize;
                        complete_buffer[output_idx] = tile_buffer[tile_idx];
                    }
                }
                
                // Progress update
                let tiles_done = tile_y * tiles_x + tile_x + 1;
                if tiles_done % 10 == 0 || tiles_done == total_tiles {
                    perf_log!("[GPU-TILE] Rendered {}/{} tiles ({:.1}%)", 
                        tiles_done, total_tiles, 
                        100.0 * tiles_done as f64 / total_tiles as f64);
                }
            }
        }
        
        perf_log!("[GPU-TILE] Tile rendering complete, assembled {}x{} iteration buffer", 
            config.width, config.height);
        
        Ok(complete_buffer)
    }
    
    /// Compose a complete orbit accumulation WGSL shader
    fn compose_orbit_shader(kernel_source: &str) -> String {
        let common = include_str!("shaders/orbit_common.wgsl");
        common.replace("// {{ORBIT_KERNEL}}", kernel_source)
    }

    /// Load and compile an orbit accumulation compute shader by name
    fn load_orbit_shader(&mut self, name: &str, kernel_source: &str) -> Result<(), String> {
        let full_source = Self::compose_orbit_shader(kernel_source);

        let label_shader = format!("{} Orbit Shader", name);
        let label_layout = format!("{} Orbit Bind Group Layout", name);
        let label_pipeline_layout = format!("{} Orbit Pipeline Layout", name);
        let label_pipeline = format!("{} Orbit Compute Pipeline", name);

        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&label_shader),
            source: wgpu::ShaderSource::Wgsl(full_source.into()),
        });

        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&label_layout),
            entries: &[
                // Storage buffer (read-only) for orbit params.
                // Using storage rather than uniform avoids the WGSL requirement
                // that array<f32, N> elements have 16-byte stride in uniform space.
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Storage buffer for atomic density histogram
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&label_pipeline_layout),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(&label_pipeline),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        self.pipelines.insert(
            name.to_string(),
            ComputePipeline {
                pipeline,
                bind_group_layout,
            },
        );

        Ok(())
    }

    /// Render orbit density on GPU, returning raw u32 counts (NOT normalized).
    ///
    /// The caller is responsible for normalization and coloring.
    fn render_orbit_density_internal(
        &self,
        width: u32,
        height: u32,
        params: &GpuOrbitParams,
        pipeline_name: &str,
    ) -> Result<Vec<u32>, String> {
        let cp = self.pipelines.get(pipeline_name)
            .ok_or_else(|| format!("No orbit pipeline '{}'", pipeline_name))?;

        let pixel_count = (width as u64) * (height as u64);
        let buffer_bytes = pixel_count * 4; // u32 per pixel

        if buffer_bytes > self.max_buffer_size {
            return Err(format!(
                "Orbit density buffer ({} MB) exceeds GPU limit ({} MB). Reduce resolution.",
                buffer_bytes / (1024 * 1024),
                self.max_buffer_size / (1024 * 1024),
            ));
        }

        // Uniform buffer
        let uniform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Orbit Params Buffer"),
            contents: bytemuck::bytes_of(params),
            usage: wgpu::BufferUsages::STORAGE,
        });

        // Density storage buffer (zero-initialized)
        let density_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Orbit Density Buffer"),
            size: buffer_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Staging buffer for readback
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Orbit Staging Buffer"),
            size: buffer_bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Bind group
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Orbit Bind Group"),
            layout: &cp.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: density_buffer.as_entire_binding(),
                },
            ],
        });

        // Encode and dispatch — 1D, one thread per sub-orbit
        // workgroup_size(64) means we need ceil(K / 64) workgroups
        // K = number of sub-orbits, passed via dispatch count
        // We dispatch exactly 1 workgroup of 64 threads = 64 sub-orbits
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Orbit Compute Encoder"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Orbit Compute Pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&cp.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(1, 1, 1); // 1 workgroup of 64 threads
        }

        // Copy density → staging
        encoder.copy_buffer_to_buffer(&density_buffer, 0, &staging_buffer, 0, buffer_bytes);
        self.queue.submit(std::iter::once(encoder.finish()));

        // Read back
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let buffer_slice = staging_buffer.slice(..);
            let (tx, rx) = std::sync::mpsc::channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                tx.send(result).ok();
            });
            self.device.poll(wgpu::Maintain::Wait);

            rx.recv()
                .map_err(|e| format!("GPU readback channel error: {}", e))?
                .map_err(|e| format!("GPU buffer map failed: {:?}", e))?;

            let data = buffer_slice.get_mapped_range();
            let result: Vec<u32> = bytemuck::cast_slice(&data).to_vec();
            drop(data);
            staging_buffer.unmap();
            Ok(result)
        }));

        match result {
            Ok(Ok(data)) => Ok(data),
            Ok(Err(e)) => Err(e),
            Err(_) => Err("GPU orbit render panicked (possible TDR timeout)".to_string()),
        }
    }

    /// Map a fractal name to its GPU pipeline name
    ///
    /// Multiple fractal names can map to the same pipeline (e.g., Mandelbrot and Powerbrot
    /// share the same shader since power is a parameter).
    fn pipeline_name_for(fractal_name: &str) -> Option<&'static str> {
        match fractal_name {
            "Mandelbrot" | "Powerbrot" => Some("Mandelbrot"),
            "Julia Set" => Some("Julia Set"),
            "Insideout Dragon" => Some("Insideout Dragon"),
            "Zubieta" => Some("Zubieta"),
            "Sin Julia" => Some("Sin Julia"),
            "Burning Ship" => Some("Burning Ship"),
            "Tippets Mandelbrot" => Some("Tippets Mandelbrot"),
            "Multifractal-Julia" => Some("Multifractal-Julia"),
            "Cactus" => Some("Cactus"),
            _ => None,
        }
    }

    /// Map a fractal name to its GPU orbit pipeline name
    fn orbit_pipeline_name_for(fractal_name: &str) -> Option<&'static str> {
        match fractal_name {
            "Multi-Julia IFS"  => Some("Multi-Julia IFS Orbit"),
            "ChaosSymmetry1"   => Some("ChaosSymmetry1 Orbit"),
            _ => None,
        }
    }
    
    /// Create a new GPU renderer
    ///
    /// Returns None if GPU initialization fails (no compatible adapter, etc.)
    pub fn new() -> Result<Self, String> {
        // Use pollster to block on async GPU initialization
        pollster::block_on(Self::new_async())
    }
    
    async fn new_async() -> Result<Self, String> {
        // Create WGPU instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        // Request adapter (GPU device)
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| "Failed to find compatible GPU adapter".to_string())?;
        
        // Get device info for logging
        let info = adapter.get_info();
        println!("GPU Renderer initialized: {} ({:?})", info.name, info.backend);
        
        // Get adapter limits - we'll use these to request a capable device
        let adapter_limits = adapter.limits();
        
        // Request device with adapter's limits to get maximum capabilities
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Fractal Compute Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: adapter_limits.clone(),
                },
                None,
            )
            .await
            .map_err(|e| format!("Failed to create GPU device: {}", e))?;
        
        // Get the actual device limits (may be clamped from what we requested)
        let device_limits = device.limits();
        
        println!("GPU max buffer size: {} MB", device_limits.max_buffer_size / (1024 * 1024));
        
        let mut renderer = Self {
            device,
            queue,
            pipelines: HashMap::new(),
            max_buffer_size: device_limits.max_buffer_size,
        };
        
        // Pre-compile all fractal shaders using kernel composition.
        // Each load_shader() call composes the kernel with common.wgsl and calls
        // device.create_compute_pipeline(), which is the expensive step.
        // Timings are printed when --profiling is active.
        let shader_compile_start = std::time::Instant::now();

        macro_rules! load_timed {
            ($renderer:expr, $name:expr, $src:expr) => {{
                let t = std::time::Instant::now();
                $renderer.load_shader($name, $src)?;
                perf_log!("[GPU-INIT] Compiled shader '{}': {:.2?}", $name, t.elapsed());
            }};
        }

        load_timed!(renderer, "Mandelbrot",          include_str!("shaders/mandelbrot_kernel.wgsl"));
        load_timed!(renderer, "Julia Set",            include_str!("shaders/julia_kernel.wgsl"));
        load_timed!(renderer, "Insideout Dragon",     include_str!("shaders/insideout_dragon_kernel.wgsl"));
        load_timed!(renderer, "Zubieta",              include_str!("shaders/zubieta_kernel.wgsl"));
        load_timed!(renderer, "Sin Julia",            include_str!("shaders/sin_julia_kernel.wgsl"));
        load_timed!(renderer, "Burning Ship",         include_str!("shaders/burning_ship_kernel.wgsl"));
        load_timed!(renderer, "Tippets Mandelbrot",   include_str!("shaders/tippets_mandelbrot_kernel.wgsl"));
        load_timed!(renderer, "Multifractal-Julia",   include_str!("shaders/multifractal_julia_kernel.wgsl"));
        load_timed!(renderer, "Cactus",               include_str!("shaders/cactus_kernel.wgsl"));

        // Orbit accumulation shaders (different composition template)
        {
            let t = std::time::Instant::now();
            renderer.load_orbit_shader("Multi-Julia IFS Orbit", include_str!("shaders/multi_julia_ifs_orbit_kernel.wgsl"))?;
            perf_log!("[GPU-INIT] Compiled orbit shader 'Multi-Julia IFS Orbit': {:.2?}", t.elapsed());
        }
        {
            let t = std::time::Instant::now();
            renderer.load_orbit_shader("ChaosSymmetry1 Orbit", include_str!("shaders/chaos_symmetry1_orbit_kernel.wgsl"))?;
            perf_log!("[GPU-INIT] Compiled orbit shader 'ChaosSymmetry1 Orbit': {:.2?}", t.elapsed());
        }

        let total_compile = shader_compile_start.elapsed();
        println!("[GPU-INIT] All {} shaders compiled in {:.2?}",
            renderer.pipelines.len(), total_compile);

        Ok(renderer)
    }
}

impl FractalRenderer for WgpuRenderer {
    fn render_iterations(
        &mut self,
        config: &RenderConfig,
        fractal: &dyn Fractal,
    ) -> Result<Vec<u32>, String> {
        let fractal_name = fractal.name();
        
        let pipeline_name = Self::pipeline_name_for(fractal_name)
            .ok_or_else(|| format!("GPU rendering not supported for fractal: {}", fractal_name))?;
        
        self.render_fractal(pipeline_name, config)
    }
    
    fn supports_fractal(&self, fractal_name: &str) -> bool {
        Self::pipeline_name_for(fractal_name).is_some()
            || Self::orbit_pipeline_name_for(fractal_name).is_some()
    }

    fn supports_orbit_density(&self, fractal_name: &str) -> bool {
        Self::orbit_pipeline_name_for(fractal_name).is_some()
    }

    fn render_orbit_density(
        &self,
        width: u32,
        height: u32,
        fractal_params: &std::collections::HashMap<String, f64>,
        fractal_name: &str,
        center_x: f32,
        center_y: f32,
        zoom: f32,
    ) -> Result<Vec<u32>, String> {
        let pipeline_name = Self::orbit_pipeline_name_for(fractal_name)
            .ok_or_else(|| format!("No GPU orbit shader for '{}'", fractal_name))?;

        let gpu_params = if fractal_name == "ChaosSymmetry1" {
            // ── ChaosSymmetry1 parameter mapping ──────────────────────────────────
            // Repurposes OrbitParams fields:  map_re[0..4] = a0..a4,  num_maps = m
            let total_samples = fractal_params.get("samples").copied().unwrap_or(5_000_000.0) as u64;
            let burn_in  = fractal_params.get("burn_in").copied().unwrap_or(1_000.0) as u32;
            let seed     = fractal_params.get("seed").copied().unwrap_or(0.0) as u32;
            let m        = fractal_params.get("m").copied().unwrap_or(3.0) as u32;
            let a0       = fractal_params.get("a0").copied().unwrap_or(1.5)  as f32;
            let a1       = fractal_params.get("a1").copied().unwrap_or(-1.5) as f32;
            let a2       = fractal_params.get("a2").copied().unwrap_or(0.0)  as f32;
            let a3       = fractal_params.get("a3").copied().unwrap_or(0.0)  as f32;
            let a4       = fractal_params.get("a4").copied().unwrap_or(0.5)  as f32;
            GpuOrbitParams {
                center_x,
                center_y,
                zoom,
                width,
                height,
                samples_per_thread: (total_samples / 64) as u32,
                burn_in,
                seed_base: if seed == 0 { 0xDEAD_BEEF } else { seed },
                num_maps: m.max(2),
                _pad0: 0,
                _pad1: 0,
                _pad2: 0,
                map_re: [a0, a1, a2, a3, a4, 0.0, 0.0, 0.0],
                map_im: [0.0; 8],
                cum_prob: [0.0; 8],
            }
        } else {
            // ── Multi-Julia IFS (and any future IFS-type orbit fractals) ──────────
            let num_maps = fractal_params.get("num_attractors").copied().unwrap_or(2.0) as u32;
            let total_samples = fractal_params.get("samples").copied().unwrap_or(5_000_000.0) as u64;
            let burn_in = fractal_params.get("burn_in").copied().unwrap_or(50.0) as u32;
            let seed = fractal_params.get("seed").copied().unwrap_or(0.0) as u32;
            let samples_per_thread = (total_samples / 64) as u32;

            let mut map_re = [0.0f32; 8];
            let mut map_im = [0.0f32; 8];
            let mut cum_prob = [0.0f32; 8];

            // Build map arrays and cumulative probabilities
            let mut total_weight = 0.0f64;
            for i in 0..num_maps.min(8) as usize {
                map_re[i] = fractal_params.get(&format!("c{}_real", i)).copied().unwrap_or(0.0) as f32;
                map_im[i] = fractal_params.get(&format!("c{}_imag", i)).copied().unwrap_or(0.0) as f32;
                let prob = fractal_params.get(&format!("prob{}", i)).copied().unwrap_or(1.0);
                total_weight += prob;
            }
            // Normalize to cumulative
            let mut running = 0.0f64;
            for i in 0..num_maps.min(8) as usize {
                let prob = fractal_params.get(&format!("prob{}", i)).copied().unwrap_or(1.0);
                running += prob / total_weight;
                cum_prob[i] = running as f32;
            }
            if num_maps > 0 {
                cum_prob[(num_maps - 1).min(7) as usize] = 1.0; // ensure exact 1.0
            }

            GpuOrbitParams {
                center_x,
                center_y,
                zoom,
                width,
                height,
                samples_per_thread,
                burn_in,
                seed_base: if seed == 0 { 0xDEADBEEF } else { seed },
                num_maps,
                _pad0: 0,
                _pad1: 0,
                _pad2: 0,
                map_re,
                map_im,
                cum_prob,
            }
        };

        self.render_orbit_density_internal(width, height, &gpu_params, pipeline_name)
    }
}
