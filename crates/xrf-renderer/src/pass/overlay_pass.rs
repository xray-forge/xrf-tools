use std::collections::HashMap;

use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext, RenderPipelineDescription, VertexLayout};

use crate::host::render_bundle::RenderBundle;
use crate::pass::overlay_parameters::OverlayParameters;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::helpers_feature::HelperDraws;
use crate::scene::level::helpers::helper_icon_atlas::HelperIconAtlas;
use crate::scene::level::helpers::helper_icon_instance::HelperIconInstance;
use crate::scene::level::helpers::helper_vertex::HelperVertex;
use crate::scene::level::level_overlays::LevelOverlays;

/// Draws a viewport's overlays into the window's pass after its frame: line segments and points, depth tested against
/// the scene by the shader since the window's pass holds no depth, the sun's disc, and the helpers' wires and icons,
/// blended by how they show behind the scene and how far off they stand.
pub struct OverlayPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  /// The helper icon atlas and its sampler, bound as the icons' third group.
  atlas_layout: wgpu::BindGroupLayout,
  /// None where the bundle holds no atlas it can read, and no icon is drawn.
  atlas: Option<wgpu::BindGroup>,
  /// The lines', the sun's, the points', the helpers' wires' and icons', one set a window format, built on first use.
  pipelines: HashMap<wgpu::TextureFormat, [wgpu::RenderPipeline; 5]>,
  generation: u64,
}

