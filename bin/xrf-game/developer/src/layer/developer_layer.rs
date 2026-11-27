use std::collections::BTreeSet;
use std::time::Instant;

use xrf_game_platform::{PlatformEvent, PlatformEventKind, PlatformMouseButton, PlatformScancode};

use crate::console::developer_console::DeveloperConsole;
use crate::input::developer_keys::DeveloperKeys;
use crate::layer::developer_frame::DeveloperFrame;
use crate::layer::developer_view::DeveloperView;
use crate::paint::developer_paint::DeveloperPaint;

/// The developer layer: an egui interface over the game, built on the main thread from the platform's events and sent
/// to the renderer's layer pass as meshes (plan decision 19: egui's core alone, through a first-party pass). It shows
/// the frame's readout in the top left corner and the camera's in the bottom right always, and the console as a drawer
/// from the top while it is open; open, it takes the keyboard and mouse.
pub struct DeveloperLayer {
  context: egui::Context,
  /// Input egui has not been given yet, in the order it came.
  events: Vec<egui::Event>,
  /// The modifier keys held down.
  held: BTreeSet<PlatformScancode>,
  console: DeveloperConsole,
  is_open: bool,
  started: Instant,
}

impl Default for DeveloperLayer {
  fn default() -> Self {
    let context: egui::Context = egui::Context::default();

    // A tool's interface: it shows what changed at once, with no fades or slides to wait through.
    context.all_styles_mut(|style| style.animation_time = 0.0);

    Self {
      context,
      events: Vec::new(),
      held: BTreeSet::new(),
      console: DeveloperConsole::default(),
      is_open: false,
      started: Instant::now(),
    }
  }
}

impl DeveloperLayer {
  /// The share of the window's height the console's drawer opens to.
  const CONSOLE_SHARE: f32 = 0.4;

  /// Whether the console is open, taking the keyboard and mouse.
  pub fn is_open(&self) -> bool {
    self.is_open
  }

  pub fn get_console_mut(&mut self) -> &mut DeveloperConsole {
    &mut self.console
  }

  /// Takes a platform event in `view`, answering whether the layer kept it from the game: the console key always, and
  /// keys, text and the mouse while the console is open, Escape closing it. Modifier keys are followed either way, so
  /// one held as the console opens still counts.
  pub fn take(&mut self, event: &PlatformEvent, view: DeveloperView) -> bool {
    self.follow_modifiers(&event.kind);

    match &event.kind {
      PlatformEventKind::Key {
        scancode: DeveloperKeys::CONSOLE,
        is_down,
        is_repeat,
      } => {
        if *is_down && !*is_repeat {
          self.is_open = !self.is_open;
        }

        true
      }
      _ if !self.is_open => false,
      PlatformEventKind::Key {
        scancode: DeveloperKeys::CLOSE,
        is_down: true,
        ..
      } => {
        self.is_open = false;

        true
      }
      PlatformEventKind::Key {
        scancode,
        is_down,
        is_repeat,
      } => {
        if let Some(key) = DeveloperKeys::get_key(*scancode) {
          self.events.push(egui::Event::Key {
            key,
            physical_key: Some(key),
            pressed: *is_down,
            repeat: *is_repeat,
            modifiers: self.get_modifiers(),
          });
        }

        true
      }
      PlatformEventKind::Text { text } => {
        // The console key's own character arrives as text with it; it opens the console, not types into it.
        let text: String = text
          .chars()
          .filter(|character| !matches!(character, '`' | '~'))
          .collect();

        if !text.is_empty() {
          self.events.push(egui::Event::Text(text));
        }

        true
      }
      PlatformEventKind::MouseMoved { x, y, .. } => {
        self.events.push(egui::Event::PointerMoved(view.to_points(*x, *y)));

        true
      }
      PlatformEventKind::MouseButton { button, is_down, x, y } => {
        let button: egui::PointerButton = match button {
          PlatformMouseButton::Left => egui::PointerButton::Primary,
          PlatformMouseButton::Right => egui::PointerButton::Secondary,
          PlatformMouseButton::Middle => egui::PointerButton::Middle,
          PlatformMouseButton::Back => egui::PointerButton::Extra1,
          PlatformMouseButton::Forward => egui::PointerButton::Extra2,
        };

        self.events.push(egui::Event::PointerButton {
          pos: view.to_points(*x, *y),
          button,
          pressed: *is_down,
          modifiers: self.get_modifiers(),
        });

        true
      }
      PlatformEventKind::MouseWheel { dx, dy } => {
        self.events.push(egui::Event::MouseWheel {
          unit: egui::MouseWheelUnit::Line,
          delta: egui::vec2(*dx, *dy),
          phase: egui::TouchPhase::Move,
          source: egui::MouseWheelSource::Wheel,
          modifiers: self.get_modifiers(),
        });

        true
      }
      _ => false,
    }
  }

