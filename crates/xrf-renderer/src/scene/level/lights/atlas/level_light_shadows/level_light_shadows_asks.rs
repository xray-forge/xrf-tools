use glam::{Vec3, Vec4};
use xrf_math::EPS_S;
use xrf_visual::{LightDescription, LightKind};

use crate::camera::camera_view::CameraView;
use crate::feature::scene::drawable::casters::light_casters::LightCasters;
use crate::feature::scene::drawable::drawable_kinds::DrawableKinds;
use crate::lighting::lamp::light_basis::LightBasis;
use crate::lighting::lamp::light_shadow_size::{
  LIGHT_SHADOW_MIN_SIZE, LIGHT_SHADOW_POINT_CONE, LIGHT_SHADOW_POINT_FACES, LIGHT_SHADOW_WIDENING,
  get_light_shadow_size, get_light_shadow_tile_size,
};
use crate::scene::level::level_time::LevelTime;
use crate::scene::level::lights::atlas::level_light_shadows::{
  DEFAULT_NEAR, FaceCandidate, FaceKey, FaceOrder, FaceSet, GROW, LevelLightShadows, SHRINK, STALE_WAIT_MOST,
  SWAY_WAIT_MOST,
};
use crate::scene::level::lights::atlas::light_face_rank::LightFaceRank;
use crate::scene::level::lights::atlas::light_shadow_ask::LightShadowAsk;
use crate::scene::level::lights::atlas::light_shadow_entry::LightShadowEntry;
use crate::scene::level::lights::atlas::light_shadow_face::LightShadowFace;
use crate::scene::level::lights::atlas::light_shadow_set::LightShadowSet;
use crate::scene::level::lights::atlas::shadow_tile::ShadowTile;
use crate::scene::level::lights::atlas::shadow_tile_allocator::ShadowTileAllocator;

impl LevelLightShadows {
  /// Takes a shadowed light in view this frame, nearest first, or one out of view kept ready, after every one in view:
  /// gives it faces where it has none at the size it wants, and puts forward those of its faces wanting a draw. One
  /// kept ready keeps the faces it has, and takes new ones only from free room.
  pub(crate) fn ask(&mut self, ask: &LightShadowAsk<'_>) {
    let LightShadowAsk {
      light,
      basis,
      eye,
      forward,
      ..
    } = *ask;
    let is_spot: bool = matches!(light.kind, LightKind::Spot);
    let spatial: Vec4 = basis.get_spatial_sphere(light);
    let distance: f32 = (eye.distance(spatial.truncate()) - spatial.w).max(0.0);
    let duel: f32 = if is_spot {
      1.0 - 0.5 * forward.dot(basis.direction)
    } else {
      1.0
    };
    let cone: f32 = if is_spot { light.cone } else { LIGHT_SHADOW_POINT_CONE };
    let color: Vec3 = Vec3::from(light.color);
    let wanted: f32 = get_light_shadow_size(
      light.range,
      distance,
      LightBasis::get_light_intensity(color),
      duel,
      cone,
    );
    let Some(asked) = self.choose_size(ask, wanted) else {
      return;
    };

    self.replace_set(ask, asked);
    self.push_candidates(ask, spatial, distance, cone);
  }

