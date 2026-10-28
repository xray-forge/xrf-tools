use std::path::PathBuf;

use xrf_dds::DdsFile;

/// The lift a cut-out's alpha takes a mip, as `shaders/common/cut_out.wgsl`'s `CUT_OUT_MIP_LIFT`.
const CUT_OUT_MIP_LIFT: f32 = 1.0;

/// The alpha references measured at: a half, and the engine's trees' `200`.
const REFERENCES: [f32; 2] = [0.5, 200.0 / 255.0];

/// How far each mip's coverage falls from its top level's, as cut-outs are minified, without the lift, with the fixed
/// lift the cuts take now, over every texture with an alpha channel in a folder of DDS files: what a computed,
/// coverage-preserving scale would still gain over the fixed lift. Ignored: it reads the installation's textures,
/// extracted into `XRF_COVERAGE_DIR` (`xrf-cli archive extract`).
#[test]
#[ignore = "reads extracted installation textures"]
fn measures_what_the_cut_out_lift_keeps_of_each_mips_coverage() {
  let Some(dir) = std::env::var_os("XRF_COVERAGE_DIR").map(PathBuf::from) else {
    eprintln!("XRF_COVERAGE_DIR is not set");
    return;
  };
  // Per mip below the top: the summed share of the top's coverage kept without the lift and with it, and the count.
  let mut kept: Vec<(f64, f64, u32)> = vec![(0.0, 0.0, 0); 12];
  let mut textures: u32 = 0;

  for entry in std::fs::read_dir(&dir).expect("the folder reads") {
    let path: PathBuf = entry.expect("an entry").path();

    if path.extension().and_then(|it| it.to_str()) != Some("dds") {
      continue;
    }

    let Ok(file) = DdsFile::read_from_bytes(&std::fs::read(&path).expect("the file reads")) else {
      continue;
    };
    let levels: u32 = file.metadata().mipmap_levels.max(1);
    let alphas: Vec<Vec<f32>> = (0..levels)
      .map_while(|level| file.decode_rgba(level).ok())
      .map(|image| {
        image
          .as_raw()
          .as_chunks::<4>()
          .0
          .iter()
          .map(|texel| f32::from(texel[3]) / 255.0)
          .collect()
      })
      .collect();

    // A texture with no cut in its alpha is no cut-out.
    if alphas.first().is_none_or(|top| top.iter().all(|alpha| *alpha > 0.98)) {
      continue;
    }

    textures += 1;

    for reference in REFERENCES {
      let cover = |alpha: &[f32], scale: f32| -> f64 {
        alpha.iter().filter(|it| **it * scale > reference).count() as f64 / alpha.len().max(1) as f64
      };
      let top: f64 = cover(&alphas[0], 1.0);

      if top < 0.01 {
        continue;
      }

      for (mip, alpha) in alphas.iter().enumerate().skip(1).take(kept.len()) {
        // Levels under four texels a side stand for nothing a blade shows.
        if alpha.len() < 16 {
          break;
        }

        let slot: &mut (f64, f64, u32) = &mut kept[mip - 1];

        slot.0 += cover(alpha, 1.0) / top;
        slot.1 += cover(alpha, 1.0 + CUT_OUT_MIP_LIFT * mip as f32) / top;
        slot.2 += 1;
      }
    }
  }

  println!("{textures} cut-out textures in {}", dir.display());
  println!("mip | samples | coverage kept, no lift | with the lift");

  for (index, (raw, lifted, count)) in kept.iter().enumerate().filter(|(_, (_, _, count))| *count > 0) {
    println!(
      "{:>3} | {count:>7} | {:>22.3} | {:>13.3}",
      index + 1,
      raw / f64::from(*count),
      lifted / f64::from(*count)
    );
  }
}
