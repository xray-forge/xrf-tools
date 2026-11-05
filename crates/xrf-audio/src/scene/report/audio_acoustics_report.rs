/// How a scene's acoustics run as of its last tick.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AudioAcousticsReport {
  /// Why the scene plays dry, its sounds placed by distance and panning alone; none while the acoustics run.
  pub failure: Option<String>,
  /// The triangles the simulation traces, none while the geometry is built or where there is none.
  pub triangles: Option<u32>,
  /// Why the scene's walls shape nothing though the acoustics run: it has no geometry, or the geometry failed.
  pub geometry_failure: Option<String>,
  /// The listener's reverb, each band's decay time in seconds (below 800 Hz, to 8 kHz, above).
  pub reverb_times: [f32; 3],
  /// How long the geometry took to read and build, in milliseconds.
  pub build_ms: f32,
  /// How long the last simulation of the sounds' ways through the walls took, in milliseconds.
  pub direct_ms: f32,
  /// How long the last simulation of the listener's reverb took, in milliseconds.
  pub reflections_ms: f32,
}
