use glam::Vec3;
use wgpu::util::DeviceExt;
use xrf_renderer_core::{
  FrameGraph, GraphBuffer, GraphBufferAccess, GraphColorAttachment, GraphDepthAttachment, GraphTexture,
  GraphTextureDescriptor,
};

use crate::contract::render_helper_options::RenderHelperOptions;
use crate::frame::view_targets::ViewTargets;
use crate::pass::present_marks::PresentMarks;
use crate::pass::static_draws::StaticDraws;
use crate::pass::static_mark_draw::StaticMarkDraw;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::helpers::gpu::helper_buffer::HelperBuffer;
use crate::scene::level::helpers::gpu::helper_icon_instance::HelperIconInstance;
use crate::scene::level::helpers::gpu::helper_vertex::HelperVertex;
use crate::scene::level::helpers::level_helper_icon::LevelHelperIcon;
use crate::scene::level::helpers::level_helper_object::LevelHelperObject;
use crate::scene::level::helpers::level_helpers::LevelHelpers;
use crate::scene::level::helpers::view::helper_draws::HelperDraws;
use crate::scene::level::helpers::view::helper_marks_pass::HelperMarksPass;
use crate::scene::level::helpers::view::helper_part::HelperPart;
use crate::scene::level::helpers::view::helper_part_draws::HelperPartDraws;
use crate::scene::level::helpers::view::helper_sources::HelperSources;
use crate::scene::static_scene::static_object_box::StaticObjectBox;

/// What a view draws of its scene's helpers: the wires and icons its options keep, which the window pass draws and a
/// pick draws again, and the spawned models they mark, which the helper marks pass draws in their colours for the
/// present to tint and outline. Every kept object's are built again only when the scene's set, its spawned objects or
/// the options the selection and hovering aside change; the selected and hovered objects' are built apart and drawn
/// over them, so the pointer moving over the scene rebuilds a handful of wires rather than every helper.
#[derive(Default)]
pub struct HelpersFeature {
  /// Every kept object's wires and icons, none singled out.
  every: Option<HelperBuild>,
  /// The selected and hovered objects' wires, their models' boxes among them, and icons, lifted and never faded.
  singled: Option<HelperBuild>,
  /// The marked models' places and colours, bound as the helper marks pass reads them.
  marks: Option<HelperMarks>,
  /// Whether the present fades what no helper covers.
  is_rest_faded: bool,
}

impl HelpersFeature {
  /// Builds the wires, icons and marks again where the scene's helpers, the objects standing in its scene or the view's
  /// options moved since: the objects of the kinds switched on and not hidden, none while helpers are off, every kept
  /// object's apart from the singled out ones'.
  pub fn set(&mut self, device: &wgpu::Device, sources: HelperSources<'_>) {
    let HelperSources {
      helpers,
      statics,
      options,
      marks_layout,
    } = sources;
    let unsingled: RenderHelperOptions = RenderHelperOptions {
      selected: Vec::new(),
      hovered: None,
      ..options.clone()
    };
    let boxes = |object: u32| statics.get_object_box(object);

    if !HelperBuild::is_current(&self.singled, &HelperBuildKey::of(sources, options)) {
      self.singled = Some(HelperBuild::new(
        device,
        HelperBuildKey::of(sources, options),
        HelperBuild::build(helpers, options, HelperPart::Singled, boxes),
      ));
    }

    if HelperBuild::is_current(&self.every, &HelperBuildKey::of(sources, &unsingled)) {
      return;
    }

    let marks: Vec<[u32; 2]> = Self::to_marks(helpers, &unsingled, |object| statics.get_object_place(object));

    self.is_rest_faded = options.is_shown && options.is_rest_faded;
    self.every = Some(HelperBuild::new(
      device,
      HelperBuildKey::of(sources, &unsingled),
      HelperBuild::build(helpers, &unsingled, HelperPart::Every, boxes),
    ));
    self.marks = (!marks.is_empty()).then(|| HelperMarks::new(device, &marks, marks_layout));
  }

