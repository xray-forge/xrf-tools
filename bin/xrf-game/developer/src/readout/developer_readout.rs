use xrf_renderer::{RenderAntialiasing, RenderFrameReport};

/// The renderer's side of the developer readout, its lines read from a frame report as xrf-app's
/// `RenderFrameReadout` reads them: the frame rate and time, the drawn size and its antialiasing, and where the scene is
/// drawn at another size, what scales it and from which size.
pub struct DeveloperReadout;

impl DeveloperReadout {
  /// The lines `report` gives, one saying none has come before the first.
  pub fn get_lines(report: Option<&RenderFrameReport>) -> Vec<String> {
    let Some(report) = report else {
      return vec![String::from("Waiting for a frame")];
    };
    let mut lines: Vec<String> = vec![
      format!("{:.0} fps · {:.1} ms", report.frames_per_second, report.frame_time),
      format!(
        "{} × {} · {}",
        report.width,
        report.height,
        Self::get_antialiasing_name(report.antialiasing)
      ),
    ];

    lines.extend(Self::get_scaling_line(report));
    lines
  }

  /// Where the scene is drawn or resolved at another size than the view's: the temporal resolve to the output, else
  /// FSR 1 to the view, the target left out where it is the view's own size.
  fn get_scaling_line(report: &RenderFrameReport) -> Option<String> {
    let is_temporal: bool = report.antialiasing.is_temporal();
    let size = |width: u32, height: u32| format!("{width} × {height}");
    let target: String = if is_temporal {
      size(report.output_width, report.output_height)
    } else {
      size(report.width, report.height)
    };
    let is_scaled: bool = report.render_width > 0
      && (report.render_width != report.width
        || report.render_height != report.height
        || (is_temporal && report.output_width != report.width));

    if !is_scaled {
      return None;
    }

    let name: String = if is_temporal {
      format!(
        "{}{}",
        Self::get_antialiasing_name(report.antialiasing),
        if report.is_ray_reconstructed { " RR" } else { "" }
      )
    } else {
      String::from("FSR 1")
    };
    let source: String = size(report.render_width, report.render_height);
    let view: String = size(report.width, report.height);

    Some(if source == target || target == view {
      format!("{name} {source}")
    } else {
      format!("{name} {source} → {target}")
    })
  }

  fn get_antialiasing_name(antialiasing: RenderAntialiasing) -> &'static str {
    match antialiasing {
      RenderAntialiasing::None => "No AA",
      RenderAntialiasing::Fxaa => "FXAA",
      RenderAntialiasing::Smaa => "SMAA",
      RenderAntialiasing::Taa => "TAA",
      RenderAntialiasing::Fsr => "FSR 3.1",
      RenderAntialiasing::Dlss => "DLSS",
    }
  }
}