impl OverlayPass {
  pub fn new(
    queue: &wgpu::Queue,
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    bundle: &dyn RenderBundle,
  ) -> Self {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = OverlayParameters::create_layout(device);
    let atlas_layout: wgpu::BindGroupLayout = Self::create_atlas_layout(device);
    let atlas: Option<wgpu::BindGroup> = HelperIconAtlas::new(device, queue, bundle)
      .inspect_err(|error| log::error!("Helper icons not drawn: {error}"))
      .ok()
      .map(|atlas| {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
          label: Some("helper icon atlas"),
          layout: &atlas_layout,
          entries: &[
            wgpu::BindGroupEntry {
              binding: 0,
              resource: wgpu::BindingResource::TextureView(&atlas.view),
            },
            wgpu::BindGroupEntry {
              binding: 1,
              resource: wgpu::BindingResource::Sampler(&atlas.sampler),
            },
          ],
        })
      });

    Self {
      layout,
      view_layout: view_layout.clone(),
      atlas_layout,
      atlas,
      pipelines: HashMap::new(),
      generation: source.get_generation(),
    }
  }

  /// The helper icons' atlas, which their corners measure their cells by, and its filtering sampler.
  fn create_atlas_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("helper icon atlas"),
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
          ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
      ],
    })
  }

  /// Builds the pipelines again from a library reloaded since they were built, one a window format each; a broken edit
  /// keeps the last good ones.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() == self.generation {
      return;
    }

    self.generation = source.get_generation();

    let rebuilt: XrfResult<HashMap<wgpu::TextureFormat, [wgpu::RenderPipeline; 5]>> = self
      .pipelines
      .keys()
      .map(|format| Ok((*format, self.create_pipeline(source, *format)?)))
      .collect();

    match rebuilt {
      Ok(pipelines) => self.pipelines = pipelines,
      Err(error) => log::error!("Overlay shader rejected, drawing with the last one: {error}"),
    }
  }

  /// Builds the pipelines drawing into a window format, unless they are built.
  ///
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn prepare(&mut self, source: PipelineSource<'_>, format: wgpu::TextureFormat) -> XrfResult {
    if !self.pipelines.contains_key(&format) {
      let pipelines: [wgpu::RenderPipeline; 5] = self.create_pipeline(source, format)?;

      self.pipelines.insert(format, pipelines);
    }

    Ok(())
  }

  /// The lines', the sun's, the points', the helpers' wires' and icons' pipelines drawing into a window format.
  fn create_pipeline(
    &self,
    source: PipelineSource<'_>,
    format: wgpu::TextureFormat,
  ) -> XrfResult<[wgpu::RenderPipeline; 5]> {
    // The camera and the overlays' parameters every pipeline binds; only the icons bind the atlas, as a third group.
    let groups: [Option<&wgpu::BindGroupLayout>; 2] = [Some(&self.view_layout), Some(&self.layout)];
    let iconed: [Option<&wgpu::BindGroupLayout>; 3] = [groups[0], groups[1], Some(&self.atlas_layout)];
    let create = |(vertex, fragment): (&'static str, &'static str),
                  (topology, buffer): (wgpu::PrimitiveTopology, VertexLayout),
                  blend: Option<wgpu::BlendState>,
                  layouts: &[Option<&wgpu::BindGroupLayout>]|
     -> XrfResult<wgpu::RenderPipeline> {
      source.get_render_pipeline(
        &RenderPipelineDescription::new(vertex, "frame/overlay", vertex)
          .with_fragment(fragment)
          .with_bind_group_layouts(layouts)
          .with_vertex_layout(buffer)
          .with_primitive(wgpu::PrimitiveState {
            topology,
            ..Default::default()
          })
          .with_target(wgpu::ColorTargetState {
            format,
            blend,
            write_mask: wgpu::ColorWrites::ALL,
          }),
      )
    };
    let lines: wgpu::RenderPipeline = create(
      ("vs_line", "fs_line"),
      (
        wgpu::PrimitiveTopology::LineList,
        VertexLayout::new(
          (LevelOverlays::LINE_FLOATS * size_of::<f32>()) as u64,
          wgpu::VertexStepMode::Vertex,
          &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4],
        ),
      ),
      None,
      &groups,
    )?;
    let sun: wgpu::RenderPipeline = create(
      ("vs_sun", "fs_sun"),
      (
        wgpu::PrimitiveTopology::TriangleList,
        VertexLayout::new(
          (LevelOverlays::SUN_FLOATS * size_of::<f32>()) as u64,
          wgpu::VertexStepMode::Instance,
          &wgpu::vertex_attr_array![0 => Float32x4],
        ),
      ),
      None,
      &groups,
    )?;
    let points: wgpu::RenderPipeline = create(
      ("vs_point", "fs_point"),
      (
        wgpu::PrimitiveTopology::TriangleList,
        VertexLayout::new(
          (LevelOverlays::POINT_FLOATS * size_of::<f32>()) as u64,
          wgpu::VertexStepMode::Instance,
          &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4],
        ),
      ),
      None,
      &groups,
    )?;
    let helpers: wgpu::RenderPipeline = create(
      ("vs_helper", "fs_helper"),
      (
        wgpu::PrimitiveTopology::LineList,
        VertexLayout::new(
          HelperVertex::SIZE,
          wgpu::VertexStepMode::Vertex,
          &HelperVertex::ATTRIBUTES,
        ),
      ),
      Some(wgpu::BlendState::ALPHA_BLENDING),
      &groups,
    )?;

    let icons: wgpu::RenderPipeline = create(
      ("vs_helper_icon", "fs_helper_icon"),
      (
        wgpu::PrimitiveTopology::TriangleList,
        VertexLayout::new(
          HelperIconInstance::SIZE,
          wgpu::VertexStepMode::Instance,
          &HelperIconInstance::ATTRIBUTES,
        ),
      ),
      Some(wgpu::BlendState::ALPHA_BLENDING),
      &iconed,
    )?;

    Ok([lines, sun, points, helpers, icons])
  }

  /// Draws into the window's pass, whose viewport and scissor are the viewport's rectangle already: the overlays, then
  /// the helpers' wires, two vertices a wire, and their icons, six corners each, over them.
  pub fn draw(
    &self,
    context: &mut RasterContext<'_>,
    format: wgpu::TextureFormat,
    (view, parameters): (&ViewBinding, &OverlayParameters),
    (overlays, helpers): (&LevelOverlays, HelperDraws<'_>),
  ) {
    let Some([lines, sun, points, wires, icons]) = self.pipelines.get(&format) else {
      return;
    };

    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_bind_group(0, &view.bind_group, &[]);

    for (buffer, count) in [&overlays.lines, &overlays.skeleton_lines].into_iter().flatten() {
      pass.set_pipeline(lines);
      pass.set_vertex_buffer(0, buffer.slice(..));
      pass.draw(0..*count, 0..1);
    }

    if let Some((buffer, count)) = &overlays.points {
      pass.set_pipeline(points);
      pass.set_vertex_buffer(0, buffer.slice(..));
      pass.draw(0..6, 0..*count);
    }

    if let Some(buffer) = &overlays.sun {
      pass.set_pipeline(sun);
      pass.set_vertex_buffer(0, buffer.slice(..));
      pass.draw(0..6, 0..1);
    }

    self.draw_helper_parts(pass, (wires, icons), helpers);
  }

  /// Draws the helpers alone into a pass whose viewport is the view's whole picture, as a screenshot including them is.
  pub fn draw_helpers(
    &self,
    context: &mut RasterContext<'_>,
    format: wgpu::TextureFormat,
    (view, parameters): (&ViewBinding, &OverlayParameters),
    helpers: HelperDraws<'_>,
  ) {
    let Some([.., wires, icons]) = self.pipelines.get(&format) else {
      return;
    };

    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_bind_group(0, &view.bind_group, &[]);
    self.draw_helper_parts(pass, (wires, icons), helpers);
  }

  /// Every kept object's wires and icons, then the singled out ones' over them, two vertices a wire and six corners an
  /// icon.
  fn draw_helper_parts(
    &self,
    pass: &mut wgpu::RenderPass<'static>,
    (wires, icons): (&wgpu::RenderPipeline, &wgpu::RenderPipeline),
    helpers: HelperDraws<'_>,
  ) {
    for (part_wires, part_icons) in helpers.wires.into_iter().zip(helpers.icons) {
      if let Some((buffer, count)) = part_wires {
        pass.set_pipeline(wires);
        pass.set_vertex_buffer(0, buffer.slice(..));
        pass.draw(0..count, 0..1);
      }

      if let (Some((buffer, count)), Some(atlas)) = (part_icons, &self.atlas) {
        pass.set_pipeline(icons);
        pass.set_bind_group(2, atlas, &[]);
        pass.set_vertex_buffer(0, buffer.slice(..));
        pass.draw(0..6, 0..count);
      }
    }
  }
}
