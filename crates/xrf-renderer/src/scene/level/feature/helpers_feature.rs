use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;
use xrf_renderer_core::{
  FrameGraph, GraphBuffer, GraphBufferAccess, GraphColorAttachment, GraphDepthAttachment, GraphTexture,
  GraphTextureDescriptor,
};

use crate::contract::render_helper_options::RenderHelperOptions;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::frame::view_targets::ViewTargets;
use crate::pass::static_draws::StaticDraws;
use crate::pass::static_gbuffer_pass::StaticGBufferPass;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::helpers::helper_vertex::HelperVertex;
use crate::scene::level::helpers::level_helper_object::LevelHelperObject;
use crate::scene::level::helpers::level_helpers::LevelHelpers;
use crate::scene::static_scene::static_scene::StaticScene;

/// What a view draws of its scene's helpers: the wires its options keep, which the window pass draws and a pick draws
/// again wide, and the spawned models they mark, which the helper marks pass draws in their colours for the present to
/// tint and outline; both built again only when the scene's set, its spawned objects or the options change.
#[derive(Default)]
pub struct HelpersFeature {
  /// The set, the scene's contents and the options the wires and marks were built for.
  built: Option<(u64, usize, RenderHelperOptions)>,
  /// Two vertices a segment, and how many vertices.
  wires: Option<(wgpu::Buffer, u32)>,
  /// The marked models' places and colours, bound as the helper marks pass reads them.
  marks: Option<(wgpu::Buffer, wgpu::BindGroup)>,
  /// Whether the present fades what no helper covers.
  is_rest_faded: bool,
}

