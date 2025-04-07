use super::*;

pub struct App {
  cursor_position: usize,
  editor_content: Rope,
  error: Option<Error>,
  renderer: Option<Renderer>,
  window: Option<Arc<Window>>,
}

impl App {
  pub fn new() -> Self {
    Self {
      cursor_position: 0,
      editor_content: Rope::new(),
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
      let text_content = self.editor_content.to_string();
      renderer.render(&text_content, self.cursor_position)?;
    }

    Ok(())
  }

  fn handle_keyboard_input(&mut self, key: Key, state: ElementState) {
    if state == ElementState::Pressed {
      match key {
        Key::Named(NamedKey::Backspace) => {
          if self.cursor_position > 0 {
            self
              .editor_content
              .remove(self.cursor_position - 1..self.cursor_position);
            self.cursor_position -= 1;
          }
        }
        Key::Named(NamedKey::Delete) => {
          if self.cursor_position < self.editor_content.len_chars() {
            self
              .editor_content
              .remove(self.cursor_position..self.cursor_position + 1);
          }
        }
        Key::Named(NamedKey::ArrowLeft) => {
          if self.cursor_position > 0 {
            self.cursor_position -= 1;
          }
        }
        Key::Named(NamedKey::ArrowRight) => {
          if self.cursor_position < self.editor_content.len_chars() {
            self.cursor_position += 1;
          }
        }
        Key::Named(NamedKey::Home) => {
          self.cursor_position = 0;
        }
        Key::Named(NamedKey::End) => {
          self.cursor_position = self.editor_content.len_chars();
        }
        Key::Named(NamedKey::Enter) => {
          self.editor_content.insert(self.cursor_position, "\n");
          self.cursor_position += 1;
        }
        Key::Named(NamedKey::Space) => {
          self.editor_content.insert(self.cursor_position, " ");
          self.cursor_position += 1;
        }
        Key::Character(c) => {
          self.editor_content.insert(self.cursor_position, &c);
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn insert_character() {
    let mut app = App::new();

    app
      .handle_keyboard_input(Key::Character("a".into()), ElementState::Pressed);

    assert_eq!(app.editor_content.to_string(), "a");
    assert_eq!(app.cursor_position, 1);

    app
      .handle_keyboard_input(Key::Character("b".into()), ElementState::Pressed);

    assert_eq!(app.editor_content.to_string(), "ab");
    assert_eq!(app.cursor_position, 2);
  }

  #[test]
  fn backspace() {
    let mut app = App::new();

    app
      .handle_keyboard_input(Key::Character("a".into()), ElementState::Pressed);

    app
      .handle_keyboard_input(Key::Character("b".into()), ElementState::Pressed);

    app.handle_keyboard_input(
      Key::Named(NamedKey::Backspace),
      ElementState::Pressed,
    );

    assert_eq!(app.editor_content.to_string(), "a");
    assert_eq!(app.cursor_position, 1);
  }

  #[test]
  fn delete_character() {
    let mut app = App::new();

    app
      .handle_keyboard_input(Key::Character("a".into()), ElementState::Pressed);

    app
      .handle_keyboard_input(Key::Character("b".into()), ElementState::Pressed);

    app.handle_keyboard_input(
      Key::Named(NamedKey::ArrowLeft),
      ElementState::Pressed,
    );

    app.handle_keyboard_input(
      Key::Named(NamedKey::Delete),
      ElementState::Pressed,
    );

    assert_eq!(app.editor_content.to_string(), "a");
    assert_eq!(app.cursor_position, 1);
  }

  #[test]
  fn cursor_movement() {
    let mut app = App::new();

    app
      .handle_keyboard_input(Key::Character("a".into()), ElementState::Pressed);

    app
      .handle_keyboard_input(Key::Character("b".into()), ElementState::Pressed);

    app
      .handle_keyboard_input(Key::Character("c".into()), ElementState::Pressed);

    app.handle_keyboard_input(
      Key::Named(NamedKey::ArrowLeft),
      ElementState::Pressed,
    );
    app.handle_keyboard_input(
      Key::Named(NamedKey::ArrowLeft),
      ElementState::Pressed,
    );

    assert_eq!(app.cursor_position, 1);

    app.handle_keyboard_input(
      Key::Named(NamedKey::ArrowRight),
      ElementState::Pressed,
    );

    assert_eq!(app.cursor_position, 2);
  }

  #[test]
  fn home_end_keys() {
    let mut app = App::new();

    app
      .handle_keyboard_input(Key::Character("a".into()), ElementState::Pressed);

    app
      .handle_keyboard_input(Key::Character("b".into()), ElementState::Pressed);

    app
      .handle_keyboard_input(Key::Character("c".into()), ElementState::Pressed);

    app
      .handle_keyboard_input(Key::Named(NamedKey::Home), ElementState::Pressed);

    assert_eq!(app.cursor_position, 0);

    app.handle_keyboard_input(Key::Named(NamedKey::End), ElementState::Pressed);

    assert_eq!(app.cursor_position, 3);
  }

  #[test]
  fn enter_key() {
    let mut app = App::new();

    app
      .handle_keyboard_input(Key::Character("a".into()), ElementState::Pressed);

    app.handle_keyboard_input(
      Key::Named(NamedKey::Enter),
      ElementState::Pressed,
    );

    app
      .handle_keyboard_input(Key::Character("b".into()), ElementState::Pressed);

    assert_eq!(app.editor_content.to_string(), "a\nb");
    assert_eq!(app.cursor_position, 3);
  }

  #[test]
  fn space_key() {
    let mut app = App::new();

    app
      .handle_keyboard_input(Key::Character("a".into()), ElementState::Pressed);

    app.handle_keyboard_input(
      Key::Named(NamedKey::Space),
      ElementState::Pressed,
    );

    app
      .handle_keyboard_input(Key::Character("b".into()), ElementState::Pressed);

    assert_eq!(app.editor_content.to_string(), "a b");
    assert_eq!(app.cursor_position, 3);
  }

  #[test]
  fn insert_at_cursor_position() {
    let mut app = App::new();

    app
      .handle_keyboard_input(Key::Character("a".into()), ElementState::Pressed);

    app
      .handle_keyboard_input(Key::Character("c".into()), ElementState::Pressed);

    app.handle_keyboard_input(
      Key::Named(NamedKey::ArrowLeft),
      ElementState::Pressed,
    );

    app
      .handle_keyboard_input(Key::Character("b".into()), ElementState::Pressed);

    assert_eq!(app.editor_content.to_string(), "abc");
    assert_eq!(app.cursor_position, 2);
  }

  #[test]
  fn multiple_characters_deletion() {
    let mut app = App::new();

    app
      .handle_keyboard_input(Key::Character("a".into()), ElementState::Pressed);

    app
      .handle_keyboard_input(Key::Character("b".into()), ElementState::Pressed);

    app
      .handle_keyboard_input(Key::Character("c".into()), ElementState::Pressed);

    app
      .handle_keyboard_input(Key::Character("d".into()), ElementState::Pressed);

    app
      .handle_keyboard_input(Key::Character("e".into()), ElementState::Pressed);

    app
      .handle_keyboard_input(Key::Character("f".into()), ElementState::Pressed);

    app.handle_keyboard_input(
      Key::Named(NamedKey::Backspace),
      ElementState::Pressed,
    );

    app.handle_keyboard_input(
      Key::Named(NamedKey::Backspace),
      ElementState::Pressed,
    );

    app.handle_keyboard_input(
      Key::Named(NamedKey::Backspace),
      ElementState::Pressed,
    );

    assert_eq!(app.editor_content.to_string(), "abc");
    assert_eq!(app.cursor_position, 3);
  }

  #[test]
  fn boundary_conditions() {
    let mut app = App::new();

    app.handle_keyboard_input(
      Key::Named(NamedKey::Backspace),
      ElementState::Pressed,
    );

    assert_eq!(app.editor_content.to_string(), "");
    assert_eq!(app.cursor_position, 0);

    app.handle_keyboard_input(
      Key::Named(NamedKey::Delete),
      ElementState::Pressed,
    );

    assert_eq!(app.editor_content.to_string(), "");
    assert_eq!(app.cursor_position, 0);

    app.handle_keyboard_input(
      Key::Named(NamedKey::ArrowLeft),
      ElementState::Pressed,
    );

    assert_eq!(app.cursor_position, 0);

    app
      .handle_keyboard_input(Key::Character("a".into()), ElementState::Pressed);

    app.handle_keyboard_input(
      Key::Named(NamedKey::ArrowRight),
      ElementState::Pressed,
    );

    assert_eq!(app.cursor_position, 1);

    app.handle_keyboard_input(
      Key::Named(NamedKey::ArrowRight),
      ElementState::Pressed,
    );

    assert_eq!(app.cursor_position, 1);
  }

  #[test]
  fn insert_multi_char_string() {
    let mut app = App::new();

    app.handle_keyboard_input(
      Key::Character("hello".into()),
      ElementState::Pressed,
    );

    assert_eq!(app.editor_content.to_string(), "hello");
    assert_eq!(app.cursor_position, 5);
  }
}
