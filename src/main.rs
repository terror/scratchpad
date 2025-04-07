use {
  crate::{app::App, error::Error, renderer::Renderer},
  ropey::Rope,
  snafu::{Backtrace, ErrorCompat, ResultExt, Snafu},
  std::{
    sync::Arc,
    time::{Duration, Instant},
  },
  wgpu::{
    Color, LoadOp, Operations, PowerPreference, RenderPassColorAttachment,
    RenderPassDescriptor, RequestAdapterOptions, StoreOp, SurfaceConfiguration,
    TextureUsages, TextureViewDescriptor, util::StagingBelt,
  },
  wgpu_glyph::{
    GlyphBrush, GlyphBrushBuilder, Section, Text, ab_glyph::FontArc,
  },
  winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes, WindowId},
  },
};

mod app;
mod error;
mod renderer;

type Result<T = (), E = Error> = std::result::Result<T, E>;

fn run() -> Result {
  let event_loop = EventLoop::with_user_event()
    .build()
    .context(error::EventLoopBuild)?;

  let mut app = App::new();

  event_loop.run_app(&mut app).context(error::RunApp)?;

  if let Some(error) = app.error() {
    return Err(error);
  }

  Ok(())
}

fn main() {
  env_logger::init();

  if let Err(err) = run() {
    eprintln!("error: {err}");

    for (i, err) in err.iter_chain().skip(1).enumerate() {
      if i == 0 {
        eprintln!();
        eprintln!("because:");
      }

      eprintln!("- {err}");
    }

    if let Some(backtrace) = err.backtrace() {
      if backtrace.status() == std::backtrace::BacktraceStatus::Captured {
        eprintln!("backtrace:");
        eprintln!("{backtrace}");
      }
    }

    std::process::exit(1);
  }
}
