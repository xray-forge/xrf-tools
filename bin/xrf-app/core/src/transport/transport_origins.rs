use tauri::Config;

/// The page origins allowed to fetch from the transport: the application's own, and no other.
#[derive(Clone, Debug)]
pub(crate) struct TransportOrigins {
  origins: Vec<String>,
}

impl TransportOrigins {
  /// Origins a bundled build serves its page from: WebView2's virtual host over either scheme, and the custom scheme
  /// every other platform uses.
  pub(crate) const BUNDLED: [&'static str; 3] =
    ["http://tauri.localhost", "https://tauri.localhost", "tauri://localhost"];

  pub(crate) fn new(origins: Vec<String>) -> Self {
    Self { origins }
  }

  /// The bundled origins, and the dev server's where this build serves its page from one.
  pub(crate) fn of_config(config: &Config) -> Self {
    let development: Option<String> = config
      .build
      .dev_url
      .as_ref()
      .filter(|_| tauri::is_dev())
      .map(|url| url.origin().ascii_serialization());

    Self::new(
      development
        .into_iter()
        .chain(Self::BUNDLED.iter().map(|origin| (*origin).to_string()))
        .collect(),
    )
  }

  pub(crate) fn allows(&self, origin: &str) -> bool {
    self.origins.iter().any(|allowed| allowed == origin)
  }

  pub(crate) fn list(&self) -> &[String] {
    &self.origins
  }
}
