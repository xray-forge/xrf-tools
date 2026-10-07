use glam::UVec2;
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphRuntime, GraphTexture, ShaderAtomicU32, StorageArray, StorageArrayMut,
  UniformBinding,
};

use crate::pass::static_cull_parameters::StaticCullParameters;
use crate::pass::static_cull_params::StaticCullParams;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::static_draws::StaticDraws;
use crate::pass::static_impostor_parameters::StaticImpostorParameters;
use crate::pass::static_occlusion_uniform::StaticOcclusionUniform;
use crate::pass::wind_uniform::WindUniform;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_scene::StaticScene;

/// The static scene's buffers as a frame graph imports them, once a frame, with the uniforms its culls and draws read
/// pushed: what every cull and static draw of the frame binds, each view adding its own list and draw arguments.
#[derive(Clone, Copy)]
pub struct StaticSceneHandles {
  pub clusters: GraphBuffer,
  pub spheres: GraphBuffer,
  pub slots: GraphBuffer,
  pub places: GraphBuffer,
  pub rows: GraphBuffer,
  pub regions: GraphBuffer,
  pub surfaces: GraphBuffer,
  pub indices: GraphBuffer,
  pub words: [GraphBuffer; StaticLayout::COUNT],
  pub skins: GraphBuffer,
  pub bones: GraphBuffer,
  pub impostors: GraphBuffer,
  pub corners: GraphBuffer,
  pub terms: GraphBuffer,
  pub impostor_list: GraphBuffer,
  /// The camera's visible list and draw arguments, its late phase's, and what a cull's arguments start from.
  pub lists: GraphBuffer,
  pub args: GraphBuffer,
  pub candidates: GraphBuffer,
  pub late: GraphBuffer,
  pub late_dispatch: GraphBuffer,
  pub args_template: GraphBuffer,
  /// The camera's depth pyramid, which its early cull tests against and its late cull after reducing it again.
  pub pyramid: GraphTexture,
  pub cull: UniformBinding<StaticCullParams>,
  pub occlusion: UniformBinding<StaticOcclusionUniform>,
  pub wind: UniformBinding<WindUniform>,
}