impl HelpersFeature {
  /// Builds the wires and marks again where the scene's helpers, the objects standing in its scene or the view's
  /// options moved since: the objects of the kinds switched on and not hidden, none while helpers are off; the marks
  /// bound by `marks_layout`.
  pub fn set(
    &mut self,
    device: &wgpu::Device,
    (helpers, statics): (&LevelHelpers, &StaticScene),
    options: &RenderHelperOptions,
    marks_layout: &wgpu::BindGroupLayout,
  ) {
    let contents: usize = statics.get_contents();

    if self.built.as_ref().is_some_and(|(version, built_contents, built)| {
      *version == helpers.version && *built_contents == contents && built == options
    }) {
      return;
    }

    self.built = Some((helpers.version, contents, options.clone()));
    self.is_rest_faded = options.is_shown && options.is_rest_faded;

    let vertices: Vec<HelperVertex> = Self::to_vertices(helpers, options, |object| statics.get_object_box(object));
    let marks: Vec<[u32; 2]> = Self::to_marks(helpers, options, |object| statics.get_object_place(object));

    self.wires = (!vertices.is_empty()).then(|| {
      (
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
          label: Some("helper wires"),
          contents: bytemuck::cast_slice(&vertices),
          usage: wgpu::BufferUsages::VERTEX,
        }),
        vertices.len() as u32,
      )
    });
    self.marks = (!marks.is_empty()).then(|| {
      let buffer: wgpu::Buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("helper marks"),
        contents: bytemuck::cast_slice(&marks),
        usage: wgpu::BufferUsages::STORAGE,
      });
      let group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("helper marks"),
        layout: marks_layout,
        entries: &[wgpu::BindGroupEntry {
          binding: 0,
          resource: buffer.as_entire_binding(),
        }],
      });

      (buffer, group)
    });
  }

  /// The wires to draw, and how many vertices; none where nothing is drawn.
  pub fn get_wires(&self) -> Option<(&wgpu::Buffer, u32)> {
    self.wires.as_ref().map(|(buffer, count)| (buffer, *count))
  }

  /// The marked models' places and colours as the helper marks pass binds them; none where no model is marked.
  pub fn get_marks(&self) -> Option<&wgpu::BindGroup> {
    self.marks.as_ref().map(|(_, group)| group)
  }

  /// Whether the present fades what no helper covers.
  pub fn is_rest_faded(&self) -> bool {
    self.is_rest_faded
  }

  /// Adds the helper marks pass where the view marks models: the spawned models' visible clusters drawn again over the
  /// view's depth (`handles`) into a marks target of the drawn size, each in its helper's colour where its fragment is
  /// the one kept; answers the target, none where no model is marked.
  pub fn add_marks_pass<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    gbuffer: &'a StaticGBufferPass,
    (view, textures): (&'a ViewBinding, &'a wgpu::BindGroup),
    (scene, handles): (&StaticSceneHandles, ViewTargetHandles),
  ) -> Option<GraphTexture> {
    let marks: &'a wgpu::BindGroup = self.get_marks()?;
    let (width, height) = handles.size;
    let target: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
      "helper marks",
      width,
      height,
      ViewTargets::MARKS,
    ));
    let draws: StaticDraws = scene.get_camera_draws();
    let args: [GraphBuffer; 2] = [scene.args, scene.late];

    StaticDraws::declare_layouts(
      graph
        .add_raster_pass("helper marks")
        .color(GraphColorAttachment::new(
          target,
          wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        ))
        .depth(GraphDepthAttachment::new_read_only(handles.depth))
        .buffer(args[0], GraphBufferAccess::Indirect)
        .buffer(args[1], GraphBufferAccess::Indirect),
      &draws.layouts,
    )
    .record(move |context| {
      let args: [&wgpu::Buffer; 2] = args.map(|args| context.get_buffer(args));

      gbuffer.mark(context, (view, &draws, textures, marks), &args);
    });

    Some(target)
  }

  /// The objects of `helpers` that `options` draws: the kinds switched on and not hidden, none while helpers are off.
  fn list_kept<'a>(
    helpers: &'a LevelHelpers,
    options: &'a RenderHelperOptions,
  ) -> impl Iterator<Item = &'a LevelHelperObject> {
    helpers.objects.iter().filter(|object| {
      options.is_shown
        && object.kind < u32::BITS
        && options.kinds & 1 << object.kind != 0
        && options.hidden.binary_search(&object.id).is_err()
    })
  }

  /// The ends of every wire `options` keeps of `helpers`, a hovered or selected model's box among them where `boxes`
  /// stands it (`StaticScene::get_object_box`).
  pub fn to_vertices(
    helpers: &LevelHelpers,
    options: &RenderHelperOptions,
    boxes: impl Fn(u32) -> Option<(Mat4, [Vec3; 2])>,
  ) -> Vec<HelperVertex> {
    Self::list_kept(helpers, options)
      .flat_map(|object: &LevelHelperObject| {
        let singled: f32 = if options.selected.binary_search(&object.id).is_ok() {
          HelperVertex::SELECTED
        } else if options.hovered == Some(object.id) {
          HelperVertex::HOVERED
        } else {
          0.0
        };
        let color: [f32; 4] = [object.color[0], object.color[1], object.color[2], singled];
        let fade: [f32; 2] = [
          if singled > 0.0 {
            0.0
          } else {
            object.draw_distance * options.distance_scale
          },
          options.occluded_strength,
        ];
        let boxed: Option<(Mat4, [Vec3; 2])> = object.model.filter(|_| singled > 0.0).and_then(&boxes);

        helpers.segments[object.segments.clone()]
          .iter()
          .copied()
          .chain(boxed.into_iter().flat_map(|(transform, bounds)| LevelHelpers::to_box_edges(transform, bounds)))
          .flatten()
          .map(move |end: Vec3| HelperVertex {
            position: end.to_array(),
            color,
            fade,
            object: object.id,
          })
      })
      .collect()
  }

  /// The models `options` marks of `helpers`, those standing at a place in the scene (`places`,
  /// `StaticScene::get_object_place`): each place and its colour packed as `unpack4x8unorm` reads it, by place
  /// ascending, as `fs_helper_mark` searches them.
  pub fn to_marks(
    helpers: &LevelHelpers,
    options: &RenderHelperOptions,
    places: impl Fn(u32) -> Option<u32>,
  ) -> Vec<[u32; 2]> {
    let mut marks: Vec<[u32; 2]> = Self::list_kept(helpers, options)
      .filter_map(|object: &LevelHelperObject| {
        let [r, g, b] = object.color.map(|channel| (channel.clamp(0.0, 1.0) * 255.0).round() as u32);

        Some([places(object.model?)?, r | g << 8 | b << 16 | 255 << 24])
      })
      .collect();

    marks.sort_unstable_by_key(|[place, _]| *place);
    marks
  }
}

impl ViewFeature for HelpersFeature {
  /// The wires' vertices and the marks.
  fn get_memory(&self) -> u64 {
    self.wires.as_ref().map_or(0, |(buffer, _)| buffer.size()) + self.marks.as_ref().map_or(0, |(buffer, _)| buffer.size())
  }

  fn release(&mut self) {
    self.built = None;
    self.wires = None;
    self.marks = None;
    self.is_rest_faded = false;
  }
}
