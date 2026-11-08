use glam::Vec3;
use wgpu::util::DeviceExt;
use xrf_renderer_core::{
  FrameGraph, GraphBuffer, GraphBufferAccess, GraphColorAttachment, GraphDepthAttachment, GraphTexture,
  GraphTextureDescriptor,
};

use crate::contract::overlay::render_helper_options::RenderHelperOptions;
use crate::feature::page::present_marks::PresentMarks;
use crate::feature::scene::statics::draw::static_draws::StaticDraws;
use crate::feature::scene::statics::draw::static_mark_draw::StaticMarkDraw;
use crate::feature::view::view_feature::ViewFeature;
use crate::frame::targets::view_targets::ViewTargets;
use crate::scene::level::page::helpers::gpu::helper_buffer::HelperBuffer;
use crate::scene::level::page::helpers::gpu::helper_icon_instance::HelperIconInstance;
use crate::scene::level::page::helpers::gpu::helper_vertex::HelperVertex;
use crate::scene::level::page::helpers::level_helper_icon::LevelHelperIcon;
use crate::scene::level::page::helpers::level_helper_object::LevelHelperObject;
use crate::scene::level::page::helpers::level_helpers::LevelHelpers;
use crate::scene::level::page::helpers::view::helper_draws::HelperDraws;
use crate::scene::level::page::helpers::view::helper_marks_pass::HelperMarksPass;
use crate::scene::level::page::helpers::view::helper_part::HelperPart;
use crate::scene::level::page::helpers::view::helper_part_draws::HelperPartDraws;
use crate::scene::level::page::helpers::view::helper_sources::HelperSources;
use crate::scene::static_scene::item::object::static_object_box::StaticObjectBox;

/// What a view draws of its scene's helpers: the wires and icons its options keep, which the window pass draws and a
/// pick draws again, and the spawned models they mark, which the helper marks pass draws in their colours for the
/// present to tint and outline. Every kept object's are built again only when the scene's set, its spawned objects or
/// the options the selection and hovering aside change; the selected, hovered and sounding objects' are built apart and
/// drawn over them, so the pointer moving over the scene, or a sound starting, rebuilds a handful of wires rather than
/// every helper.
#[derive(Default)]
pub(crate) struct HelpersFeature {
  /// Every kept object's wires and icons, none singled out.
  every: Option<HelperBuild>,
  /// The selected, hovered and sounding objects' wires, the selected and hovered models' boxes among them, and icons,
  /// lifted and never dimmed behind what is drawn; the selected and hovered never fade.
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
  pub(crate) fn set(&mut self, device: &wgpu::Device, sources: HelperSources<'_>) {
    let HelperSources {
      helpers,
      sounding,
      statics,
      options,
      marks_layout,
    } = sources;
    let boxes = |object: u32| statics.get_object_box(object);

    // Each part is compared with what it was built from before anything is copied, as most frames change nothing.
    if !HelperBuild::is_current(self.singled.as_ref(), sources, HelperPart::Singled) {
      self.singled = Some(HelperBuild::new(
        device,
        HelperBuildKey::of(sources, options.clone(), sounding),
        HelperBuild::build(helpers, options, sounding, HelperPart::Singled, boxes),
      ));
    }

    if HelperBuild::is_current(self.every.as_ref(), sources, HelperPart::Every) {
      return;
    }

    let unsingled: RenderHelperOptions = RenderHelperOptions {
      selected: Vec::new(),
      hovered: None,
      ..options.clone()
    };
    let marks: Vec<[u32; 2]> = Self::list_marks(helpers, &unsingled, |object| statics.get_object_place(object));
    let geometry: HelperGeometry = HelperBuild::build(helpers, &unsingled, &[], HelperPart::Every, boxes);

    self.is_rest_faded = options.is_shown && options.is_rest_faded;
    self.every = Some(HelperBuild::new(
      device,
      HelperBuildKey::of(sources, unsingled, &[]),
      geometry,
    ));
    self.marks = (!marks.is_empty()).then(|| HelperMarks::new(device, &marks, marks_layout));
  }

