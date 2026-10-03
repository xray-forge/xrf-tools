use crate::core::window::{WebviewCollectionPace, WebviewOptions};

const WRY_DEFAULT: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection";

#[test]
fn the_webview_starts_with_frequent_collections() {
  assert_eq!(
    WebviewOptions::default().to_browser_args(),
    format!("{WRY_DEFAULT} --js-flags=--incremental-marking-soft-trigger=25")
  );
}

#[test]
fn a_collection_pace_moves_where_marking_starts_and_v8s_own_adds_nothing() {
  let paced = |collection_pace: WebviewCollectionPace| -> String {
    WebviewOptions {
      collection_pace,
      is_vsync: true,
      is_frame_rate_limited: true,
    }
    .to_browser_args()
  };

  assert_eq!(paced(WebviewCollectionPace::Default), WRY_DEFAULT);
  assert_eq!(
    paced(WebviewCollectionPace::Earlier),
    format!("{WRY_DEFAULT} --js-flags=--incremental-marking-soft-trigger=50")
  );
  assert_eq!(
    paced(WebviewCollectionPace::Frequent),
    format!("{WRY_DEFAULT} --js-flags=--incremental-marking-soft-trigger=25")
  );
}

#[test]
fn frames_past_the_display_lift_vsync_and_the_frame_rate_limit_each_on_its_own() {
  let lifted = |is_vsync: bool, is_frame_rate_limited: bool| -> String {
    WebviewOptions {
      collection_pace: WebviewCollectionPace::Default,
      is_vsync,
      is_frame_rate_limited,
    }
    .to_browser_args()
  };

  assert_eq!(lifted(true, true), WRY_DEFAULT);
  assert_eq!(lifted(false, true), format!("{WRY_DEFAULT} --disable-gpu-vsync"));
  assert_eq!(lifted(true, false), format!("{WRY_DEFAULT} --disable-frame-rate-limit"));
  assert_eq!(
    lifted(false, false),
    format!("{WRY_DEFAULT} --disable-gpu-vsync --disable-frame-rate-limit")
  );
}

#[test]
fn a_choice_is_written_under_the_names_the_settings_read() {
  let options: WebviewOptions = WebviewOptions {
    collection_pace: WebviewCollectionPace::Earlier,
    is_vsync: false,
    is_frame_rate_limited: true,
  };
  let written: String = serde_json::to_string(&options).unwrap();

  assert_eq!(
    written,
    r#"{"collectionPace":"earlier","isVsync":false,"isFrameRateLimited":true}"#
  );
  assert_eq!(serde_json::from_str::<WebviewOptions>(&written).unwrap(), options);
}

#[test]
fn a_choice_kept_with_the_retired_webgpu_options_reads_with_the_defaults_of_the_rest() {
  let kept: WebviewOptions = serde_json::from_str(r#"{"isShaderCacheDoubled":true,"isWebgpuDeveloper":true}"#).unwrap();

  assert_eq!(kept.collection_pace, WebviewCollectionPace::Frequent);
  assert!(kept.is_vsync && kept.is_frame_rate_limited);
}

#[test]
fn a_choice_of_v8s_own_pace_made_before_frequent_became_the_default_is_kept() {
  let kept: WebviewOptions =
    serde_json::from_str(r#"{"isShaderCacheDoubled":true,"isWebgpuDeveloper":false,"collectionPace":"default"}"#)
      .unwrap();

  assert_eq!(kept.collection_pace, WebviewCollectionPace::Default);
  assert_eq!(kept.collection_pace.soft_trigger(), None);
}
