use std::collections::VecDeque;

/// The console: the last lines it was given and what is typed into it. It knows no commands yet: a line entered is
/// echoed and answered as unknown, until the console's command registry exists.
#[derive(Debug, Default)]
pub struct DeveloperConsole {
  lines: VecDeque<String>,
  input: String,
}

impl DeveloperConsole {
  /// How many lines it keeps, the oldest let go first.
  pub const LINES: usize = 200;

  /// Adds a line at the bottom.
  pub fn push(&mut self, line: impl Into<String>) {
    if self.lines.len() == Self::LINES {
      self.lines.pop_front();
    }

    self.lines.push_back(line.into());
  }

  /// The lines it holds, oldest first.
  pub fn get_lines(&self) -> impl Iterator<Item = &str> {
    self.lines.iter().map(String::as_str)
  }

  /// Draws it into `ui`: its lines in a scroll held to the bottom, then the field typed into, which a press of Enter
  /// submits.
  pub(crate) fn show(&mut self, ui: &mut egui::Ui) {
    let field_height: f32 = ui.spacing().interact_size.y + ui.spacing().item_spacing.y * 2.0;

    egui::ScrollArea::vertical()
      .max_height(ui.available_height() - field_height)
      .stick_to_bottom(true)
      .auto_shrink([false, false])
      .show(ui, |ui| {
        for line in &self.lines {
          ui.monospace(line);
        }
      });

    let field = ui.add(
      egui::TextEdit::singleline(&mut self.input)
        .font(egui::TextStyle::Monospace)
        .desired_width(f32::INFINITY),
    );

    if field.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
      let entered: String = std::mem::take(&mut self.input);

      if !entered.trim().is_empty() {
        self.push(format!("> {entered}"));
        self.push(format!("Unknown command: {}", entered.trim()));
      }
    }

    field.request_focus();
  }
}
