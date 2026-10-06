use glam::{Mat4, Vec2, Vec3, Vec4};
use xrf_material::XraySurfaceDraw;

use crate::camera::camera_view::CameraView;
use crate::contract::render_ambient_occlusion_settings::RenderAmbientOcclusionSettings;
use crate::contract::render_antialiasing::RenderAntialiasing;
use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_image_corrections::RenderImageCorrections;
use crate::contract::render_indirect_light_settings::RenderIndirectLightSettings;
use crate::contract::render_lights_settings::RenderLightsSettings;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_reflection_settings::RenderReflectionSettings;
use crate::contract::render_shadow_settings::RenderShadowSettings;
use crate::contract::render_upscaling_settings::RenderUpscalingSettings;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::ambient_occlusion_uniform::AmbientOcclusionUniform;
use crate::pass::bitmask_search::BitmaskSearch;
use crate::pass::bloom_uniform::BloomUniform;
use crate::pass::contact_shadow_uniform::ContactShadowUniform;
use crate::pass::fsr_uniform::FsrUniform;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::present_uniform::PresentUniform;
use crate::pass::rain_uniform::RainUniform;
use crate::pass::reflection_trace::ReflectionTrace;
use crate::pass::reflection_uniform::ReflectionUniform;
use crate::pass::static_cull_params::StaticCullParams;
use crate::pass::static_occlusion_uniform::StaticOcclusionUniform;
use crate::pass::temporal_uniform::TemporalUniform;
use crate::pass::thunder_uniform::ThunderUniform;
use crate::pass::upscale_uniform::UpscaleUniform;
use crate::pass::vbao_uniform::VbaoUniform;
use crate::pass::wet_uniform::WetUniform;
use crate::pass::wind_uniform::WindUniform;
use crate::scene::level::sky_views::SkyViews;
use crate::scene::level::weather_views::WeatherViews;

/// One frame of a view as its preparation leaves it: the camera and sun it is drawn by, what its options and weather
/// decided it draws, and what its passes read of that. Prepared before the frame graph is declared, and read by it.
pub struct ViewInfo {
  /// What the present pass shows and the upscale passes read this frame.
  pub present: PresentUniform,
  pub upscale: UpscaleUniform,
  /// What the bloom's build and its two blurs read this frame.
  pub bloom: [BloomUniform; 3],
  /// What the ambient occlusion searches by this frame.
  pub occlusion_settings: AmbientOcclusionUniform,
  /// What VBAO searches, accumulates and filters by this frame.
  pub vbao: VbaoUniform,
  /// What the contact shadows march by this frame: towards no light where none are drawn.
  pub contact_shadows: ContactShadowUniform,
  /// Whether the sun's contact shadows pass draws this frame.
  pub is_sun_contact: bool,
  /// What the rain, the wet surfaces and a strike draw by this frame, and the weather textures they draw with.
  pub rain: RainUniform,
  pub wet: WetUniform,
  pub thunder: ThunderUniform,
  pub weather_views: WeatherViews,
  /// What the temporal resolve, or FSR 2, moves its history by this frame.
  pub temporal: TemporalUniform,
  pub fsr: FsrUniform,
  pub camera: CameraView,
  /// The camera's view and projection, which become the depth history once the pyramid is reduced.
  pub matrices: (Mat4, Mat4),
  /// Where the sunlight travels.
  pub sun_direction: Vec3,
  /// The wind's amplitude and the time the trees sway by: what has still shadows drawn again.
  pub sway: (f32, f32),
  /// Where this frame's samples sit within their pixels, and how many places the jitter cycles through.
  pub jitter: Vec2,
  pub jitter_phases: u32,
  /// Whether a temporal resolve gathers this frame's samples, and whether it is FSR 2's rather than TAA.
  pub is_temporal: bool,
  /// The smoothing pass over the scene as drawn, FXAA or SMAA, while one smooths it.
  pub smoothing: Option<RenderAntialiasing>,
  pub is_fsr: bool,
  /// The cull's parameters for this frame's view and options.
  pub cull: StaticCullParams,
  /// The view the depth pyramid was reduced through, which the early cull tests against.
  pub occlusion: StaticOcclusionUniform,
  /// How the trees sway this frame, which every static draw reads.
  pub wind: WindUniform,
  /// The viewport's rectangle in its window, which the frame is upscaled to where it is drawn smaller.
  pub output: RenderRect,
  pub upscaling: RenderUpscalingSettings,
  pub shadow_settings: RenderShadowSettings,
  pub lights_settings: RenderLightsSettings,
  pub ambient_occlusion: RenderAmbientOcclusionSettings,
  /// What corrects the finished image, and what the present shows.
  pub corrections: RenderImageCorrections,
  pub debug_view: RenderDebugView,
  /// Whether its ambient occlusion is searched: on, and the view lit and solid.
  pub is_occlusion_drawn: bool,
  /// What the visibility-bitmask search yields this frame, none where it does not run.
  pub bitmask: Option<BitmaskSearch>,
  /// Whether the bitmask search accumulates into its history this frame.
  pub is_bitmask_accumulated: bool,
  /// What the indirect light is gathered with, whether or not it is.
  pub indirect_light: RenderIndirectLightSettings,
  /// What the reflections are traced with, whether or not they are.
  pub reflection_settings: RenderReflectionSettings,
  /// Whether and how the reflections are traced this frame, none where they are not.
  pub reflection: Option<ReflectionTrace>,
  /// What the reflections' trace, accumulation and filter read this frame.
  pub reflections: ReflectionUniform,
  /// Whether the static surfaces draw as their edges, which nothing composited or planted is drawn over.
  pub is_wireframe: bool,
  /// Whether the sky is blurred into the haze map the distance fades into.
  pub is_hazing: bool,
  /// Whether the wall marks are laid into the albedo.
  pub is_wallmarked: bool,
  /// Whether the sun's light shafts are added, drawn through its shadow's cascades.
  pub is_shafted: bool,
  pub is_bloomed: bool,
  /// The lighting the passes read, as the frame's weather and options make it.
  pub lighting: LightingUniform,
  /// The weather textures the sky draws with and the water reflects; none until the frame's sky is prepared.
  pub sky: Option<SkyViews>,
  /// The sun's sprite as the sky draws it: its texture, and its colour and radius.
  pub sun_sprite: Option<(String, Vec4)>,
  /// The rain: the streaks drawn and the splash's indices, none while it does not rain.
  pub rain_draw: Option<(u32, u32)>,
  /// Whether the surfaces are wetted this frame: while it rains, or, enhanced, while the level is still wet after.
  pub is_wet: bool,
  /// A strike: how its model and glows composite and the model's indices, none while none strikes.
  pub thunder_draw: Option<([XraySurfaceDraw; 3], u32)>,
  /// Composited clusters sorted back to front.
  pub sorted_count: u32,
  /// The colour the selection is outlined in, or none while nothing it names is drawn.
  pub selection_color: Option<[f32; 3]>,
}

