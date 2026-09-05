use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use serde_json::json;
use tauri::ipc::{Channel, InvokeResponseBody};
use uuid::Uuid;
use xrf_db::{SpawnFile, XRayByteOrder};
use xrf_job::{ExecutionRequest, JobHandle, JobOutcome};
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use super::{SpawnConversion, convert, register_conversion, run_conversion};
use crate::core::execution::ExecutionState;
use crate::core::jobs::JobRegistry;
use crate::plugins::spawn::request::SpawnConversionRequest;

fn request(case: &str) -> SpawnConversionRequest {
  let directory: PathBuf = build_absolute_generated_test_resource_path(&format!("spawn/conversion/{case}"));

  fs::create_dir_all(&directory).expect("scratch directory");

  SpawnConversionRequest {
    source: directory.join("source.spawn"),
    destination: directory.join("output"),
  }
}

#[test]
fn pack_and_unpack_share_one_lease_until_work_finishes() {
  let registry: Arc<JobRegistry> = Arc::new(JobRegistry::new());
  let request: SpawnConversionRequest = request("lease");

  for first in [SpawnConversion::Pack, SpawnConversion::Unpack] {
    let (job, registration) = register_conversion(&registry, &request, first, Uuid::new_v4(), Channel::new(|_| Ok(())))
      .expect("first conversion registers");

    job.cancel();

    for second in [SpawnConversion::Pack, SpawnConversion::Unpack] {
      assert!(register_conversion(&registry, &request, second, Uuid::new_v4(), Channel::new(|_| Ok(()))).is_err());
    }

    drop(registration);
  }
}

#[test]
fn cancellation_before_reading_preserves_existing_output() {
  let request: SpawnConversionRequest = request("cancelled");

  fs::write(&request.destination, b"existing output").expect("existing destination");

  let job: JobHandle = JobHandle::inert();

  job.cancel();

  for operation in [SpawnConversion::Pack, SpawnConversion::Unpack] {
    assert!(
      convert(&request, operation, &job)
        .expect("cancelled before missing source is read")
        .outcome
        == JobOutcome::Cancelled
    );
  }

  assert_eq!(fs::read(&request.destination).unwrap(), b"existing output");
}

#[test]
fn failure_is_retained_and_releases_the_conversion_lease() {
  let registry: Arc<JobRegistry> = Arc::new(JobRegistry::new());
  let execution: ExecutionState = ExecutionState::new(ExecutionRequest::Auto).expect("execution pool");
  let request: SpawnConversionRequest = request("failure");

  for operation in [SpawnConversion::Pack, SpawnConversion::Unpack] {
    let id: Uuid = Uuid::new_v4();
    let outcome = tauri::async_runtime::block_on(run_conversion(
      &execution,
      &registry,
      request.clone(),
      operation,
      id,
      Channel::new(|_| Ok(())),
    ));

    assert!(outcome.is_err());

    let listed = registry
      .list()
      .into_iter()
      .find(|job| job.id == id)
      .expect("retained job");

    assert_eq!(serde_json::to_value(listed.conclusion).unwrap(), json!("failed"));
    assert!(listed.error.is_some());
    assert_eq!(listed.request, Some(serde_json::to_value(&request).unwrap()));
  }
}

fn synthetic_spawn() -> SpawnFile {
  serde_json::from_value(json!({
    "header": { "version": 10, "guid": Uuid::nil(), "graphGuid": Uuid::nil(), "objectsCount": 0, "levelsCount": 0 },
    "alifeSpawn": { "objects": [] }, "artefactSpawn": { "nodes": [] }, "patrols": { "patrols": [] },
    "graphs": {
      "header": { "version": 8, "verticesCount": 0, "edgesCount": 0, "pointsCount": 0, "guid": Uuid::nil(), "levelsCount": 0 },
      "levels": [], "vertices": [], "edges": [], "points": [],
      "crossTables": [{ "version": 8, "nodesCount": 0, "verticesCount": 0, "levelGuid": Uuid::nil(), "gameGuid": Uuid::nil() }]
    }
  })).expect("synthetic spawn with an empty cross-table")
}

#[test]
fn conversions_round_trip_and_retain_the_output() {
  let request: SpawnConversionRequest = request("round_trip");

  synthetic_spawn()
    .write_to_path::<XRayByteOrder, _>(&request.source)
    .expect("write synthetic spawn");

  let registry: Arc<JobRegistry> = Arc::new(JobRegistry::new());
  let execution: ExecutionState = ExecutionState::new(ExecutionRequest::Auto).expect("execution pool");
  let unpacked = tauri::async_runtime::block_on(run_conversion(
    &execution,
    &registry,
    request.clone(),
    SpawnConversion::Unpack,
    Uuid::new_v4(),
    Channel::new(|_| Ok(())),
  ))
  .expect("unpack job");
  let packed = tauri::async_runtime::block_on(run_conversion(
    &execution,
    &registry,
    SpawnConversionRequest {
      source: unpacked.destination,
      destination: request.source.with_extension("repacked"),
    },
    SpawnConversion::Pack,
    Uuid::new_v4(),
    Channel::new(|_| Ok(())),
  ))
  .expect("pack job");

  assert_eq!(packed.outcome, JobOutcome::Completed);
  assert_eq!(
    fs::read(&request.source).unwrap(),
    fs::read(&packed.destination).unwrap()
  );
  assert!(
    registry
      .list()
      .iter()
      .all(|job| serde_json::to_value(job.conclusion).unwrap() == json!("completed"))
  );
  assert!(registry.list().iter().all(|job| job.result.is_some()));
}

#[test]
fn cancellation_during_reading_stops_before_output_but_writing_finishes() {
  let execution: ExecutionState = ExecutionState::new(ExecutionRequest::Auto).expect("execution pool");

  for phase in ["read", "write"] {
    for operation in [SpawnConversion::Pack, SpawnConversion::Unpack] {
      let mut request: SpawnConversionRequest = request(&format!("cancel_{phase}_{operation:?}"));

      match operation {
        SpawnConversion::Pack => {
          request.source.set_extension("chunks");
          synthetic_spawn()
            .export_to_path::<XRayByteOrder, _>(&request.source)
            .expect("unpacked source");
        }
        SpawnConversion::Unpack => synthetic_spawn()
          .write_to_path::<XRayByteOrder, _>(&request.source)
          .expect("packed source"),
      }

      let registry: Arc<JobRegistry> = Arc::new(JobRegistry::new());
      let cancelling = Arc::downgrade(&registry);
      let id: Uuid = Uuid::new_v4();
      let progress = Channel::new(move |body| {
        if let InvokeResponseBody::Json(body) = body {
          let value: serde_json::Value = serde_json::from_str(&body).expect("progress");

          if value["levels"][0]["id"] == phase {
            cancelling.upgrade().expect("live registry").cancel(id);
          }
        }

        Ok(())
      });
      let result = tauri::async_runtime::block_on(run_conversion(
        &execution,
        &registry,
        request.clone(),
        operation,
        id,
        progress,
      ))
      .expect("conversion result");

      assert_eq!(result.outcome == JobOutcome::Cancelled, phase == "read");
      assert_eq!(request.destination.exists(), phase == "write");
      assert_eq!(
        serde_json::to_value(registry.list()[0].conclusion).unwrap(),
        json!(if phase == "read" { "cancelled" } else { "completed" })
      );
    }
  }
}