  /// The wires and icons to draw: every kept object's, then the singled out ones' over them.
  pub(crate) fn get_draws(&self) -> HelperDraws<'_> {
    HelperDraws {
      every: HelperBuild::get_draws(self.every.as_ref()),
      singled: HelperBuild::get_draws(self.singled.as_ref()),
    }
  }

  /// The marked models' places and colours as the helper marks pass binds them; none where no model is marked.
  pub(crate) fn get_marks(&self) -> Option<&wgpu::BindGroup> {
    self.marks.as_ref().map(|marks| &marks.group)
  }

  /// What the present lays over the frame of the marks: whether any model is marked, and whether the rest fades.
  pub(crate) fn get_present_marks(&self) -> PresentMarks {
    PresentMarks {
      is_marked: self.marks.is_some(),
      is_rest_faded: self.is_rest_faded,
    }
  }

  /// Adds the helper marks pass where the view marks models: the spawned models' visible clusters drawn again over the
  /// view's depth into a marks target of the drawn size, each in its helper's colour where its fragment is the one kept;
  /// answers the target, none where no model is marked.
  pub(crate) fn add_marks_pass<'a>(
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
  /// helpers are off; for the singled out part, those selected, hovered or among `sounding`.
  fn list_kept<'a>(
    helpers: &'a LevelHelpers,
    options: &'a RenderHelperOptions,
    sounding: &'a [u32],
    part: HelperPart,
  ) -> impl Iterator<Item = &'a LevelHelperObject> {
    helpers.objects.iter().filter(move |object| {
      options.is_shown
        && object.kind < u32::BITS
        && options.kinds & (1 << object.kind) != 0
        && options.hidden.binary_search(&object.id).is_err()
        && (part == HelperPart::Every || Self::get_singled(object, options, sounding) > 0.0)
    })
  }

  /// How `object` is singled out under `options`, its sound playing where `sounding` holds it:
  /// [`HelperVertex::SELECTED`], [`HelperVertex::HOVERED`], [`HelperVertex::SOUNDING`] or nought.
  fn get_singled(object: &LevelHelperObject, options: &RenderHelperOptions, sounding: &[u32]) -> f32 {
    if options.selected.binary_search(&object.id).is_ok() {
      HelperVertex::SELECTED
    } else if options.hovered == Some(object.id) {
      HelperVertex::HOVERED
    } else if sounding.binary_search(&object.id).is_ok() {
      HelperVertex::SOUNDING
    } else {
      0.0
    }
  }

  /// The ends of every wire `options` keeps of `helpers` for `part`, `sounding` the objects whose sounds play, a
  /// selected or hovered model's box among them where `boxes` stands it (`StaticScene::get_object_box`).
  pub(crate) fn list_vertices(
    helpers: &LevelHelpers,
    options: &RenderHelperOptions,
    sounding: &[u32],
    part: HelperPart,
    boxes: impl Fn(u32) -> Option<StaticObjectBox>,
  ) -> Vec<HelperVertex> {
    Self::list_kept(helpers, options, sounding, part)
      .flat_map(|object: &LevelHelperObject| {
        let look: HelperLook = HelperLook::of(object, options, sounding, part);
        let boxed: Option<StaticObjectBox> = object.model.filter(|_| look.is_picked).and_then(&boxes);

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
  pub(crate) fn list_icons(
    helpers: &LevelHelpers,
    options: &RenderHelperOptions,
    sounding: &[u32],
    part: HelperPart,
  ) -> Vec<HelperIconInstance> {
    Self::list_kept(helpers, options, sounding, part)
      .flat_map(|object: &LevelHelperObject| {
        let look: HelperLook = HelperLook::of(object, options, sounding, part);

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
  pub(crate) fn list_marks(
    helpers: &LevelHelpers,
    options: &RenderHelperOptions,
    places: impl Fn(u32) -> Option<u32>,
  ) -> Vec<[u32; 2]> {
    let mut marks: Vec<[u32; 2]> = Self::list_kept(helpers, options, &[], HelperPart::Every)
      .filter_map(|object: &LevelHelperObject| {
        let [r, g, b] = object
          .color
          .map(|channel| (channel.clamp(0.0, 1.0) * 255.0).round() as u32);

        Some([places(object.model?)?, r | (g << 8) | (b << 16) | (255 << 24)])
      })
      .collect();

    marks.sort_unstable_by_key(|[place, _]| *place);
    marks
  }
}

impl ViewFeature for HelpersFeature {
  fn release(&mut self) {
    *self = Self::default();
  }
}

/// What a part of a view's helpers was built from: the scene's set by its version, the scene's spawned objects by how
/// many were added and removed so far, the options it was built under, and the objects it drew sounding.
#[derive(Debug)]
struct HelperBuildKey {
  version: u64,
  contents: usize,
  options: RenderHelperOptions,
  sounding: Vec<u32>,
}

impl HelperBuildKey {
  /// What `sources` hold, under `options`, with `sounding` drawn lit.
  fn of(sources: HelperSources<'_>, options: RenderHelperOptions, sounding: &[u32]) -> Self {
    Self {
      version: sources.helpers.version,
      contents: sources.statics.get_contents(),
      options,
      sounding: sounding.to_vec(),
    }
  }

  /// Whether `part` built from it would be built from `sources` alike, copying nothing: the singled out objects' under
  /// the options and sounds as they are, every kept object's under the options with nothing selected or hovered and
  /// nothing sounding.
  fn is_of(&self, sources: HelperSources<'_>, part: HelperPart) -> bool {
    self.version == sources.helpers.version
      && self.contents == sources.statics.get_contents()
      && match part {
        HelperPart::Singled => self.options == *sources.options && self.sounding == sources.sounding,
        HelperPart::Every => Self::is_unsingled_of(&self.options, sources.options) && self.sounding.is_empty(),
      }
  }

  /// Whether `built` is `options` with nothing selected or hovered.
  fn is_unsingled_of(built: &RenderHelperOptions, options: &RenderHelperOptions) -> bool {
    let RenderHelperOptions {
      is_shown,
      kinds,
      hidden,
      selected: _,
      hovered: _,
      occluded_strength,
      distance_scale,
      is_rest_faded,
    } = options;

    built.selected.is_empty()
      && built.hovered.is_none()
      && built.is_shown == *is_shown
      && built.kinds == *kinds
      && built.hidden == *hidden
      && built.occluded_strength == *occluded_strength
      && built.distance_scale == *distance_scale
      && built.is_rest_faded == *is_rest_faded
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

  /// `part`'s wires' ends and icons of `helpers` under `options`, `sounding` lit, its models' boxes where `boxes` stands
  /// them.
  fn build(
    helpers: &LevelHelpers,
    options: &RenderHelperOptions,
    sounding: &[u32],
    part: HelperPart,
    boxes: impl Fn(u32) -> Option<StaticObjectBox>,
  ) -> HelperGeometry {
    HelperGeometry {
      wires: HelpersFeature::list_vertices(helpers, options, sounding, part, boxes),
      icons: HelpersFeature::list_icons(helpers, options, sounding, part),
    }
  }

  /// Whether `built` stands for `part` of what `sources` hold now.
  fn is_current(built: Option<&Self>, sources: HelperSources<'_>, part: HelperPart) -> bool {
    built.is_some_and(|built| built.key.is_of(sources, part))
  }

  /// What `built` draws, nothing where it was never built.
  fn get_draws(built: Option<&Self>) -> HelperPartDraws<'_> {
    built.map_or_else(HelperPartDraws::default, |built| HelperPartDraws {
      wires: built.wires.as_ref(),
      icons: built.icons.as_ref(),
    })
  }
}

/// One part's helpers before they are uploaded: its wires' ends, two a segment, and its icons, an instance each.
struct HelperGeometry {
  wires: Vec<HelperVertex>,
  icons: Vec<HelperIconInstance>,
}

/// The marked models' places and colours, bound as the helper marks pass reads them; the bind group keeps the buffer.
struct HelperMarks {
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

    Self { group }
  }
}

/// How one object's helpers draw in a part: its colour with how it is singled out in alpha, nought for every kept
/// object's, then the distance it fades out by, none for a selected or hovered one, and how strongly it shows hidden.
#[derive(Clone, Copy)]
struct HelperLook {
  color: [f32; 4],
  fade: [f32; 2],
  /// Whether it is selected or hovered, which draws a model's box.
  is_picked: bool,
}

impl HelperLook {
  /// How `object` draws under `options`, `sounding` lit, built for `part`.
  fn of(object: &LevelHelperObject, options: &RenderHelperOptions, sounding: &[u32], part: HelperPart) -> Self {
    let singled: f32 = match part {
      HelperPart::Every => 0.0,
      HelperPart::Singled => HelpersFeature::get_singled(object, options, sounding),
    };
    let is_picked: bool = part == HelperPart::Singled && HelpersFeature::get_singled(object, options, &[]) > 0.0;
    let reach: f32 = if is_picked {
      0.0
    } else {
      object.draw_distance * options.distance_scale
    };

    Self {
      color: [object.color[0], object.color[1], object.color[2], singled],
      fade: [reach, options.occluded_strength],
      is_picked,
    }
  }
}
