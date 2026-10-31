use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;
use xrf_renderer_core::{
  FrameGraph, GraphBuffer, GraphBufferAccess, GraphColorAttachment, GraphDepthAttachment, GraphTexture,
  GraphTextureDescriptor,
};

use crate::contract::render_helper_icon::RenderHelperIcon;
use crate::contract::render_helper_options::RenderHelperOptions;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::frame::view_targets::ViewTargets;
use crate::pass::static_draws::StaticDraws;
use crate::pass::static_gbuffer_pass::StaticGBufferPass;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::helpers::helper_icon_instance::HelperIconInstance;
use crate::scene::level::helpers::helper_vertex::HelperVertex;
use crate::scene::level::helpers::level_helper_object::LevelHelperObject;
use crate::scene::level::helpers::level_helpers::LevelHelpers;
use crate::scene::static_scene::static_scene::StaticScene;

/// What a view draws of its scene's helpers: the wires and icons its options keep, which the window pass draws and a
/// pick draws again, and the spawned models they mark, which the helper marks pass draws in their colours for the
/// present to tint and outline. Every kept object's are built again only when the scene's set, its spawned objects or
/// the options the selection and hovering aside change; the selected and hovered objects' are built apart and drawn
/// over them, so the pointer moving over the scene rebuilds a handful of wires rather than every helper.
#[derive(Default)]
pub struct HelpersFeature {
  /// The set, the scene's contents and the options, singled out objects aside, every kept object's were built for.
  built: Option<(u64, usize, RenderHelperOptions)>,
  /// The set, the scene's contents and the options the singled out objects' were built for.
  singled_built: Option<(u64, usize, RenderHelperOptions)>,
  /// Every kept object's wires, two vertices a segment, and icons, an instance each, each with how many.
  wires: Option<(wgpu::Buffer, u32)>,
  icons: Option<(wgpu::Buffer, u32)>,
  /// The selected and hovered objects' wires, their models' boxes among them, and icons, lifted and never faded.
  singled_wires: Option<(wgpu::Buffer, u32)>,
  singled_icons: Option<(wgpu::Buffer, u32)>,
  /// The marked models' places and colours, bound as the helper marks pass reads them.
  marks: Option<(wgpu::Buffer, wgpu::BindGroup)>,
  /// Whether the present fades what no helper covers.
  is_rest_faded: bool,
}

/// Which of the kept objects a build of a view's helpers takes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HelperPart {
  /// Every kept object, none singled out.
  Every,
  /// The selected and hovered objects alone, singled out, drawn over every kept object's.
  Singled,
}