  /// The wires and icons to draw: every kept object's, then the singled out ones' over them.
  pub fn get_draws(&self) -> HelperDraws<'_> {
    HelperDraws {
      every: HelperBuild::get_draws(&self.every),
      singled: HelperBuild::get_draws(&self.singled),
    }
  }

  /// The marked models' places and colours as the helper marks pass binds them; none where no model is marked.
  pub fn get_marks(&self) -> Option<&wgpu::BindGroup> {
    self.marks.as_ref().map(|marks| &marks.group)
  }

  /// What the present lays over the frame of the marks: whether any model is marked, and whether the rest fades.
  pub fn get_present_marks(&self) -> PresentMarks {
    PresentMarks {
      is_marked: self.marks.is_some(),
      is_rest_faded: self.is_rest_faded,
    }
  }

  /// Adds the helper marks pass where the view marks models: the spawned models' visible clusters drawn again over the
  /// view's depth into a marks target of the drawn size, each in its helper's colour where its fragment is the one kept;
  /// answers the target, none where no model is marked.
  pub fn add_marks_pass<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    pass: HelperMarksPass<'a, '_>,
  ) -> Option<GraphTexture> {
    let marks: &'a wgpu::BindGroup = self.get_marks()?;
    let HelperMarksPass {
      gbuffer,
      view,
      textures,
      scene,
      handles,
    } = pass;
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

      gbuffer.mark(
        context,
        StaticMarkDraw {
          view,
          draws: &draws,
          textures,
          marks,
          args: &args,
        },
      );
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

  /// The ends of every wire `options` keeps of `helpers` for `part`, a singled out model's box among them where
  /// `boxes` stands it (`StaticScene::get_object_box`).
  pub fn to_vertices(
    helpers: &LevelHelpers,
    options: &RenderHelperOptions,
    part: HelperPart,
    boxes: impl Fn(u32) -> Option<StaticObjectBox>,
  ) -> Vec<HelperVertex> {
    Self::list_kept(helpers, options, part)
      .flat_map(|object: &LevelHelperObject| {
        let look: HelperLook = HelperLook::of(object, options, part);
        let boxed: Option<StaticObjectBox> = object.model.filter(|_| look.is_singled()).and_then(&boxes);

        helpers.segments[object.segments.clone()]
          .iter()
          .copied()
          .chain(boxed.into_iter().flat_map(StaticObjectBox::to_edges))
          .flatten()
          .map(move |end: Vec3| HelperVertex {
            position: end.to_array(),
            color: look.color,
            fade: look.fade,
            object: object.id,
          })
      })
      .collect()
  }

  /// Every icon `options` keeps of `helpers` for `part`, drawn as its object's wires are.
  pub fn to_icons(helpers: &LevelHelpers, options: &RenderHelperOptions, part: HelperPart) -> Vec<HelperIconInstance> {
    Self::list_kept(helpers, options, part)
      .flat_map(|object: &LevelHelperObject| {
        let look: HelperLook = HelperLook::of(object, options, part);

        helpers.icons[object.icons.clone()]
          .iter()
          .map(move |icon: &LevelHelperIcon| HelperIconInstance {
            position: icon.center.to_array(),
            icon: icon.icon.get_index(),
            color: look.color,
            fade: look.fade,
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

impl ViewFeature for HelpersFeature {
  /// The wires' vertices, the icons and the marks.
  fn get_memory(&self) -> u64 {
    [&self.every, &self.singled]
      .into_iter()
      .flatten()
      .map(HelperBuild::get_memory)
      .sum::<u64>()
      + self.marks.as_ref().map_or(0, |marks| marks.buffer.size())
  }

  fn release(&mut self) {
    *self = Self::default();
  }
}

/// What a part of a view's helpers was built from: the scene's set by its version, the scene's spawned objects by how
/// many were added and removed so far, and the options it was built under.
#[derive(Clone, Debug, PartialEq)]
struct HelperBuildKey {
  version: u64,
  contents: usize,
  options: RenderHelperOptions,
}

impl HelperBuildKey {
  /// What `sources` hold, under `options`.
  fn of(sources: HelperSources<'_>, options: &RenderHelperOptions) -> Self {
    Self {
      version: sources.helpers.version,
      contents: sources.statics.get_contents(),
      options: options.clone(),
    }
  }
}

/// One part of a view's helpers as built: what it was built from, and its wires' and icons' buffers.
struct HelperBuild {
  key: HelperBuildKey,
  wires: Option<HelperBuffer>,
  icons: Option<HelperBuffer>,
}

impl HelperBuild {
  /// The part's buffers of `geometry`, built from `key`.
  fn new(device: &wgpu::Device, key: HelperBuildKey, geometry: HelperGeometry) -> Self {
    Self {
      key,
      wires: HelperBuffer::new(device, "helper wires", &geometry.wires),
      icons: HelperBuffer::new(device, "helper icons", &geometry.icons),
    }
  }

  /// `part`'s wires' ends and icons of `helpers` under `options`, its models' boxes where `boxes` stands them.
  fn build(
    helpers: &LevelHelpers,
    options: &RenderHelperOptions,
    part: HelperPart,
    boxes: impl Fn(u32) -> Option<StaticObjectBox>,
  ) -> HelperGeometry {
    HelperGeometry {
      wires: HelpersFeature::to_vertices(helpers, options, part, boxes),
      icons: HelpersFeature::to_icons(helpers, options, part),
    }
  }

  /// Whether `built` was built from `key`.
  fn is_current(built: &Option<Self>, key: &HelperBuildKey) -> bool {
    built.as_ref().is_some_and(|built| built.key == *key)
  }

  /// What `built` draws, nothing where it was never built.
  fn get_draws(built: &Option<Self>) -> HelperPartDraws<'_> {
    built
      .as_ref()
      .map_or_else(HelperPartDraws::default, |built| HelperPartDraws {
        wires: built.wires.as_ref(),
        icons: built.icons.as_ref(),
      })
  }

  fn get_memory(&self) -> u64 {
    [&self.wires, &self.icons]
      .into_iter()
      .flatten()
      .map(|buffer| buffer.buffer.size())
      .sum()
  }
}

/// One part's helpers before they are uploaded: its wires' ends, two a segment, and its icons, an instance each.
struct HelperGeometry {
  wires: Vec<HelperVertex>,
  icons: Vec<HelperIconInstance>,
}

/// The marked models' places and colours, and their bind group as the helper marks pass reads them.
struct HelperMarks {
  buffer: wgpu::Buffer,
  group: wgpu::BindGroup,
}

impl HelperMarks {
  /// `marks` uploaded and bound by `layout`.
  fn new(device: &wgpu::Device, marks: &[[u32; 2]], layout: &wgpu::BindGroupLayout) -> Self {
    let buffer: wgpu::Buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("helper marks"),
      contents: bytemuck::cast_slice(marks),
      usage: wgpu::BufferUsages::STORAGE,
    });
    let group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("helper marks"),
      layout,
      entries: &[wgpu::BindGroupEntry {
        binding: 0,
        resource: buffer.as_entire_binding(),
      }],
    });

    Self { buffer, group }
  }
}

/// How one object's helpers draw in a part: its colour with how it is singled out in alpha, nought for every kept
/// object's, then the distance it fades out by, none for a singled out one, and how strongly it shows hidden.
#[derive(Clone, Copy)]
struct HelperLook {
  color: [f32; 4],
  fade: [f32; 2],
}

impl HelperLook {
  /// How `object` draws under `options`, built for `part`.
  fn of(object: &LevelHelperObject, options: &RenderHelperOptions, part: HelperPart) -> Self {
    let singled: f32 = match part {
      HelperPart::Every => 0.0,
      HelperPart::Singled => HelpersFeature::get_singled(object, options),
    };
    let reach: f32 = if singled > 0.0 {
      0.0
    } else {
      object.draw_distance * options.distance_scale
    };

    Self {
      color: [object.color[0], object.color[1], object.color[2], singled],
      fade: [reach, options.occluded_strength],
    }
  }

  /// Whether it is singled out, which draws a model's box.
  fn is_singled(self) -> bool {
    self.color[3] > 0.0
  }
}