  /// The size of the faces a light asks for, from the texels it wants: the size it has while that stays near enough,
  /// else the nearest tile size; none where it asks nothing this frame, being kept ready already or asked for already
  /// by a nearer view.
  fn choose_size(&mut self, ask: &LightShadowAsk<'_>, wanted: f32) -> Option<u32> {
    let frame: u64 = self.frame;
    let entry: &mut LightShadowEntry = self.entries.entry(ask.handle).or_default();

    // Kept ready out of view once a frame, and never over a view's ask.
    if !ask.is_in_view {
      if entry.seen == frame || entry.warmed == frame {
        return None;
      }

      entry.warmed = frame;
    }

    let is_asked: bool = ask.is_in_view && entry.asked == frame;
    let current: Option<u32> = entry.next.as_ref().or(entry.shown.as_ref()).map(|set| set.size);
    let asked: u32 = match current {
      Some(size) if !ask.is_in_view || (wanted <= size as f32 * GROW && wanted >= size as f32 * SHRINK) => size,
      _ => get_light_shadow_tile_size(wanted),
    };

    // Another view of the scene asked for it this frame already: the size the nearer one wants stands.
    if is_asked && current.is_some_and(|size| asked <= size) {
      return None;
    }

    if ask.is_in_view {
      entry.seen = frame;
      entry.asked = frame;
    }

    Some(asked)
  }

  /// Gives a light faces of the `asked` size: drawing them already, it keeps those; back at the size it shows while
  /// another was being drawn, the shown faces stand and the other goes; at a size it neither shows nor draws, new faces
  /// replace those it draws. Faces given smaller squares than their size, the atlas too full when they were made, take
  /// larger ones in their place once room can be made (`grow_set`), so a light shrunk under pressure grows back.
  fn replace_set(&mut self, ask: &LightShadowAsk<'_>, asked: u32) {
    // Out of the map while the faces are made, which may evict another light's.
    let Some(mut entry) = self.entries.remove(&ask.handle) else {
      return;
    };

    if let Some(next) = entry.next.as_ref().filter(|set| set.size == asked) {
      if next.is_shrunk()
        && let Some(grown) = self.grow_set(ask, next)
        && let Some(next) = entry.next.replace(grown)
      {
        Self::release_set(&mut self.allocator, next);
      }
    } else if let Some(shown) = entry.shown.as_ref().filter(|set| set.size == asked) {
      let grown: Option<LightShadowSet> = if shown.is_shrunk() {
        self.grow_set(ask, shown)
      } else {
        None
      };

      if let Some(next) = std::mem::replace(&mut entry.next, grown) {
        Self::release_set(&mut self.allocator, next);
      }
    } else {
      if let Some(next) = entry.next.take() {
        Self::release_set(&mut self.allocator, next);
      }

      entry.next = self.create_set(ask, asked);
    }

    self.entries.insert(ask.handle, entry);
  }

  /// Faces at `shrunk`'s size, or at the largest square under it that is larger than its faces' squares, room made from
  /// the lights out of view longest where a view sees it, as for new faces; none where no larger square fits yet.
  fn grow_set(&mut self, ask: &LightShadowAsk<'_>, shrunk: &LightShadowSet) -> Option<LightShadowSet> {
    let least: u32 = shrunk.faces.first().map_or(shrunk.size, |face| face.tile.size * 2);

    self.create_set_within(ask, shrunk.size, least)
  }

