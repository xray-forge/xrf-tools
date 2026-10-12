use glam::{Mat4, Vec3, Vec4};

use crate::camera::camera_view::CameraView;
use crate::contract::render_ambient_occlusion_settings::RenderAmbientOcclusionSettings;
use crate::contract::render_debanding_quality::RenderDebandingQuality;
use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_image_corrections::RenderImageCorrections;
use crate::contract::render_lights_settings::RenderLightsSettings;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_reflection_settings::RenderReflectionSettings;
use crate::contract::render_shadow_settings::RenderShadowSettings;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::ambient_occlusion_uniform::AmbientOcclusionUniform;
use crate::pass::contact_shadow_uniform::ContactShadowUniform;
use crate::pass::froxel_uniform::FroxelUniform;
use crate::pass::glass_uniform::GlassUniform;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::present_uniform::PresentUniform;
use crate::pass::reflection_trace::ReflectionTrace;
use crate::pass::reflection_uniform::ReflectionUniform;
use crate::pass::static_cull_params::StaticCullParams;
use crate::pass::static_occlusion_uniform::StaticOcclusionUniform;
use crate::pass::wet_uniform::WetUniform;
use crate::pass::wind_uniform::WindUniform;
use crate::scene::level::sky_views::SkyViews;
use crate::scene::static_scene::static_layout::StaticLayout;

/// One frame of a view as its preparation leaves it: the camera and sun it is drawn by, what its options and weather
/// decided it draws, and what its passes read of that. Prepared before the frame graph is declared, and read by it.
pub struct ViewInfo {
  /// What the present pass shows this frame.
  pub present: PresentUniform,
  /// What the ambient occlusion searches by this frame.
  pub occlusion_settings: AmbientOcclusionUniform,
  /// What the contact shadows march by this frame: towards no light where none are drawn.
  pub contact_shadows: ContactShadowUniform,
  /// Whether the sun's contact shadows pass draws this frame.
  pub is_sun_contact: bool,
  /// What the wet surfaces draw by this frame, and their splash volume and flow as the weather texture cache holds
  /// them when the frame is prepared, each its kind's placeholder until its file is up.
  pub wet: WetUniform,
  pub wet_views: Option<[wgpu::TextureView; 2]>,
  pub camera: CameraView,
  /// The camera's view and projection, which become the depth history once the pyramid is reduced.
  pub matrices: (Mat4, Mat4),
  /// Where the sunlight travels.
  pub sun_direction: Vec3,
  /// What the static scene is drawn with off the viewport (the surroundings cube): textured, the viewport's bumping
  /// and the baked hemisphere's strength.
  pub switches: Vec4,
  /// The wind's amplitude and the time the trees sway by: what has still shadows drawn again.
  pub sway: (f32, f32),
  /// The cull's parameters for this frame's view and options.
  pub cull: StaticCullParams,
  /// The view the depth pyramid was reduced through, which the early cull tests against.
  pub occlusion: StaticOcclusionUniform,
  /// How the trees sway this frame, which every static draw reads.
  pub wind: WindUniform,
  /// The viewport's rectangle in its window, which the frame is upscaled to where it is drawn smaller.
  pub output: RenderRect,
  pub shadow_settings: RenderShadowSettings,
  pub lights_settings: RenderLightsSettings,
  pub ambient_occlusion: RenderAmbientOcclusionSettings,
  /// What corrects the finished image, and what the present shows.
  pub corrections: RenderImageCorrections,
  pub debug_view: RenderDebugView,
  /// Whether its ambient occlusion is searched: on, and the view lit and solid.
  pub is_occlusion_drawn: bool,
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
  /// How hard the sky is debanded, none where it is drawn as it is.
  pub debanding: Option<RenderDebandingQuality>,
  /// The lighting the passes read, as the frame's weather and options make it.
  pub lighting: LightingUniform,
  /// The weather textures the sky draws with and the water reflects; none until the frame's sky is prepared.
  pub sky: Option<SkyViews>,
  /// The sun's sprite as the sky draws it: its texture, and its colour and radius.
  pub sun_sprite: Option<(String, Vec4)>,
  /// Whether the surfaces are wetted this frame: while it rains, or, enhanced, while the level is still wet after.
  pub is_wet: bool,
  /// Composited clusters sorted back to front, and the runs of them alike in layout, in their order: the layout,
  /// the run's first entry in the sorted list and how many.
  pub sorted_count: u32,
  pub sorted_runs: Vec<(StaticLayout, u32, u32)>,
  /// Whether the scene's distorting surfaces are drawn into the distortion target this frame.
  pub is_glass_distorting: bool,
  /// The enhanced glass's strengths, where it draws the scene's glass this frame.
  pub glass: Option<GlassUniform>,
  /// The enhanced volumetric lighting's grid and strengths, where it draws in place of the engine's shafts this frame.
  pub froxels: Option<FroxelUniform>,
  /// The colour the selection is outlined in, or none while nothing it names is drawn.
  pub selection_color: Option<[f32; 3]>,
}

impl Default for ViewInfo {
  fn default() -> Self {
    Self {
      present: PresentUniform::default(),
      occlusion_settings: AmbientOcclusionUniform::default(),
      contact_shadows: ContactShadowUniform::default(),
      is_sun_contact: false,
      wet: WetUniform::default(),
      wet_views: None,
      camera: CameraView {
        position: Vec3::ZERO,
        view: Mat4::IDENTITY,
        projection: Mat4::IDENTITY,
      },
      matrices: (Mat4::IDENTITY, Mat4::IDENTITY),
      sun_direction: RenderLighting::default().get_sun_direction(),
      switches: Vec4::new(1.0, 1.0, 1.0, 0.0),
      sway: (0.0, 0.0),
      cull: StaticCullParams::default(),
      occlusion: bytemuck::Zeroable::zeroed(),
      wind: WindUniform::default(),
      output: RenderRect::default(),
      shadow_settings: RenderShadowSettings::default(),
      lights_settings: RenderLightsSettings::default(),
      ambient_occlusion: RenderAmbientOcclusionSettings::default(),
      corrections: RenderImageCorrections::default(),
      debug_view: RenderDebugView::Final,
      is_occlusion_drawn: false,
      reflection_settings: RenderReflectionSettings::default(),
      reflection: None,
      reflections: ReflectionUniform::default(),
      is_wireframe: false,
      is_hazing: false,
      is_wallmarked: true,
      is_shafted: false,
      debanding: None,
      lighting: bytemuck::Zeroable::zeroed(),
      sky: None,
      sun_sprite: None,
      is_wet: false,
      sorted_count: 0,
      is_glass_distorting: false,
      glass: None,
      froxels: None,
      sorted_runs: Vec::new(),
      selection_color: None,
    }
  }
}
