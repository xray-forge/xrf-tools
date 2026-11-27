use std::collections::VecDeque;

/// The console's drawer: the last lines it was given, what is typed into it, and the lines entered, which the game
/// takes to run (`take_entered`) and answers by pushing lines back. Tab asks the game to complete the line
/// (`take_completing`, answered by `complete`); Up and Down walk back and forth through the lines entered before.
#[derive(Debug, Default)]
pub struct DeveloperConsole {
  lines: VecDeque<String>,
  input: String,
  entered: Vec<String>,
  /// The lines entered, oldest first, without one repeated straight after itself.
  history: VecDeque<String>,
  /// Which of them the field shows, none while it shows what is being typed.
  browsed: Option<usize>,
  /// The line Tab asked to complete, not yet taken.
  completing: Option<String>,
  /// Whether the field's cursor goes to its end at the next draw, after its text was replaced.
  is_cursor_reset: bool,
}

impl DeveloperConsole {
  /// How many lines it keeps, the oldest let go first.
  pub const LINES: usize = 200;

  /// How many entered lines Up walks back through.
  pub const HISTORY: usize = 100;

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

  /// The lines entered since the last take, in order, for the game to run.
  pub fn take_entered(&mut self) -> Vec<String> {
    std::mem::take(&mut self.entered)
  }

  /// The line Tab asked to complete since the last take, none where it was not pressed.
  pub fn take_completing(&mut self) -> Option<String> {
    self.completing.take()
  }

  /// Replaces the line being typed with `line`, as completed, the cursor at its end.
  pub fn complete(&mut self, line: String) {
    self.input = line;
    self.browsed = None;
    self.is_cursor_reset = true;
  }

  /// Draws it into `ui`: its lines in a scroll held to the bottom, then the field typed into, which a press of Enter
  /// submits, Tab completes and Up and Down fill from the lines entered before.
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

    // Taken before the field sees them: Tab would move the focus away, and the arrows are the history's.
    let [is_tab, is_up, is_down]: [bool; 3] = [egui::Key::Tab, egui::Key::ArrowUp, egui::Key::ArrowDown]
      .map(|key| ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, key)));

    if is_tab {
      self.completing = Some(self.input.clone());
    }

    if is_up {
      self.browse_back();
    } else if is_down {
      self.browse_forward();
    }

    let field = ui.add(
      egui::TextEdit::singleline(&mut self.input)
        .font(egui::TextStyle::Monospace)
        .desired_width(f32::INFINITY),
    );

    if field.changed() {
      self.browsed = None;
    }

    if std::mem::take(&mut self.is_cursor_reset)
      && let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), field.id)
    {
      let end = egui::text::CCursor::new(self.input.chars().count());

      state.cursor.set_char_range(Some(egui::text::CCursorRange::one(end)));
      state.store(ui.ctx(), field.id);
    }

    if field.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
      self.enter();
    }

    field.request_focus();
  }

  /// Hands the typed line to the game and keeps it in the history.
  fn enter(&mut self) {
    let entered: String = std::mem::take(&mut self.input);

    self.browsed = None;

    if entered.trim().is_empty() {
      return;
    }

    if self.history.back() != Some(&entered) {
      if self.history.len() == Self::HISTORY {
        self.history.pop_front();
      }

      self.history.push_back(entered.clone());
    }

    self.entered.push(entered);
  }

  /// Fills the field with the line entered before the one it shows, the last for none.
  fn browse_back(&mut self) {
    let Some(last) = self.history.len().checked_sub(1) else {
      return;
    };
    let index: usize = self.browsed.map_or(last, |index| index.saturating_sub(1));

    self.show_history(Some(index));
  }

  /// Fills the field with the line entered after the one it shows, empty past the last.
  fn browse_forward(&mut self) {
    let Some(index) = self.browsed else {
      return;
    };

    self.show_history((index + 1 < self.history.len()).then_some(index + 1));
  }

  fn show_history(&mut self, index: Option<usize>) {
    self.input = index
      .and_then(|index| self.history.get(index))
      .cloned()
      .unwrap_or_default();
    self.browsed = index;
    self.is_cursor_reset = true;
  }
}
