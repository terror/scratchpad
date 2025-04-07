use super::*;

pub struct Renderer {
  config: SurfaceConfiguration,
  cursor_blink_timer: Instant,
  cursor_visible: bool,
  device: wgpu::Device,
  glyph_brush: GlyphBrush<()>,
  queue: wgpu::Queue,
  size: winit::dpi::PhysicalSize<u32>,
  staging_belt: wgpu::util::StagingBelt,
  surface: wgpu::Surface<'static>,
}

impl Renderer {
  pub async fn new(window: Arc<Window>) -> Result<Self> {
    let size = window.inner_size();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

    let surface = instance
      .create_surface(window.clone())
      .context(error::CreateSurface)?;

    let adapter = instance
      .request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
      })
      .await
      .ok_or(Error::internal("failed to get gpu adapter"))?;

    let (device, queue) = adapter
      .request_device(
        &wgpu::DeviceDescriptor {
          required_features: wgpu::Features::empty(),
          required_limits: wgpu::Limits::default(),
          label: Some(env!("CARGO_PKG_NAME")),
          memory_hints: wgpu::MemoryHints::default(),
        },
        None,
      )
      .await
      .context(error::Device)?;

    let surface_caps = surface.get_capabilities(&adapter);

    let format = surface_caps
      .formats
      .iter()
      .find(|f| f.is_srgb())
      .copied()
      .unwrap_or(surface_caps.formats[0]);

    let config = SurfaceConfiguration {
      alpha_mode: surface_caps.alpha_modes[0],
      desired_maximum_frame_latency: 2,
      format,
      height: size.height,
      present_mode: surface_caps.present_modes[0],
      usage: TextureUsages::RENDER_ATTACHMENT,
      view_formats: vec![],
      width: size.width,
    };

    surface.configure(&device, &config);

    let staging_belt = StagingBelt::new(1024);

    let font =
      FontArc::try_from_slice(include_bytes!("../assets/FiraCode-Regular.ttf"))
        .map_err(|error| {
          Error::internal(format!("failed to load font: {error}"))
        })?;

    let glyph_brush =
      GlyphBrushBuilder::using_font(font).build(&device, format);

    Ok(Self {
      config,
      cursor_blink_timer: Instant::now(),
      cursor_visible: true,
      device,
      glyph_brush,
      queue,
      size,
      staging_belt,
      surface,
    })
  }

  pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
      self.size = new_size;
      self.config.width = new_size.width;
      self.config.height = new_size.height;
      self.surface.configure(&self.device, &self.config);
    }
  }

  pub fn render(
    &mut self,
    text_content: &str,
    cursor_position: usize,
  ) -> Result {
    if self.cursor_blink_timer.elapsed() > Duration::from_millis(500) {
      self.cursor_visible = !self.cursor_visible;
      self.cursor_blink_timer = Instant::now();
    }

    let output = self
      .surface
      .get_current_texture()
      .context(error::CurrentTexture)?;

    let view = output
      .texture
      .create_view(&TextureViewDescriptor::default());

    let mut encoder =
      self
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
          label: Some("Render Encoder"),
        });

    encoder.begin_render_pass(&RenderPassDescriptor {
      label: Some("Clear Pass"),
      color_attachments: &[Some(RenderPassColorAttachment {
        view: &view,
        resolve_target: None,
        ops: Operations {
          load: LoadOp::Clear(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
          }),
          store: StoreOp::Store,
        },
      })],
      depth_stencil_attachment: None,
      timestamp_writes: None,
      occlusion_query_set: None,
    });

    let text_before_cursor = &text_content[0..cursor_position];

    let font_size = 32.0;

    let (x_margin, y_margin) = (30.0, 40.0);

    self.glyph_brush.queue(Section {
      screen_position: (x_margin, y_margin),
      bounds: (self.size.width as f32, self.size.height as f32),
      text: vec![
        Text::new(text_content)
          .with_color([0.0, 0.0, 0.0, 1.0])
          .with_scale(font_size),
      ],
      ..Section::default()
    });

    self
      .glyph_brush
      .draw_queued(
        &self.device,
        &mut self.staging_belt,
        &mut encoder,
        &view,
        self.size.width,
        self.size.height,
      )
      .map_err(|e| Error::internal(format!("Failed to render text: {}", e)))?;

    if self.cursor_visible {
      let char_width = 15.2;

      let cursor_x = x_margin + (text_before_cursor.len() as f32 * char_width);

      self.glyph_brush.queue(Section {
        screen_position: (cursor_x, y_margin),
        bounds: (self.size.width as f32, self.size.height as f32),
        text: vec![
          Text::new("|")
            .with_color([0.0, 0.0, 0.0, 1.0])
            .with_scale(font_size),
        ],
        ..Section::default()
      });

      self
        .glyph_brush
        .draw_queued(
          &self.device,
          &mut self.staging_belt,
          &mut encoder,
          &view,
          self.size.width,
          self.size.height,
        )
        .map_err(|e| {
          Error::internal(format!("Failed to render cursor: {}", e))
        })?;
    }

    self.staging_belt.finish();

    self.queue.submit(std::iter::once(encoder.finish()));

    output.present();

    self.staging_belt.recall();

    Ok(())
  }
}