impl Default for ViewInfo {
  fn default() -> Self {
    Self {
      present: PresentUniform::default(),
      upscale: UpscaleUniform::default(),
      bloom: [BloomUniform::default(); 3],
      occlusion_settings: AmbientOcclusionUniform::default(),
      vbao: VbaoUniform::default(),
      contact_shadows: ContactShadowUniform::default(),
      is_sun_contact: false,
      rain: RainUniform::default(),
      wet: WetUniform::default(),
      thunder: ThunderUniform::default(),
      weather_views: WeatherViews::default(),
      temporal: TemporalUniform::default(),
      fsr: FsrUniform::default(),
      camera: CameraView {
        position: Vec3::ZERO,
        view: Mat4::IDENTITY,
        projection: Mat4::IDENTITY,
      },
      matrices: (Mat4::IDENTITY, Mat4::IDENTITY),
      sun_direction: RenderLighting::default().get_sun_direction(),
      sway: (0.0, 0.0),
      jitter: Vec2::ZERO,
      jitter_phases: 1,
      is_temporal: false,
      smoothing: None,
      is_fsr: false,
      cull: StaticCullParams::default(),
      occlusion: bytemuck::Zeroable::zeroed(),
      wind: WindUniform::default(),
      output: RenderRect::default(),
      upscaling: RenderUpscalingSettings::default(),
      shadow_settings: RenderShadowSettings::default(),
      lights_settings: RenderLightsSettings::default(),
      ambient_occlusion: RenderAmbientOcclusionSettings::default(),
      corrections: RenderImageCorrections::default(),
      debug_view: RenderDebugView::Final,
      is_occlusion_drawn: false,
      bitmask: None,
      is_bitmask_accumulated: false,
      indirect_light: RenderIndirectLightSettings::default(),
      reflection_settings: RenderReflectionSettings::default(),
      reflection: None,
      reflections: ReflectionUniform::default(),
      is_wireframe: false,
      is_hazing: false,
      is_wallmarked: true,
      is_shafted: false,
      is_bloomed: false,
      lighting: bytemuck::Zeroable::zeroed(),
      sky: None,
      sun_sprite: None,
      rain_draw: None,
      is_wet: false,
      thunder_draw: None,
      sorted_count: 0,
      selection_color: None,
    }
  }
}
