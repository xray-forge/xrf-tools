/// How the generated modules address one another and the runtime they call.
///
/// Every specifier is the root joined with a path relative to the mirror's output directory, so one mirror can sit
/// under an application's path alias (`@/core/ipc`) or inside a package that maps its own files (`#`). The runtime
/// beside the mirror provides `invoke` (the counted wrapper commands call), `raw` (`invokeRaw`), `bulk/bulk-call`
/// (`IBulkCall`) and `signature` (`IIpcCommandSignature`, `IIpcBulkRouteSignature`).
#[derive(Clone, Debug)]
pub struct MirrorImports {
  root: &'static str,
}

impl MirrorImports {
  /// Specifiers under `root`, which names the mirror's output directory as its modules import it.
  pub const fn new(root: &'static str) -> Self {
    Self { root }
  }

  /// The counted `invoke` every generated command calls.
  pub(crate) fn invoke(&self) -> String {
    format!("{}/invoke", self.root)
  }

  /// `invokeRaw`, which raw command wrappers call.
  pub(crate) fn raw(&self) -> String {
    format!("{}/raw", self.root)
  }

  /// `IBulkCall`, which bulk route wrappers answer: the module declaring it, since a package's own `#/*` mapping
  /// resolves a file and never a directory's barrel.
  pub(crate) fn bulk(&self) -> String {
    format!("{}/bulk/bulk-call", self.root)
  }

  /// The helper types each plugin's signatures are read through.
  pub(crate) fn signature(&self) -> String {
    format!("{}/signature", self.root)
  }

  /// The types module `module`.
  pub(crate) fn type_module(&self, module: &str) -> String {
    format!("{}/types/{module}", self.root)
  }

  /// The command module `module` (`levels`, `archives-bulk`).
  pub(crate) fn command_module(&self, module: &str) -> String {
    format!("{}/commands/{module}", self.root)
  }
}

#[cfg(test)]
mod tests {
  use super::MirrorImports;

  #[test]
  fn every_specifier_starts_at_the_root() {
    let imports: MirrorImports = MirrorImports::new("#");

    assert_eq!(imports.invoke(), "#/invoke");
    assert_eq!(imports.raw(), "#/raw");
    assert_eq!(imports.bulk(), "#/bulk/bulk-call");
    assert_eq!(imports.signature(), "#/signature");
    assert_eq!(imports.type_module("xrf-app-core"), "#/types/xrf-app-core");
    assert_eq!(imports.command_module("levels-bulk"), "#/commands/levels-bulk");
  }
}
