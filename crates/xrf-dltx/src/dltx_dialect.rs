use std::collections::BTreeMap;

use xrf_error::XrfResult;
use xrf_ltx::{
  Ltx, LtxDialect, LtxDocumentSource, LtxFieldOrigin, LtxResolution, LtxResolutionDiagnostic, LtxStandardDialect,
  Section,
};

use crate::discovery::dltx_discovery::DltxDiscovery;
use crate::load::dltx_loader::DltxLoader;
use crate::resolve::dltx_resolve_result::DltxResolveResult;
use crate::resolve::dltx_resolver::DltxResolver;

/// The Monolith and Anomaly DLTX patch dialect.
///
/// Reproduces the values that engine loads, and says what it does silently. Not vanilla LTX: base data resolves
/// differently here even with no patch file present, so this is never the right choice for a vanilla or OpenXRay
/// tree.
#[derive(Debug, Default)]
pub struct DltxDialect;

impl LtxDialect for DltxDialect {
  fn get_name(&self) -> &'static str {
    "dltx"
  }

  /// The `mod_<base>_*.ltx` files, which patch another config rather than standing alone.
  ///
  /// Without this a patch file would be verified as though it were a config in its own right, and every override in
  /// it would be reported as patching a section nothing declares.
  fn plan_attachments(&self, roots: &[String], source: &dyn LtxDocumentSource) -> XrfResult<Vec<String>> {
    let mut attachments: Vec<String> = Vec::new();

    for root in roots {
      let directory: &str = Ltx::directory_of(root);
      let siblings: Vec<String> = source.list_file_names(directory)?;

      for attachment in DltxDiscovery::attachments_of(Self::file_name_of(root), &siblings) {
        attachments.push(if directory.is_empty() {
          attachment.name
        } else {
          format!("{directory}\\{}", attachment.name)
        });
      }
    }

    attachments.sort();
    attachments.dedup();

    Ok(attachments)
  }

  fn resolve(&self, root: &str, source: &dyn LtxDocumentSource) -> XrfResult<LtxResolution> {
    let resolved: DltxResolveResult = DltxResolver::new(&DltxLoader::new(source).load(root)?).resolve_all()?;

    Ok(LtxResolution {
      diagnostics: Self::to_diagnostics(&resolved),
      ltx: Self::to_ltx(&resolved, root),
      provenance: Self::to_provenance(&resolved),
    })
  }
}

impl DltxDialect {
  /// The last segment of a logical path.
  fn file_name_of(logical_path: &str) -> &str {
    match logical_path.rsplit_once('\\') {
      Some((_, name)) => name,
      None => logical_path,
    }
  }

  /// Lowers a DLTX resolution into the shared resolved shape.
  ///
  /// Sections arrive sorted by name and fields by key, which is the order the engine's own container ends up in and
  /// therefore part of matching it. Standard LTX keeps the authored order instead.
  fn to_ltx(resolved: &DltxResolveResult, root: &str) -> Ltx {
    let mut ltx: Ltx = Ltx::new();

    for (section, fields) in &resolved.sections {
      // Entered once and filled in place. Going through `set_to` per field re-looked-up the section and cloned its
      // name for every value it held.
      let target: &mut Section = ltx.entry(section.clone()).or_insert_with(Default::default);

      for (key, value) in fields {
        target.insert(key, value);
      }
    }

    ltx.set_source_paths(root);
    ltx.shrink_to_fit();

    ltx
  }

  fn to_provenance(resolved: &DltxResolveResult) -> BTreeMap<(String, String), LtxFieldOrigin> {
    let mut provenance: BTreeMap<(String, String), LtxFieldOrigin> = BTreeMap::new();

    for (section, fields) in &resolved.sections {
      for key in fields.keys() {
        if let Some(origin) = resolved.provenance.get(section, key) {
          provenance.insert(
            (section.clone(), key.clone()),
            LtxFieldOrigin {
              depth: origin.depth,
              file: origin.file.clone(),
              operation: String::from(origin.operation.as_prefix()),
            },
          );
        }
      }
    }

    provenance
  }

  /// Warnings only. Anything the engine refuses to start on already came back as an error.
  fn to_diagnostics(resolved: &DltxResolveResult) -> Vec<LtxResolutionDiagnostic> {
    resolved
      .diagnostics
      .iter()
      .map(|diagnostic| LtxResolutionDiagnostic {
        engine_behaviour: diagnostic.engine_behaviour.clone(),
        file: diagnostic.file.clone(),
        message: diagnostic.message.clone(),
        section: diagnostic.section.clone(),
      })
      .collect()
  }
}

/// Standard LTX, re-exported so a caller picking a dialect from a flag names both in one place.
pub type DltxStandardDialect = LtxStandardDialect;
