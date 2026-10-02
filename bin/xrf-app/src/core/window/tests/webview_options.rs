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
  };
  let both: WebviewOptions = WebviewOptions {
    is_shader_cache_doubled: true,
    is_webgpu_developer: true,
    collection_pace: WebviewCollectionPace::Default,
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
fn a_choice_is_written_under_the_names_the_settings_read() {
  let options: WebviewOptions = WebviewOptions {
    is_shader_cache_doubled: false,
    is_webgpu_developer: true,
    collection_pace: WebviewCollectionPace::Earlier,
  };
  let written: String = serde_json::to_string(&options).unwrap();

  assert_eq!(
    written,
    r#"{"isShaderCacheDoubled":false,"isWebgpuDeveloper":true,"collectionPace":"earlier"}"#
  );
  assert_eq!(serde_json::from_str::<WebviewOptions>(&written).unwrap(), options);
}

#[test]
fn a_choice_kept_before_the_collection_pace_existed_reads_with_v8s_own() {
  let kept: WebviewOptions = serde_json::from_str(r#"{"isShaderCacheDoubled":true,"isWebgpuDeveloper":true}"#).unwrap();

  assert_eq!(kept.collection_pace, WebviewCollectionPace::Default);
  assert!(kept.is_webgpu_developer);
}
