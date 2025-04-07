use super::*;

pub struct App {
  cursor_position: usize,
  editor_content: String,
  error: Option<Error>,
  renderer: Option<Renderer>,
  window: Option<Arc<Window>>,
}

impl App {
  pub fn new() -> Self {
    Self {
      cursor_position: 0,
      editor_content: String::new(),
      error: None,
      renderer: None,
      window: None,
    }
  }

  pub fn error(self) -> Option<Error> {
    self.error
  }

  fn resize(&mut self, new_size: PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
      if let Some(renderer) = &mut self.renderer {
        renderer.resize(new_size);
      }
    }
  }

  fn render(&mut self) -> Result {
    if let Some(renderer) = &mut self.renderer {
      renderer.render(&self.editor_content, self.cursor_position)?;
    }

    Ok(())
  }

  fn handle_keyboard_input(&mut self, key: Key, state: ElementState) {
    if state == ElementState::Pressed {
      match key {
        Key::Named(NamedKey::Backspace) => {
          if self.cursor_position > 0 {
            self.editor_content.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
          }
        }
        Key::Named(NamedKey::Delete) => {
          if self.cursor_position < self.editor_content.len() {
            self.editor_content.remove(self.cursor_position);
          }
        }
        Key::Named(NamedKey::ArrowLeft) => {
          if self.cursor_position > 0 {
            self.cursor_position -= 1;
          }
        }
        Key::Named(NamedKey::ArrowRight) => {
          if self.cursor_position < self.editor_content.len() {
            self.cursor_position += 1;
          }
        }
        Key::Named(NamedKey::Home) => {
          self.cursor_position = 0;
        }
        Key::Named(NamedKey::End) => {
          self.cursor_position = self.editor_content.len();
        }
        Key::Named(NamedKey::Enter) => {
          self.editor_content.insert(self.cursor_position, '\n');
          self.cursor_position += 1;
        }
        Key::Named(NamedKey::Space) => {
          self.editor_content.insert(self.cursor_position, ' ');
          self.cursor_position += 1;
        }
        Key::Character(c) => {
          self.editor_content.insert_str(self.cursor_position, &c);
          self.cursor_position += c.len();
        }
        _ => {}
      }
    }
  }
}

impl ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.window.is_none() {
      let window = match event_loop
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
        Ok(window) => Arc::new(window),
        Err(err) => {
          self.error = Some(err);
          event_loop.exit();
          return;
        }
      };

      let window_clone = window.clone();

      let future = async move { Renderer::new(window_clone).await };

      match pollster::block_on(future) {
        Ok(renderer) => {
          self.renderer = Some(renderer);
          self.window = Some(window);
        }
        Err(err) => {
          self.error = Some(err);
          event_loop.exit();
          return;
        }
      };

      if let Some(window) = &self.window {
        window.request_redraw();
      }
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
      WindowEvent::Resized(new_size) => {
        self.resize(new_size);
      }
      WindowEvent::KeyboardInput { event, .. } => {
        if event.state == ElementState::Pressed {
          match event.logical_key {
            Key::Named(NamedKey::Escape) => {
              event_loop.exit();
            }
            _ => {
              self.handle_keyboard_input(event.logical_key, event.state);

              if let Some(window) = &self.window {
                window.request_redraw();
              }
            }
          }
        } else {
          self.handle_keyboard_input(event.logical_key, event.state);
        }
      }
      WindowEvent::RedrawRequested => {
        match self.render() {
          Ok(_) => {}
          Err(e) => {
            self.error = Some(e);
            event_loop.exit();
          }
        }

        if let Some(window) = &self.window {
          window.request_redraw();
        }
      }
      _ => {}
    }
  }

  fn about_to_wait(&mut self, _: &ActiveEventLoop) {
    if let Some(window) = &self.window {
      window.request_redraw();
    }
  }
}