impl HelpersFeature {
  /// Builds the wires, icons and marks again where the scene's helpers, the objects standing in its scene or the view's
  /// options moved since: the objects of the kinds switched on and not hidden, none while helpers are off, every kept
  /// object's apart from the singled out ones'; the marks bound by `marks_layout`.
  pub fn set(
    &mut self,
    device: &wgpu::Device,
    (helpers, statics): (&LevelHelpers, &StaticScene),
    options: &RenderHelperOptions,
    marks_layout: &wgpu::BindGroupLayout,
  ) {
    let contents: usize = statics.get_contents();
    let unsingled: RenderHelperOptions = RenderHelperOptions {
      selected: Vec::new(),
      hovered: None,
      ..options.clone()
    };
    let is_built = |built: &Option<(u64, usize, RenderHelperOptions)>, options: &RenderHelperOptions| {
      built.as_ref().is_some_and(|(version, built_contents, built)| {
        *version == helpers.version && *built_contents == contents && built == options
      })
    };
    let boxes = |object: u32| statics.get_object_box(object);

    if !is_built(&self.singled_built, options) {
      self.singled_built = Some((helpers.version, contents, options.clone()));
      self.singled_wires = Self::create_buffer(
        device,
        "singled helper wires",
        &Self::to_vertices(helpers, options, HelperPart::Singled, boxes),
      );
      self.singled_icons = Self::create_buffer(
        device,
        "singled helper icons",
        &Self::to_icons(helpers, options, HelperPart::Singled),
      );
    }

    if is_built(&self.built, &unsingled) {
      return;
    }

    self.is_rest_faded = options.is_shown && options.is_rest_faded;
    self.wires = Self::create_buffer(
      device,
      "helper wires",
      &Self::to_vertices(helpers, &unsingled, HelperPart::Every, boxes),
    );
    self.icons = Self::create_buffer(
      device,
      "helper icons",
      &Self::to_icons(helpers, &unsingled, HelperPart::Every),
    );

    let marks: Vec<[u32; 2]> = Self::to_marks(helpers, &unsingled, |object| statics.get_object_place(object));

    self.built = Some((helpers.version, contents, unsingled));
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

  /// A vertex buffer of `items` and how many, none for none.
  fn create_buffer<T: bytemuck::Pod>(device: &wgpu::Device, label: &str, items: &[T]) -> Option<(wgpu::Buffer, u32)> {
    (!items.is_empty()).then(|| {
      (
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
          label: Some(label),
          contents: bytemuck::cast_slice(items),
          usage: wgpu::BufferUsages::VERTEX,
        }),
        items.len() as u32,
      )
    })
  }

  /// The wires and icons to draw: every kept object's, then the singled out ones' over them.
  pub fn get_draws(&self) -> HelperDraws<'_> {
    HelperDraws {
      wires: [Self::to_draw(&self.wires), Self::to_draw(&self.singled_wires)],
      icons: [Self::to_draw(&self.icons), Self::to_draw(&self.singled_icons)],
    }
  }

  fn to_draw(built: &Option<(wgpu::Buffer, u32)>) -> Option<(&wgpu::Buffer, u32)> {
    built.as_ref().map(|(buffer, count)| (buffer, *count))
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

  /// The objects of `helpers` that `options` draws, of `part`: the kinds switched on and not hidden, none while
  /// helpers are off.
  fn list_kept<'a>(
    helpers: &'a LevelHelpers,
    options: &'a RenderHelperOptions,
    part: HelperPart,
  ) -> impl Iterator<Item = &'a LevelHelperObject> {
    helpers.objects.iter().filter(move |object| {
      options.is_shown
        && object.kind < u32::BITS
        && options.kinds & 1 << object.kind != 0
        && options.hidden.binary_search(&object.id).is_err()
        && (part == HelperPart::Every || Self::get_singled(object, options) > 0.0)
    })
  }

  /// How `object` is singled out under `options`: [`HelperVertex::SELECTED`], [`HelperVertex::HOVERED`] or nought.
  fn get_singled(object: &LevelHelperObject, options: &RenderHelperOptions) -> f32 {
    if options.selected.binary_search(&object.id).is_ok() {
      HelperVertex::SELECTED
    } else if options.hovered == Some(object.id) {
      HelperVertex::HOVERED
    } else {
      0.0
    }
  }

  /// How `object` draws under `options`, built for `part`: its colour with how it is singled out, nought for every
  /// kept object's, then the distance it fades out by, none for a singled out one, and how strongly it shows hidden.
  fn to_look(object: &LevelHelperObject, options: &RenderHelperOptions, part: HelperPart) -> ([f32; 4], [f32; 2]) {
    let singled: f32 = match part {
      HelperPart::Every => 0.0,
      HelperPart::Singled => Self::get_singled(object, options),
    };
    let reach: f32 = if singled > 0.0 {
      0.0
    } else {
      object.draw_distance * options.distance_scale
    };

    (
      [object.color[0], object.color[1], object.color[2], singled],
      [reach, options.occluded_strength],
    )
  }

  /// The ends of every wire `options` keeps of `helpers` for `part`, a singled out model's box among them where
  /// `boxes` stands it (`StaticScene::get_object_box`).
  pub fn to_vertices(
    helpers: &LevelHelpers,
    options: &RenderHelperOptions,
    part: HelperPart,
    boxes: impl Fn(u32) -> Option<(Mat4, [Vec3; 2])>,
  ) -> Vec<HelperVertex> {
    Self::list_kept(helpers, options, part)
      .flat_map(|object: &LevelHelperObject| {
        let (color, fade): ([f32; 4], [f32; 2]) = Self::to_look(object, options, part);
        let boxed: Option<(Mat4, [Vec3; 2])> = object.model.filter(|_| color[3] > 0.0).and_then(&boxes);

        helpers.segments[object.segments.clone()]
          .iter()
          .copied()
          .chain(
            boxed
              .into_iter()
              .flat_map(|(transform, bounds)| LevelHelpers::to_box_edges(transform, bounds)),
          )
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

  /// Every icon `options` keeps of `helpers` for `part`, drawn as its object's wires are.
  pub fn to_icons(helpers: &LevelHelpers, options: &RenderHelperOptions, part: HelperPart) -> Vec<HelperIconInstance> {
    Self::list_kept(helpers, options, part)
      .flat_map(|object: &LevelHelperObject| {
        let (color, fade): ([f32; 4], [f32; 2]) = Self::to_look(object, options, part);

        helpers.icons[object.icons.clone()]
          .iter()
          .map(move |(position, icon): &(Vec3, RenderHelperIcon)| HelperIconInstance {
            position: position.to_array(),
            icon: icon.get_index(),
            color,
            fade,
            object: object.id,
            _pad: 0,
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
    let mut marks: Vec<[u32; 2]> = Self::list_kept(helpers, options, HelperPart::Every)
      .filter_map(|object: &LevelHelperObject| {
        let [r, g, b] = object
          .color
          .map(|channel| (channel.clamp(0.0, 1.0) * 255.0).round() as u32);

        Some([places(object.model?)?, r | g << 8 | b << 16 | 255 << 24])
      })
      .collect();

    marks.sort_unstable_by_key(|[place, _]| *place);
    marks
  }
}

/// What a view draws of its helpers over its frame: the wires, two vertices a segment, and the icons, an instance each,
/// each with how many, every kept object's then the singled out ones' over them; none where it draws none.
#[derive(Clone, Copy, Default)]
pub struct HelperDraws<'a> {
  pub wires: [Option<(&'a wgpu::Buffer, u32)>; 2],
  pub icons: [Option<(&'a wgpu::Buffer, u32)>; 2],
}

impl ViewFeature for HelpersFeature {
  /// The wires' vertices, the icons and the marks.
  fn get_memory(&self) -> u64 {
    [&self.wires, &self.icons, &self.singled_wires, &self.singled_icons]
      .into_iter()
      .flatten()
      .map(|(buffer, _)| buffer.size())
      .sum::<u64>()
      + self.marks.as_ref().map_or(0, |(buffer, _)| buffer.size())
  }

  fn release(&mut self) {
    self.built = None;
    self.singled_built = None;
    self.wires = None;
    self.icons = None;
    self.singled_wires = None;
    self.singled_icons = None;
    self.marks = None;
    self.is_rest_faded = false;
  }
}