impl StaticSceneHandles {
  pub fn import<'r>(
    (graph, bindings, runtime): (&mut FrameGraph<'_>, &mut GraphBindings<'r>, &mut GraphRuntime),
    scene: &'r StaticScene,
    pyramid: &'r wgpu::TextureView,
    (cull, occlusion, wind): (&StaticCullParams, &StaticOcclusionUniform, &WindUniform),
  ) -> Self {
    let mut import = |label: &'static str, buffer: &'r wgpu::Buffer| bindings.import_buffer(graph, label, buffer);
    let handles = (
      import("static clusters", scene.clusters.get_buffer()),
      import("static spheres", scene.clusters.get_lane(StaticScene::CLUSTER_SPHERES)),
      import("static slots", scene.slots.get_buffer()),
      import("static places", scene.places.get_buffer()),
      import("static rows", scene.rows.get_buffer()),
      import("static regions", &scene.regions),
      import("static surfaces", scene.surfaces.get_buffer()),
      import("static indices", scene.indices.get_buffer()),
    );
    let words: [GraphBuffer; StaticLayout::COUNT] = [
      import("static baked vertices", scene.words[0].get_buffer()),
      import("static tree vertices", scene.words[1].get_buffer()),
      import("static model vertices", scene.words[2].get_buffer()),
    ];
    let skinning: (GraphBuffer, GraphBuffer) = (
      import("static model skins", scene.skins.get_buffer()),
      import("static bones", scene.bones.get_buffer()),
    );
    let impostors: [GraphBuffer; 4] = [
      import("static impostors", scene.impostors.get_buffer()),
      import(
        "static impostor corners",
        scene.impostors.get_lane(StaticScene::IMPOSTOR_CORNERS),
      ),
      import(
        "static impostor terms",
        scene.impostors.get_lane(StaticScene::IMPOSTOR_TERMS),
      ),
      import(
        "static impostor list",
        scene.impostors.get_lane(StaticScene::IMPOSTOR_LIST),
      ),
    ];
    let culled: [GraphBuffer; 6] = [
      import("static lists", scene.lists.get_buffer()),
      import("static draw arguments", &scene.args),
      import("static candidates", scene.candidates.get_buffer()),
      import("static late arguments", &scene.late),
      import("static late dispatch", &scene.late_dispatch),
      import("static draw arguments template", &scene.args_template),
    ];
    let (clusters, spheres, slots, places, rows, regions, surfaces, indices) = handles;

    Self {
      clusters,
      spheres,
      slots,
      places,
      rows,
      regions,
      surfaces,
      indices,
      words,
      skins: skinning.0,
      bones: skinning.1,
      impostors: impostors[0],
      corners: impostors[1],
      terms: impostors[2],
      impostor_list: impostors[3],
      lists: culled[0],
      args: culled[1],
      candidates: culled[2],
      late: culled[3],
      late_dispatch: culled[4],
      args_template: culled[5],
      pyramid: bindings.import_view(graph, "depth pyramid", pyramid),
      cull: runtime.push_uniform(cull),
      occlusion: runtime.push_uniform(occlusion),
      wind: runtime.push_uniform(wind),
    }
  }

  /// A cull into a view's own visible list and draw arguments: the camera's, or a shadow's.
  pub fn get_cull_parameters(
    &self,
    lists: StorageArrayMut<UVec2>,
    args: StorageArrayMut<ShaderAtomicU32>,
  ) -> StaticCullParameters {
    StaticCullParameters {
      clusters: StorageArray::new(self.clusters),
      spheres: StorageArray::new(self.spheres),
      slots: StorageArray::new(self.slots),
      places: StorageArray::new(self.places),
      rows: StorageArray::new(self.rows),
      regions: StorageArray::new(self.regions),
      lists,
      args,
      params: self.cull,
      candidates: StorageArrayMut::new(self.candidates),
      late: StorageArrayMut::new(self.late),
      pyramid: self.pyramid,
      occlusion: self.occlusion,
      impostors: StorageArray::new(self.impostors),
      terms: StorageArrayMut::new(self.terms),
      impostor_list: StorageArrayMut::new(self.impostor_list),
    }
  }

  /// The camera's cull, into the scene's own list and arguments.
  pub fn get_camera_cull(&self) -> StaticCullParameters {
    self.get_cull_parameters(StorageArrayMut::new(self.lists), StorageArrayMut::new(self.args))
  }

  /// Each layout's draws of what a view's visible list holds, by `StaticLayout::get_index`.
  pub fn get_layout_draws(&self, lists: StorageArray<UVec2>) -> [StaticDrawParameters; StaticLayout::COUNT] {
    self.words.map(|words| StaticDrawParameters {
      clusters: StorageArray::new(self.clusters),
      slots: StorageArray::new(self.slots),
      places: StorageArray::new(self.places),
      surfaces: StorageArray::new(self.surfaces),
      indices: StorageArray::new(self.indices),
      lists,
      words: StorageArray::new(words),
      wind: self.wind,
      skins: StorageArray::new(self.skins),
      bones: StorageArray::new(self.bones),
    })
  }

  /// The camera's draws of what its cull listed, the impostors' too.
  pub fn get_camera_draws(&self) -> StaticDraws {
    StaticDraws {
      layouts: self.get_layout_draws(StorageArray::new(self.lists)),
      impostors: StaticImpostorParameters {
        impostors: StorageArray::new(self.impostors),
        corners: StorageArray::new(self.corners),
        terms: StorageArray::new(self.terms),
        impostor_list: StorageArray::new(self.impostor_list),
        surfaces: StorageArray::new(self.surfaces),
      },
    }
  }
}
