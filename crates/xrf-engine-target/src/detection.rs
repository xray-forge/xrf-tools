use std::fs;
use std::path::{Path, PathBuf};

use xrf_vfs::{FsgameFile, XrayRoots};

use crate::xray_engine::XrayEngine;
use crate::xray_engine_evidence::XrayEngineEvidence;
use crate::xray_engine_resolution::XrayEngineResolution;

/// Where Atmosfear's weather graphs live, as a mount answers for them.
pub const WEATHER_GRAPHS_LOGICAL_PATH: &str = "configs\\environment\\dynamic_weather_graphs.ltx";

/// The section Anomaly's Atmosfear lists its weather states in, which no other reference tree writes.
const ATMOSFEAR_CYCLES_SECTION: &[u8] = b"[weather_cycles]";
/// The alias Anomaly's `fsgame.ltx` declares for its Warfare mode, which no other reference installation declares.
const WARFARE_PRESETS_ALIAS: &str = "$warfare_presets$";
const EXECUTABLES_DIRECTORY: &str = "bin";
const ANOMALY_LAUNCHER: &str = "anomalylauncher.exe";
const ANOMALY_EXECUTABLE_PREFIX: &str = "anomalydx";
const VERIFIED_EXECUTABLE: &str = "verifieddx11.exe";
const EXECUTABLE_EXTENSION: &str = ".exe";

/// The engine roots show they target: by installation layout, then by data, then Vanilla.
///
/// `read_config` reads one logical path through whatever the caller mounted, answering `None` where nothing resolves;
/// it is called only when no installation shows Anomaly.
pub fn detect_engine(roots: &XrayRoots, read_config: impl FnOnce(&str) -> Option<Vec<u8>>) -> XrayEngineResolution {
  let installations: Vec<PathBuf> = roots.list_installations();

  installations
    .iter()
    .find_map(|installation| find_anomaly_executable(installation))
    .map(|executable| {
      XrayEngineResolution::detected(
        XrayEngine::Extended,
        XrayEngineEvidence::AnomalyExecutables,
        executable.display().to_string(),
      )
    })
    .or_else(|| {
      installations
        .iter()
        .find(|installation| declares_warfare_presets(installation))
        .map(|installation| {
          XrayEngineResolution::detected(
            XrayEngine::Extended,
            XrayEngineEvidence::AnomalyFsgame,
            installation.join(FsgameFile::FILE_NAME).display().to_string(),
          )
        })
    })
    .or_else(|| {
      read_config(WEATHER_GRAPHS_LOGICAL_PATH)
        .filter(|bytes| has_section(bytes, ATMOSFEAR_CYCLES_SECTION))
        .map(|_| {
          XrayEngineResolution::detected(
            XrayEngine::Extended,
            XrayEngineEvidence::AtmosfearCycles,
            WEATHER_GRAPHS_LOGICAL_PATH,
          )
        })
    })
    .unwrap_or_else(XrayEngineResolution::undetected)
}

/// An executable only an Anomaly installation ships, if this one holds one.
fn find_anomaly_executable(installation: &Path) -> Option<PathBuf> {
  find_file(installation, |name| name == ANOMALY_LAUNCHER).or_else(|| {
    find_file(&installation.join(EXECUTABLES_DIRECTORY), |name| {
      name == VERIFIED_EXECUTABLE
        || (name.starts_with(ANOMALY_EXECUTABLE_PREFIX) && name.ends_with(EXECUTABLE_EXTENSION))
    })
  })
}

/// The first file directly in a directory whose lower-cased name matches, in name order.
fn find_file(directory: &Path, is_matching: impl Fn(&str) -> bool) -> Option<PathBuf> {
  let mut found: Vec<PathBuf> = fs::read_dir(directory)
    .ok()?
    .filter_map(Result::ok)
    .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
    .filter(|entry| is_matching(&entry.file_name().to_string_lossy().to_ascii_lowercase()))
    .map(|entry| entry.path())
    .collect();

  found.sort();
  found.into_iter().next()
}

/// Whether an installation's `fsgame.ltx` declares Anomaly's Warfare presets; one that cannot be read declares nothing.
fn declares_warfare_presets(installation: &Path) -> bool {
  FsgameFile::read(installation).is_ok_and(|fsgame| fsgame.find_declaration(WARFARE_PRESETS_ALIAS).is_some())
}

