use std::collections::HashMap;

use xrf_ltx::Ltx;
use xrf_renderer::{RenderLevelSpawn, RenderSpawnGroup, RenderSpawnObject};
use xrf_spawn::{AlifeObject, AlifeObjectInherited, AlifeObjectItemArtefact};
use xrf_visual::VisualTransform;

use crate::parts::LevelSpawn;
use crate::spawn::LevelSpawnCategory;

impl LevelSpawn {
  /// The objects of the spawn the renderer draws, in the spawn's order, each visual named once in the order first
  /// stood, and each artefact `sections` says lies hidden until a detector reveals it marked so; none is marked without
  /// them.
  pub(crate) fn get_render(&self, sections: Option<&Ltx>) -> RenderLevelSpawn {
    let mut visuals: Vec<String> = Vec::new();
    let mut indices: HashMap<&str, u32> = HashMap::new();
    let mut objects: Vec<RenderSpawnObject> = Vec::new();

    for (index, object) in self.objects.iter().enumerate() {
      let Some((category, name)) = LevelSpawnCategory::get_drawn_visual(object) else {
        continue;
      };
      let visual: u32 = *indices.entry(name).or_insert_with(|| {
        visuals.push(name.to_owned());

        (visuals.len() - 1) as u32
      });

      objects.push(RenderSpawnObject {
        index: index as u32,
        group: RenderSpawnGroup {
          category: category.into(),
          is_released: self.get_release(index).is_some(),
          is_detector_hidden: Self::is_detector_hidden(object, sections),
        },
        visual,
        transform: VisualTransform::of_spawn(&object.header.position, &object.header.direction).to_matrix(),
      });
    }

    RenderLevelSpawn {
      objects,
      visuals,
      detail: 0.0,
    }
  }

  /// Whether `object` is an artefact its section, read from `sections`, hides until a detector reveals it.
  fn is_detector_hidden(object: &AlifeObject, sections: Option<&Ltx>) -> bool {
    matches!(object.inherited, AlifeObjectInherited::CseAlifeItemArtefact(_))
      && sections
        .and_then(|sections| sections.section(&object.header.section))
        .is_some_and(AlifeObjectItemArtefact::is_detector_hidden)
  }
}
