use super::*;

pub struct App {
  window: Option<Window>,
  error: Option<Error>,
}

impl App {
  pub fn new() -> Self {
    Self {
      window: None,
      error: None,
    }
  }

  pub fn error(self) -> Option<Error> {
    self.error
  }
}

impl ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.window.is_none() {
      self.window = match event_loop
        .create_window(
          WindowAttributes::default()
            .with_inner_size(PhysicalSize {
              width: 1600,
              height: 1200,
            })
            .with_min_inner_size(PhysicalSize {
              width: 800,
              height: 600,
            })
            .with_title(env!("CARGO_PKG_NAME")),
        )
        .context(error::CreateWindow)
      {
        Ok(window) => Some(window),
        Err(err) => {
          self.error = Some(err);
          event_loop.exit();
          return;
        }
      };
    }
  }

  fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    _id: WindowId,
    event: WindowEvent,
  ) {
    match event {
      WindowEvent::CloseRequested => {
        event_loop.exit();
      }
      WindowEvent::KeyboardInput { event, .. }
        if event.state == ElementState::Pressed =>
      {
        match event.logical_key {
          Key::Named(NamedKey::Escape) => {
            event_loop.exit();
          }
          _ => {}
        }
      }
      WindowEvent::RedrawRequested => {
        if let Some(window) = &self.window {
          window.request_redraw();
        }
      }
      _ => {}
    }
  }
}
