use crate::core::window::{WebviewCollectionPace, WebviewOptions};

const WRY_DEFAULT: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection";

#[test]
fn the_webview_starts_with_the_doubled_pipeline_cache_and_without_developer_features() {
  assert_eq!(
    WebviewOptions::default().to_browser_args(),
    format!("{WRY_DEFAULT} --enable-features=AggressiveShaderCacheLimits")
  );
}

#[test]
fn every_choice_keeps_the_default_it_replaces() {
  let none: WebviewOptions = WebviewOptions {
    is_shader_cache_doubled: false,
    is_webgpu_developer: false,
    collection_pace: WebviewCollectionPace::Default,
    is_vsync_disabled: false,
    is_frame_rate_unlimited: false,
  };
  let both: WebviewOptions = WebviewOptions {
    is_shader_cache_doubled: true,
    is_webgpu_developer: true,
    collection_pace: WebviewCollectionPace::Default,
    is_vsync_disabled: false,
    is_frame_rate_unlimited: false,
  };

  assert_eq!(none.to_browser_args(), WRY_DEFAULT);
  assert_eq!(
    both.to_browser_args(),
    format!("{WRY_DEFAULT} --enable-features=AggressiveShaderCacheLimits --enable-webgpu-developer-features")
  );
}

#[test]
fn a_collection_pace_moves_where_marking_starts_and_v8s_own_adds_nothing() {
  let paced = |collection_pace: WebviewCollectionPace| -> String {
    WebviewOptions {
      is_shader_cache_doubled: false,
      is_webgpu_developer: false,
      collection_pace,
      is_vsync_disabled: false,
      is_frame_rate_unlimited: false,
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
  let lifted = |is_vsync_disabled: bool, is_frame_rate_unlimited: bool| -> String {
    WebviewOptions {
      is_shader_cache_doubled: false,
      is_webgpu_developer: false,
      collection_pace: WebviewCollectionPace::Default,
      is_vsync_disabled,
      is_frame_rate_unlimited,
    }
    .to_browser_args()
  };

  assert_eq!(lifted(true, false), format!("{WRY_DEFAULT} --disable-gpu-vsync"));
  assert_eq!(lifted(false, true), format!("{WRY_DEFAULT} --disable-frame-rate-limit"));
  assert_eq!(
    lifted(true, true),
    format!("{WRY_DEFAULT} --disable-gpu-vsync --disable-frame-rate-limit")
  );
}

#[test]
fn a_choice_is_written_under_the_names_the_settings_read() {
  let options: WebviewOptions = WebviewOptions {
    is_shader_cache_doubled: false,
    is_webgpu_developer: true,
    collection_pace: WebviewCollectionPace::Earlier,
    is_vsync_disabled: true,
    is_frame_rate_unlimited: false,
  };
  let written: String = serde_json::to_string(&options).unwrap();

  assert_eq!(
    written,
    r#"{"isShaderCacheDoubled":false,"isWebgpuDeveloper":true,"collectionPace":"earlier","isVsyncDisabled":true,"isFrameRateUnlimited":false}"#
  );
  assert_eq!(serde_json::from_str::<WebviewOptions>(&written).unwrap(), options);
}

#[test]
fn a_choice_kept_before_the_later_options_existed_reads_with_their_defaults() {
  let kept: WebviewOptions = serde_json::from_str(r#"{"isShaderCacheDoubled":true,"isWebgpuDeveloper":true}"#).unwrap();

  assert_eq!(kept.collection_pace, WebviewCollectionPace::Default);
  assert!(!kept.is_vsync_disabled && !kept.is_frame_rate_unlimited);
  assert!(kept.is_webgpu_developer);
}
