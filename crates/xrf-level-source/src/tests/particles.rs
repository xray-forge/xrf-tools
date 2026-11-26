use std::sync::Arc;

use xrf_level::LevelPsStaticPlacement;
use xrf_ltx::Ltx;
use xrf_math::{Matrix4x4, Vector3d};
use xrf_renderer::{RenderParticlePlacement, RenderParticleSource};
use xrf_spawn::{AlifeObject, AlifeObjectInherited, ClsId};
use xrf_visual::{VisualRestPose, VisualTransform};

use crate::fixtures::{new_artefact, new_item_base, new_object};
use crate::parts::LevelParticlesPart;

#[test]
fn lifts_a_planted_system_a_centimetre() {
  let mut values: [f32; 16] = [0.0; 16];

  values[0] = 1.0;
  values[5] = 1.0;
  values[10] = 1.0;
  values[12] = 4.0;
  values[13] = 2.0;
  values[15] = 1.0;

  let placement = LevelParticlesPart::place_planted(&LevelPsStaticPlacement {
    game_types: Some(1),
    effect: String::from("zones\\zone_thermal_idle"),
    transform: Matrix4x4 { values },
  });

  assert_eq!(placement.transform[12], 4.0);
  assert_eq!(placement.transform[13], 2.0 + 0.01);
  assert_eq!(
    placement.source,
    RenderParticleSource::Static {
      name: String::from("zones\\zone_thermal_idle")
    }
  );
}

fn artefact_sections() -> Ltx {
  Ltx::read_from_str(
    r"
[af_base]
can_be_controlled = true
af_rank = 1

[af_medusa]:af_base
particles = artefact\af_gravi_idle

[af_junk]:af_base
af_rank = 0
particles = artefact\af_acidic_idle

[af_quiet]:af_base
",
  )
  .and_then(Ltx::into_inherited)
  .expect("the sections to parse and inherit")
}

/// A pose whose root bone stands `height` metres over the visual's origin, as an artefact's `link` stands at its middle.
fn root_at(height: f32) -> Arc<VisualRestPose> {
  Arc::new(VisualRestPose {
    names: vec![String::from("link")],
    transforms: vec![VisualTransform {
      i: Vector3d::new(1.0, 0.0, 0.0),
      j: Vector3d::new(0.0, 1.0, 0.0),
      k: Vector3d::new(0.0, 0.0, 1.0),
      c: Vector3d::new(0.0, height, 0.0),
    }],
  })
}

// `SwitchAfParticles` plays the section's particles at the visual's root bone, its basis upright whatever the
// artefact's turn, marked where a detector must reveal it.
#[test]
fn plays_an_artefacts_particles_upright_at_its_root_bone() {
  let ltx: Ltx = artefact_sections();
  let mut turned: AlifeObject = new_artefact("medusa", "af_medusa");

  turned.header.direction = Vector3d::new(0.0, 1.0, 0.0);

  let placement: RenderParticlePlacement =
    LevelParticlesPart::place_artefact(&turned, &ltx, &mut |_| Some(root_at(0.25)))
      .expect("the medusa to play its particles");

  assert_eq!(
    placement.source,
    RenderParticleSource::Artefact {
      idle: String::from("artefact\\af_gravi_idle"),
      is_detector_hidden: true,
    }
  );
  // At `1, 2, 3` in engine space, the root a quarter metre up, which a turn about `y` leaves where it is.
  assert_eq!(
    placement.transform,
    [
      1.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 2.25, 3.0, 1.0
    ]
  );
  assert_eq!(
    LevelParticlesPart::place_artefact(&new_artefact("junk", "af_junk"), &ltx, &mut |_| None).map(|it| it.source),
    Some(RenderParticleSource::Artefact {
      idle: String::from("artefact\\af_acidic_idle"),
      is_detector_hidden: false,
    })
  );
}

// A visual that cannot be read leaves the effect at the object's origin.
#[test]
fn plays_an_artefacts_particles_at_its_origin_without_its_visual() {
  let ltx: Ltx = artefact_sections();
  let placement: RenderParticlePlacement =
    LevelParticlesPart::place_artefact(&new_artefact("medusa", "af_medusa"), &ltx, &mut |_| None)
      .expect("the medusa to play its particles");

  assert_eq!(&placement.transform[12..15], [1.0, 2.0, 3.0]);
}

// No particles named, something holding it, or not an artefact at all: nothing plays.
#[test]
fn plays_no_particles_for_a_quiet_held_or_other_object() {
  let ltx: Ltx = artefact_sections();
  let mut held: AlifeObject = new_artefact("held", "af_medusa");

  held.header.parent_id = 7;

  let item: AlifeObject = new_object(
    "item",
    "af_medusa",
    ClsId::SFood,
    AlifeObjectInherited::CseAlifeItem(Box::new(new_item_base("dynamics\\artefacts\\af_medusa"))),
  );

  assert_eq!(
    LevelParticlesPart::place_artefact(&new_artefact("quiet", "af_quiet"), &ltx, &mut |_| None),
    None
  );
  assert_eq!(LevelParticlesPart::place_artefact(&held, &ltx, &mut |_| None), None);
  assert_eq!(LevelParticlesPart::place_artefact(&item, &ltx, &mut |_| None), None);
}