  /// Puts forward the faces of a light wanting a draw: those holding nothing yet or changed under, those its light
  /// carried, and those the sway or a drawable kind's casters move in: the light's bounding sphere (`spatial`),
  /// `distance` metres from the eye, its cone as `cone`.
  fn push_candidates(&mut self, ask: &LightShadowAsk<'_>, spatial: Vec4, distance: f32, cone: f32) {
    let LightShadowAsk {
      handle,
      light,
      basis,
      eye,
      is_in_view,
      is_driving,
      statics,
      sway,
      casters,
      ..
    } = *ask;
    let Some(entry) = self.entries.get_mut(&handle) else {
      return;
    };

    let near: f32 = entry
      .next
      .as_ref()
      .or(entry.shown.as_ref())
      .map_or(DEFAULT_NEAR, |set| set.near);

    // Only an item coming or going within the light's reach changes what its faces hold or what sways in it.
    let contents: usize = statics.get_contents();
    let touches = |sphere: Vec4| spatial.truncate().distance(sphere.truncate()) < spatial.w + sphere.w;

    if entry
      .swaying_contents
      .is_none_or(|at| statics.is_changed_since(at, touches))
    {
      entry.swaying_contents = Some(contents);
      entry.swaying = Self::measure_swaying(basis.position, spatial, near, sway.places);
    }

    // A kind's casters stand about the eye: a light near it whose reach meets them casts their shadow. The faces are
    // drawn with the scene's driving view's casters, so only its asks say which lights cast them; another view's leave
    // that be.
    if is_driving {
      let is_cast = |casters: &&LightCasters| {
        distance < casters.light_distance && eye.distance(spatial.truncate()) < spatial.w + casters.reach
      };

      entry.casting = casters.iter().filter(is_cast).map(|casters| casters.kind).collect();
      entry.casting_lean = casters
        .iter()
        .filter(is_cast)
        .map(|casters| casters.lean)
        .fold(0.0, f32::max);
    }

    let casting: DrawableKinds = entry.casting;
    let swaying: f32 = entry.swaying.max(entry.casting_lean / near);

    self.is_swaying |= sway.amplitude * swaying > 0.0;
    // How wide a face's texels are, for each metre from the light, times the square's side.
    let spread: f32 = 2.0 * f32::midpoint(cone, LIGHT_SHADOW_WIDENING).tan();

    for (set_kind, set) in [(FaceSet::Shown, &mut entry.shown), (FaceSet::Next, &mut entry.next)] {
      if let Some(set) = set {
        for (face, state) in set.faces.iter_mut().enumerate() {
          let key: FaceKey = FaceKey {
            handle,
            set: set_kind,
            face,
          };
          // A light a motion carries has moved off where its faces look from: each looks from where it stands now,
          // and is drawn again as a swaying one is, lighting with what it holds meanwhile.
          let is_moved: bool = state.view.position != basis.position;

          if is_moved {
            state.view = Self::get_face_view(light, basis, face, set.near, set.far);
            state.drawn_static = None;
          }

          // Out of view, nothing it holds shows: it waits as long as a swaying face.
          let (is_away, stale_wait, moved_wait): (bool, u32, u32) = if is_in_view {
            (false, STALE_WAIT_MOST, 0)
          } else {
            (true, SWAY_WAIT_MOST, SWAY_WAIT_MOST)
          };

          let candidate = |is_current: bool, by: LightFaceRank, wait_most: u32| FaceCandidate {
            order: FaceOrder {
              is_away,
              is_current,
              by,
            },
            key,
            wait_most,
          };

          if state.drawn.is_none_or(|drawn| statics.is_changed_since(drawn, touches)) || state.casting != casting {
            state.drawn_static = None;
            self
              .candidates
              .push(candidate(false, LightFaceRank::Distance(distance), stale_wait));
          } else if is_moved {
            self
              .candidates
              .push(candidate(true, LightFaceRank::DrawnAt(state.drawn_at), moved_wait));
          } else if sway.is_redrawn(swaying, spread / state.tile.size as f32, state.drawn_at) {
            self
              .candidates
              .push(candidate(true, LightFaceRank::DrawnAt(state.drawn_at), SWAY_WAIT_MOST));
          }
        }
      }
    }
  }

  /// The faces a light asks for (`ask`) at a size, room made from the lights out of view longest where a view sees it,
  /// and at smaller squares where there is still none.
  fn create_set(&mut self, ask: &LightShadowAsk<'_>, size: u32) -> Option<LightShadowSet> {
    self.create_set_within(ask, size, LIGHT_SHADOW_MIN_SIZE)
  }