/// Whether an LTX file opens a section of this header on a line of its own, whatever its parents or case.
fn has_section(bytes: &[u8], header: &[u8]) -> bool {
  bytes.split(|byte| *byte == b'\n').any(|line| {
    let line: &[u8] = line.trim_ascii_start();

    line.len() >= header.len() && line[..header.len()].eq_ignore_ascii_case(header)
  })
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::{Path, PathBuf};

  use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
  use xrf_vfs::{XrayMountMode, XrayRoots};

  use super::{WEATHER_GRAPHS_LOGICAL_PATH, detect_engine};
  use crate::{XrayEngine, XrayEngineEvidence, XrayEngineResolution};

  const VANILLA_FSGAME: &str = "$fs_root$ = false| false|\n$game_data$ = false| true| $fs_root$| gamedata\\\n\
    $game_config$ = true| false| $game_data$| configs\\\n";
  const ANOMALY_FSGAME: &str = "$fs_root$ = false| false|\n$arch_dir$ = false| false| $fs_root$| db\\\n\
    $arch_dir_resource$ = false| false| $arch_dir$| mods\\\n$arch_dir_addons$ = false| true| $arch_dir$| addons\\\n\
    $game_data$ = true| true| $fs_root$| gamedata\\\n$game_config$ = true| false| $game_data$| configs\\\n\
    $warfare_presets$ = true| false| $game_config$| warfare_presets\\| *.ltx\n";
  const IXRAY_FSGAME: &str = "$fs_root$ = false| false|\n$arch_dir_resources$ = false| false| $fs_root$| resources\\\n\
    $arch_dir_addons$ = true| true| $fs_root$| ixr_addons\\\n$game_data$ = false| true| $fs_root$| gamedata\\\n";
  const ATMOSFEAR_GRAPHS: &str = "; Atmosfear\n[weather_cycles]\nclear\nstorm\n\n[cycle_clear]\nw_clear1\n";
  const COC_GRAPHS: &str = "[atmosfear_clear]\nclear = 1\n";
  const VANILLA_GRAPHS: &str = "[dynamic_default]\nclear = 0.4\n";
  const COC_MAPS: &str = "[zaton]\nweathers = atmosfear\n";

  /// Builds a throwaway tree under the generated scratch root, scoped by name since tests share it.
  fn tree(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root: PathBuf = build_absolute_generated_test_resource_path(&format!("engine_detection/{name}"));

    let _ = fs::remove_dir_all(&root);

    fs::create_dir_all(&root).expect("detection root");

    for (path, contents) in files {
      let file: PathBuf = root.join(path);

      fs::create_dir_all(file.parent().expect("a fixture file has a parent")).expect("fixture directory");
      fs::write(file, contents).expect("fixture file");
    }

    root
  }

  /// Detects as the app does: the data rule reads through the roots' own mounts.
  fn detect(roots: &XrayRoots) -> XrayEngineResolution {
    detect_engine(roots, |path| roots.open().ok()?.read_bytes(path).ok())
  }

  fn detect_root(path: &Path) -> XrayEngineResolution {
    detect(&XrayRoots::one(path.to_path_buf(), XrayMountMode::Auto))
  }

  #[test]
  fn names_an_anomaly_installation_by_its_executables() {
    let root: PathBuf = tree(
      "anomaly",
      &[
        ("fsgame.ltx", ANOMALY_FSGAME),
        ("AnomalyLauncher.exe", ""),
        ("bin/AnomalyDX11AVX.exe", ""),
        ("bin/VerifiedDX11.exe", ""),
        ("gamedata/configs/system.ltx", "[system]\n"),
      ],
    );

    let resolution: XrayEngineResolution = detect_root(&root);

    assert_eq!(resolution.engine, XrayEngine::Extended);
    assert_eq!(resolution.evidence, XrayEngineEvidence::AnomalyExecutables);
    assert_eq!(
      resolution.subject,
      Some(root.join("AnomalyLauncher.exe").display().to_string())
    );
  }

  #[test]
  fn names_an_installation_of_renderer_executables_alone() {
    let root: PathBuf = tree(
      "anomaly_bin",
      &[("fsgame.ltx", VANILLA_FSGAME), ("bin/anomalydx9.exe", "")],
    );

    let resolution: XrayEngineResolution = detect_root(&root);

    assert_eq!(resolution.evidence, XrayEngineEvidence::AnomalyExecutables);
    assert_eq!(
      resolution.subject,
      Some(root.join("bin").join("anomalydx9.exe").display().to_string())
    );
  }

  #[test]
  fn names_the_installation_a_picked_gamedata_sits_in() {
    let root: PathBuf = tree(
      "anomaly_gamedata",
      &[
        ("fsgame.ltx", ANOMALY_FSGAME),
        ("bin/VerifiedDX11.exe", ""),
        ("gamedata/configs/system.ltx", "[system]\n"),
      ],
    );

    assert_eq!(
      detect_root(&root.join("gamedata")).evidence,
      XrayEngineEvidence::AnomalyExecutables
    );
  }

  #[test]
  fn names_an_anomaly_installation_by_its_fsgame_alone() {
    let root: PathBuf = tree("anomaly_fsgame", &[("fsgame.ltx", ANOMALY_FSGAME)]);

    let resolution: XrayEngineResolution = detect_root(&root);

    assert_eq!(resolution.engine, XrayEngine::Extended);
    assert_eq!(resolution.evidence, XrayEngineEvidence::AnomalyFsgame);
  }

  #[test]
  fn names_an_atmosfear_data_tree_by_its_weather_cycles() {
    let root: PathBuf = tree(
      "atmosfear",
      &[
        ("configs/system.ltx", "[system]\n"),
        ("configs/environment/dynamic_weather_graphs.ltx", ATMOSFEAR_GRAPHS),
        ("meshes/.keep", ""),
        ("textures/.keep", ""),
      ],
    );

    let resolution: XrayEngineResolution = detect_root(&root);

    assert_eq!(resolution.engine, XrayEngine::Extended);
    assert_eq!(resolution.evidence, XrayEngineEvidence::AtmosfearCycles);
    assert_eq!(resolution.subject.as_deref(), Some(WEATHER_GRAPHS_LOGICAL_PATH));
  }

  #[test]
  fn leaves_an_openxray_installation_vanilla() {
    let root: PathBuf = tree(
      "openxray",
      &[
        ("fsgame.ltx", VANILLA_FSGAME),
        ("bin/xrEngine.exe", ""),
        (
          "gamedata/configs/environment/dynamic_weather_graphs.ltx",
          VANILLA_GRAPHS,
        ),
      ],
    );

    assert_eq!(detect_root(&root), XrayEngineResolution::undetected());
  }

  #[test]
  fn leaves_a_tree_with_sun_positions_but_no_atmosfear_vanilla() {
    // Gunslinger for OpenXRay ships `sun_positions.ltx`, which is no sign of Monolith.
    let root: PathBuf = tree(
      "gunslinger",
      &[
        ("configs/system.ltx", "[system]\n"),
        (
          "configs/environment/sun_positions.ltx",
          "[00:00:00]\nsun_altitude = -30\n",
        ),
        (
          "configs/environment/dynamic_weather_graphs.ltx",
          "[weather_periods]\ngood = 1\n",
        ),
        ("configs/game_maps_single.ltx", "[zaton]\nweathers = dynamic_zaton\n"),
      ],
    );

    assert_eq!(detect_root(&root), XrayEngineResolution::undetected());
  }

  #[test]
  fn leaves_a_call_of_chernobyl_tree_playing_atmosfear_vanilla() {
    // CoC writes `weathers = atmosfear` but no `[weather_cycles]`, and its engine reads no sun table.
    let root: PathBuf = tree(
      "coc",
      &[
        ("configs/system.ltx", "[system]\n"),
        ("configs/environment/dynamic_weather_graphs.ltx", COC_GRAPHS),
        ("configs/game_maps_single.ltx", COC_MAPS),
      ],
    );

    assert_eq!(detect_root(&root), XrayEngineResolution::undetected());
  }

  #[test]
  fn leaves_an_ixray_installation_declaring_addons_vanilla() {
    let root: PathBuf = tree("ixray", &[("fsgame.ltx", IXRAY_FSGAME)]);

    assert_eq!(detect_root(&root), XrayEngineResolution::undetected());
  }

  #[test]
  fn reads_no_config_once_the_layout_decided() {
    let root: PathBuf = tree("layout_first", &[("fsgame.ltx", ANOMALY_FSGAME)]);

    let resolution: XrayEngineResolution = detect_engine(&XrayRoots::one(root, XrayMountMode::Auto), |_| {
      panic!("the layout decided before any config is read")
    });

    assert_eq!(resolution.evidence, XrayEngineEvidence::AnomalyFsgame);
  }

  #[test]
  fn finds_a_section_whatever_its_case_parents_or_indent() {
    assert!(super::has_section(
      b"\r\n  [Weather_Cycles]:base\r\n",
      b"[weather_cycles]"
    ));
    assert!(!super::has_section(b"; [weather_cycles]\n", b"[weather_cycles]"));
    assert!(!super::has_section(b"[weather_cycles_old]\n", b"[weather_cycles]"));
  }
}
