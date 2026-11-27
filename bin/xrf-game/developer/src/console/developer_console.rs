use std::collections::VecDeque;

use crate::console::developer_suggestion::DeveloperSuggestion;

/// The console's drawer: the last lines it was given, what is typed into it, and the lines entered, which the game
/// takes to run (`take_entered`) and answers by pushing lines back.
///
/// As a line is typed the game is told (`take_typed`) and answers with suggestions (`suggest`), listed in a popup under
/// the field, each with a hint. Up and Down move through them while they show, and through the lines entered before
/// while they do not or a line recalled shows; Tab takes the one picked, or asks the game to complete the line as far as it can
/// (`take_completing`, answered by `complete`).
#[derive(Debug, Default)]
pub struct DeveloperConsole {
  lines: VecDeque<String>,
  input: String,
  entered: Vec<String>,
  /// The lines entered, oldest first, without one repeated straight after itself.
  history: VecDeque<String>,
  /// Which of them the field shows, none while it shows what is being typed.
  browsed: Option<usize>,
  /// What the line being typed may become, as the game last suggested.
  suggestions: Vec<DeveloperSuggestion>,
  /// Which suggestion is picked, none until Up or Down picks one.
  picked: Option<usize>,
  /// Whether the line changed since the game was last told.
  is_typed: bool,
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

  /// How many suggestions the popup shows at once, scrolled to the one picked.
  pub const SHOWN_SUGGESTIONS: usize = 12;

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

  /// The line being typed, where it changed since the last take, for the game to suggest from.
  pub fn take_typed(&mut self) -> Option<String> {
    std::mem::take(&mut self.is_typed).then(|| self.input.clone())
  }

  /// Lists `suggestions` under the field, none picked; none hides the popup.
  pub fn suggest(&mut self, suggestions: Vec<DeveloperSuggestion>) {
    self.suggestions = suggestions;
    self.picked = None;
  }

  /// The line Tab asked to complete since the last take, none where it was not pressed.
  pub fn take_completing(&mut self) -> Option<String> {
    self.completing.take()
  }

  /// Replaces the line being typed with `line`, as completed, the cursor at its end.
  pub fn complete(&mut self, line: String) {
    self.replace_input(line);
    self.browsed = None;
  }

  /// Draws it into `ui`: its lines in a scroll held to the bottom, then the field typed into, which a press of Enter
  /// submits, and under it the suggestions.
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

    // Taken before the field sees them: Tab would move the focus away, and the arrows are the popup's or the history's.
    let [is_tab, is_up, is_down]: [bool; 3] = [egui::Key::Tab, egui::Key::ArrowUp, egui::Key::ArrowDown]
      .map(|key| ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, key)));

    if is_tab {
      match self.picked {
        Some(picked) => self.take_suggestion(picked),
        None => self.completing = Some(self.input.clone()),
      }
    }

    // Walking the history keeps the arrows, though a line recalled brings its own suggestions.
    if !self.suggestions.is_empty() && self.browsed.is_none() {
      if is_up {
        self.pick(-1);
      } else if is_down {
        self.pick(1);
      }
    } else if is_up {
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
      self.is_typed = true;
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

    if let Some(clicked) = self.show_suggestions(ui.ctx(), field.rect) {
      self.take_suggestion(clicked);
    }
  }

  /// Lists the suggestions in a popup under `field`, a window of them around the one picked, answering the one clicked.
  fn show_suggestions(&self, context: &egui::Context, field: egui::Rect) -> Option<usize> {
    if self.suggestions.is_empty() {
      return None;
    }

    let start: usize = self
      .picked
      .map_or(0, |picked| (picked + 1).saturating_sub(Self::SHOWN_SUGGESTIONS));
    let end: usize = (start + Self::SHOWN_SUGGESTIONS).min(self.suggestions.len());
    let mut clicked: Option<usize> = None;

    egui::Area::new(egui::Id::unique("developer_suggestions"))
      .order(egui::Order::Foreground)
      .fixed_pos(field.left_bottom() + egui::vec2(0.0, 4.0))
      .fade_in(false)
      .show(context, |ui| {
        egui::Frame::new()
          .fill(egui::Color32::from_black_alpha(235))
          .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(70)))
          .inner_margin(6.0)
          .corner_radius(3.0)
          .show(ui, |ui| {
            egui::Grid::new("developer_suggestion_rows")
              .num_columns(2)
              .spacing(egui::vec2(16.0, 2.0))
              .show(ui, |ui| {
                for (index, suggestion) in self.suggestions.iter().enumerate().take(end).skip(start) {
                  let name = egui::RichText::new(&suggestion.text).monospace();

                  if ui.selectable_label(self.picked == Some(index), name).clicked() {
                    clicked = Some(index);
                  }

                  ui.label(
                    egui::RichText::new(&suggestion.hint)
                      .monospace()
                      .color(egui::Color32::from_gray(140)),
                  );
                  ui.end_row();
                }
              });

            let hidden: usize = self.suggestions.len() - (end - start);

            if hidden > 0 {
              ui.label(
                egui::RichText::new(format!("{hidden} more; Up and Down move through them"))
                  .monospace()
                  .color(egui::Color32::from_gray(110)),
              );
            }
          });
      });

    clicked
  }

  /// Picks the suggestion `step` from the one picked, wrapping round; the first going down from none, the last going
  /// up.
  fn pick(&mut self, step: isize) {
    let count: usize = self.suggestions.len();

    self.picked = Some(match self.picked {
      None if step < 0 => count - 1,
      None => 0,
      Some(picked) => (picked + count).wrapping_add_signed(step) % count,
    });
  }

  /// Puts the suggestion at `index` in the field: a space after a whole name, so its value follows, none after a group,
  /// so the next Tab or keystroke goes on into it.
  fn take_suggestion(&mut self, index: usize) {
    let Some(suggestion) = self.suggestions.get(index) else {
      return;
    };
    let line: String = if suggestion.text.ends_with('.') {
      suggestion.text.clone()
    } else {
      format!("{} ", suggestion.text)
    };

    self.complete(line);
  }

  /// Replaces the field's text with `line`, the cursor at its end, and tells the game it changed.
  fn replace_input(&mut self, line: String) {
    self.input = line;
    self.is_cursor_reset = true;
    self.is_typed = true;
  }

  /// Hands the typed line to the game and keeps it in the history.
  fn enter(&mut self) {
    let entered: String = std::mem::take(&mut self.input);

    self.browsed = None;
    self.is_typed = true;

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
    let line: String = index
      .and_then(|index| self.history.get(index))
      .cloned()
      .unwrap_or_default();

    self.replace_input(line);
    self.browsed = index;
  }
}