  /// The faces a light asks for (`ask`) at a size, room made from the lights out of view longest where a view sees it,
  /// and at smaller squares where there is still none, none under `least`.
  fn create_set_within(&mut self, ask: &LightShadowAsk<'_>, size: u32, least: u32) -> Option<LightShadowSet> {
    let LightShadowAsk {
      light,
      basis,
      is_in_view: may_evict,
      ..
    } = *ask;
    let is_spot: bool = matches!(light.kind, LightKind::Spot);
    let count: usize = if is_spot { 1 } else { LIGHT_SHADOW_POINT_FACES.len() };
    let near: f32 = if light.near > 0.0 { light.near } else { DEFAULT_NEAR };
    let far: f32 = light.range + light.range_jitter + EPS_S;
    let mut side: u32 = size;

    loop {
      if let Some(tiles) = self.allocate(count, side) {
        let faces: Vec<LightShadowFace> = tiles
          .into_iter()
          .enumerate()
          .map(|(face, tile)| LightShadowFace {
            tile,
            view: Self::get_face_view(light, basis, face, near, far),
            drawn: None,
            drawn_static: None,
            drawn_at: LevelTime::default(),
            waited: 0,
            casting: DrawableKinds::default(),
          })
          .collect();

        return Some(LightShadowSet { size, near, far, faces });
      }

      if !(may_evict && self.evict_oldest()) {
        if side <= least {
          return None;
        }

        side /= 2;
      }
    }
  }

  /// Squares for every face of a light, all or none.
  fn allocate(&mut self, count: usize, side: u32) -> Option<Vec<ShadowTile>> {
    let mut tiles: Vec<ShadowTile> = Vec::with_capacity(count);

    for _ in 0..count {
      match self.allocator.allocate(side) {
        Some(tile) => tiles.push(tile),
        None => {
          tiles.into_iter().for_each(|tile| self.allocator.release(tile));

          return None;
        }
      }
    }

    Some(tiles)
  }

  /// Lets the light out of view longest give its faces back; false where every light holding any is in view.
  fn evict_oldest(&mut self) -> bool {
    let Some(index) = self
      .entries
      .iter()
      .filter(|(_, entry)| entry.seen < self.frame && (entry.shown.is_some() || entry.next.is_some()))
      .min_by_key(|(_, entry)| entry.seen)
      .map(|(index, _)| *index)
    else {
      return false;
    };

    if let Some(entry) = self.entries.remove(&index) {
      entry
        .shown
        .into_iter()
        .chain(entry.next)
        .for_each(|set| Self::release_set(&mut self.allocator, set));
    }

    true
  }

  pub(super) fn release_set(allocator: &mut ShadowTileAllocator, set: LightShadowSet) {
    set.faces.into_iter().for_each(|face| allocator.release(face.tile));
  }

  /// The most any swaying place a light's sphere reaches leans for each metre it stands from the light, by its reach
  /// over its nearest distance: what a face's texel, which widens with the distance, is weighed against.
  fn measure_swaying(position: Vec3, sphere: Vec4, near: f32, places: &[(Vec4, f32)]) -> f32 {
    places
      .iter()
      .filter(|(place, _)| sphere.truncate().distance(place.truncate()) < sphere.w + place.w)
      .map(|(place, reach)| reach / (position.distance(place.truncate()) - place.w).max(near))
      .fold(0.0, f32::max)
  }

  /// A face's camera, built as `compute_xf_spot` builds its own: at the light, down its direction or the face's axis,
  /// the cone widened, its depth reversed.
  fn get_face_view(light: &LightDescription, basis: &LightBasis, face: usize, near: f32, far: f32) -> CameraView {
    let (direction, up, cone): (Vec3, Vec3, f32) = match light.kind {
      LightKind::Spot => (basis.direction, basis.up, light.cone),
      LightKind::Point => {
        let (direction, up) = LIGHT_SHADOW_POINT_FACES[face];

        (direction, up, LIGHT_SHADOW_POINT_CONE)
      }
    };
    let view = glam::camera::rh::view::look_at_mat4(basis.position, basis.position + direction, up);

    CameraView::new(
      basis.position,
      view,
      (cone + LIGHT_SHADOW_WIDENING).to_degrees(),
      1.0,
      near,
      far,
    )
  }
}
