use crate::core::window::WebviewOptions;

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
  };
  let both: WebviewOptions = WebviewOptions {
    is_shader_cache_doubled: true,
    is_webgpu_developer: true,
  };

  assert_eq!(none.to_browser_args(), WRY_DEFAULT);
  assert_eq!(
    both.to_browser_args(),
    format!("{WRY_DEFAULT} --enable-features=AggressiveShaderCacheLimits --enable-webgpu-developer-features")
  );
}

#[test]
fn a_choice_is_written_under_the_names_the_settings_read() {
  let options: WebviewOptions = WebviewOptions {
    is_shader_cache_doubled: false,
    is_webgpu_developer: true,
  };
  let written: String = serde_json::to_string(&options).unwrap();

  assert_eq!(written, r#"{"isShaderCacheDoubled":false,"isWebgpuDeveloper":true}"#);
  assert_eq!(serde_json::from_str::<WebviewOptions>(&written).unwrap(), options);
}