  /// Builds one frame over `view`: the `frame` lines in the top left corner and the `camera` lines in the bottom right,
  /// as xrf-app lays its viewport's readouts out.
  pub fn build(&mut self, view: DeveloperView, frame: &[String], camera: &[String]) -> DeveloperFrame {
    let scale: f32 = view.get_scale();
    let mut input: egui::RawInput = egui::RawInput {
      screen_rect: Some(egui::Rect::from_min_size(
        egui::Pos2::ZERO,
        egui::vec2(view.width as f32 / scale, view.height as f32 / scale),
      )),
      time: Some(self.started.elapsed().as_secs_f64()),
      events: std::mem::take(&mut self.events),
      focused: true,
      ..egui::RawInput::default()
    };

    if let Some(viewport) = input.viewports.get_mut(&egui::ViewportId::ROOT) {
      viewport.native_pixels_per_point = Some(scale);
    } else {
      input.viewports.insert(
        egui::ViewportId::ROOT,
        egui::ViewportInfo {
          native_pixels_per_point: Some(scale),
          ..egui::ViewportInfo::default()
        },
      );
    }

    let is_open: bool = self.is_open;
    let console: &mut DeveloperConsole = &mut self.console;
    let output: egui::FullOutput = self.context.run_ui(input, |ui| {
      let height: f32 = ui.ctx().content_rect().height();

      // A drawer at the top, full width, as X-Ray's, id Tech's and Unreal's consoles open, shown whole at once at a
      // fixed height: a panel sized from its content would shrink by its margin every frame. Closing it is the console
      // key's or Escape's.
      let drawer: Option<egui::Rect> = is_open.then(|| {
        egui::Panel::top("developer_console")
          .exact_size(height * Self::CONSOLE_SHARE)
          .frame(
            egui::Frame::new()
              .fill(egui::Color32::from_black_alpha(220))
              .inner_margin(8.0),
          )
          .show(ui, |ui| console.show(ui))
          .response
          .rect
      });
      // The frame block keeps its corner, dropped below the drawer while it is open.
      let top: f32 = drawer.map_or(0.0, |rect| rect.bottom());

      Self::show_block(
        ui,
        "developer_frame",
        egui::Align2::LEFT_TOP,
        egui::vec2(8.0, top + 8.0),
        frame,
      );
      Self::show_block(
        ui,
        "developer_camera",
        egui::Align2::RIGHT_BOTTOM,
        egui::vec2(-8.0, -8.0),
        camera,
      );
    });
    let pixels_per_point: f32 = output.pixels_per_point;
    let primitives: Vec<egui::ClippedPrimitive> = self.context.tessellate(output.shapes, pixels_per_point);

    DeveloperFrame {
      textures: DeveloperPaint::create_textures(output.textures_delta),
      layer: DeveloperPaint::create_layer(primitives, pixels_per_point),
    }
  }

  /// Follows which modifier keys are held; losing focus lets all go, since their releases will never arrive.
  fn follow_modifiers(&mut self, kind: &PlatformEventKind) {
    match kind {
      PlatformEventKind::Key { scancode, is_down, .. } if DeveloperKeys::get_modifier(*scancode).is_some() => {
        if *is_down {
          self.held.insert(*scancode);
        } else {
          self.held.remove(scancode);
        }
      }
      PlatformEventKind::Focused { is_focused: false } => self.held.clear(),
      _ => {}
    }
  }

  /// The modifiers the held keys make.
  pub(crate) fn get_modifiers(&self) -> egui::Modifiers {
    self
      .held
      .iter()
      .filter_map(|scancode| DeveloperKeys::get_modifier(*scancode))
      .fold(egui::Modifiers::NONE, |modifiers, modifier| modifiers | modifier)
  }

  /// A block of `lines` held to a corner of the window by `anchor`, `offset` in from it, over a dark backing; none
  /// without lines.
  fn show_block(ui: &mut egui::Ui, name: &str, anchor: egui::Align2, offset: egui::Vec2, lines: &[String]) {
    if lines.is_empty() {
      return;
    }

    egui::Area::new(egui::Id::unique(name))
      .anchor(anchor, offset)
      .fade_in(false)
      .show(ui.ctx(), |ui| {
        egui::Frame::new()
          .fill(egui::Color32::from_black_alpha(160))
          .inner_margin(6.0)
          .corner_radius(3.0)
          .show(ui, |ui| {
            for line in lines {
              ui.add(
                egui::Label::new(
                  egui::RichText::new(line)
                    .monospace()
                    .color(egui::Color32::from_gray(220)),
                )
                .extend(),
              );
            }
          });
      });
  }
}
