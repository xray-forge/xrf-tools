//! A compile's AI step on a level directory: what it reads, writes, checks and refuses.

use std::path::PathBuf;

use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_geometry::BvhTracer;
use xrf_job::JobHandle;
use xrf_level::{BuildAimapFile, BuildCformFile, LevelAiFile};
use xrf_report::Status;

use crate::tests::ai::aimap;
use crate::tests::detail_step::Fixture;
use crate::{
  AiMap, AiMapCompilation, AiMapCompiler, AiMapCovers, AiMapGuid, AiMapRequest, AiRayModel, LevelCompileReport,
  LevelCompileRequest, LevelCompileStep, LevelCompileStepRun, LevelCompiler, RayBackend, ShadowMaterials,
};

/// The details step's level, with the AI tests' four-node map as `build.aimap`.
fn level(name: &str, aimap: &BuildAimapFile) -> XrfResult<Fixture> {
  let fixture: Fixture = Fixture::new(file!(), name)?;

  Fixture::write(&fixture.level.join(BuildAimapFile::FILE_NAME), |writer| {
    aimap.write::<XRayByteOrder>(writer)
  })?;

  Ok(fixture)
}

fn request(fixture: &Fixture, covers: AiMapCovers, is_forced: bool) -> LevelCompileRequest {
  LevelCompileRequest {
    ai_covers: covers,
    ..fixture.request_steps(&[LevelCompileStep::Ai], is_forced)
  }
}

/// The run the compile reports for its one step.
fn run(report: &LevelCompileReport) -> &LevelCompileStepRun {
  &report.steps[0].run
}

#[test]
fn writes_the_map_the_compiler_compiles_and_checks_it_as_a_graph() -> XrfResult {
  let fixture: Fixture = level("full", &aimap())?;
  let report: LevelCompileReport =
    LevelCompiler::compile(&request(&fixture, AiMapCovers::Full, false), &JobHandle::inert())?;
  let cform: BuildCformFile =
    BuildCformFile::read_from_path::<XRayByteOrder, _>(&fixture.level.join(BuildCformFile::FILE_NAME))?;
  let tracer: BvhTracer = cform.build_tracer();
  let shadows: ShadowMaterials = ShadowMaterials::new_opaque(cform.raycast_faces.len());
  let expected: AiMapCompilation = AiMapCompiler::compile(
    AiMapRequest {
      aimap: &aimap(),
      covers: AiMapCovers::Full,
      rays: Some(AiRayModel {
        tracer: &tracer,
        shadows: &shadows,
      }),
      backend: RayBackend::Cpu,
      dxc: None,
      gpu: None,
      replaced: None,
    },
    &JobHandle::inert(),
  )?;
  let written: AiMap = AiMap::read_from_path(&fixture.level.join(LevelAiFile::FILE_NAME))?;

  assert_eq!(written.nodes, expected.map.nodes);
  assert_eq!(report.work, None, "no SDK compiler ran");

  let LevelCompileStepRun::AiMapCompiler {
    covers,
    gpu,
    guid,
    census,
    graph,
  } = run(&report)
  else {
    panic!("the AI map compiler ran: {report:?}");
  };

  assert_eq!((*covers, gpu, *guid), (AiMapCovers::Full, &None, AiMapGuid::Derived));
  assert_eq!((census.nodes, census.components), (4, 1));
  assert_eq!(graph.status(), Status::Passed);

  Ok(())
}

#[test]
fn refuses_to_overwrite_the_map_unless_forced_and_keeps_its_identity() -> XrfResult {
  let fixture: Fixture = level("forced", &aimap())?;
  let output: PathBuf = fixture.level.join(LevelAiFile::FILE_NAME);

  LevelCompiler::compile(&request(&fixture, AiMapCovers::None, false), &JobHandle::inert())?;

  let first: AiMap = AiMap::read_from_path(&output)?;
  let error: String = LevelCompiler::compile(&request(&fixture, AiMapCovers::None, false), &JobHandle::inert())
    .unwrap_err()
    .to_string();

  assert!(error.contains("force the compile"), "{error}");

  let report: LevelCompileReport =
    LevelCompiler::compile(&request(&fixture, AiMapCovers::Full, true), &JobHandle::inert())?;

  assert!(matches!(
    run(&report),
    LevelCompileStepRun::AiMapCompiler {
      guid: AiMapGuid::Kept,
      ..
    }
  ));
  assert_eq!(AiMap::read_from_path(&output)?.file.header.guid, first.file.header.guid);

  Ok(())
}

#[test]
fn compiles_with_no_covers_from_the_nodes_alone() -> XrfResult {
  let fixture: Fixture = level("draft", &aimap())?;

  for input in [BuildCformFile::FILE_NAME, "build.prj"] {
    std::fs::remove_file(fixture.level.join(input))?;
  }

  let report: LevelCompileReport = LevelCompiler::compile(
    &LevelCompileRequest {
      gamedata: fixture.level.join("no game data"),
      ..request(&fixture, AiMapCovers::None, false)
    },
    &JobHandle::inert(),
  )?;

  assert!(matches!(
    run(&report),
    LevelCompileStepRun::AiMapCompiler {
      covers: AiMapCovers::None,
      gpu: None,
      ..
    }
  ));

  Ok(())
}

#[test]
fn refuses_a_map_linking_a_node_to_itself_leaving_none_written() -> XrfResult {
  let mut looped: BuildAimapFile = aimap();

  looped.nodes[0].links[0] = 0;

  let fixture: Fixture = level("self_link", &looped)?;
  let error: String = LevelCompiler::compile(&request(&fixture, AiMapCovers::None, false), &JobHandle::inert())
    .unwrap_err()
    .to_string();

  assert!(error.contains("graph check"), "{error}");
  assert!(!fixture.level.join(LevelAiFile::FILE_NAME).exists());

  Ok(())
}
