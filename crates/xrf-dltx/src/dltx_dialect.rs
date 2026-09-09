use std::collections::BTreeMap;
use std::sync::Arc;

use xrf_error::XrfResult;
use xrf_ltx::{
  Ltx, LtxDialect, LtxDocumentSource, LtxFieldOrigin, LtxProvenance, LtxResolution, LtxResolutionDiagnostic,
  LtxResolveRequest, LtxStandardDialect, Section,
};

use crate::discovery::dltx_discovery::DltxDiscovery;
use crate::load::dltx_load_result::DltxLoadResult;
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

  fn resolve(
    &self,
    root: &str,
    source: &dyn LtxDocumentSource,
    request: LtxResolveRequest,
  ) -> XrfResult<LtxResolution> {
    // The load result outlives the resolve by one call now, because it is what knows which file declared each
    // section. It is still dropped here rather than retained.
    let loaded: DltxLoadResult = DltxLoader::new(source).load(root)?;
    let resolved: DltxResolveResult = DltxResolver::new(&loaded).resolve_all()?;

    Ok(LtxResolution {
      diagnostics: Self::to_diagnostics(&resolved),
      ltx: Self::to_ltx(&resolved, &loaded, root),
      // The resolver tracks the winning statement either way - it has to, to pick one - but folding that into a map
      // over every field of the tree is the caller's cost to ask for. An Anomaly sweep resolves thousands of roots
      // and reads none of it.
      provenance: if request.is_with_provenance() {
        Self::to_provenance(&resolved)
      } else {
        LtxProvenance::default()
      },
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
  fn to_ltx(resolved: &DltxResolveResult, loaded: &DltxLoadResult, root: &str) -> Ltx {
    let mut ltx: Ltx = Ltx::new();
    // One handle per declaring file rather than one per section: a config commonly declares hundreds.
    let mut origins: BTreeMap<&str, Arc<str>> = BTreeMap::new();

    for (section, fields) in &resolved.sections {
      // Entered once and filled in place. Going through `set_to` per field re-looked-up the section and cloned its
      // name for every value it held.
      let target: &mut Section = ltx.entry(section.clone()).or_insert_with(Default::default);

      for (key, value) in fields {
        target.insert(key, value);
      }

      // A section an override created without a base declaration has no declaring file, and says so rather than
      // naming the root that resolved it.
      if let Some(declared_in) = loaded.section_files.get(section) {
        target.set_origin(Arc::clone(
          origins
            .entry(declared_in.as_str())
            .or_insert_with(|| Arc::from(declared_in.as_str())),
        ));
      }
    }

    ltx.set_source_paths(root);
    ltx.shrink_to_fit();

    ltx
  }

  /// Which statement won each resolved field.
  ///
  /// Every field gets one, not only a patched one: the point of the record is that a value can be accounted for, and
  /// "the base file wrote it and nothing contested it" is as much an answer as naming a `mod_*.ltx`. Winning files are
  /// shared by name, because one file commonly wins hundreds of fields.
  fn to_provenance(resolved: &DltxResolveResult) -> LtxProvenance {
    let mut provenance: LtxProvenance = LtxProvenance::default();
    let mut files: BTreeMap<&str, Arc<str>> = BTreeMap::new();

    for (section, fields) in &resolved.sections {
      let name: Arc<str> = Arc::from(section.as_str());

      for key in fields.keys() {
        if let Some(origin) = resolved.provenance.get(section, key) {
          provenance.insert(
            Arc::clone(&name),
            Arc::from(key.as_str()),
            LtxFieldOrigin::Loaded {
              depth: origin.depth,
              file: Arc::clone(
                files
                  .entry(origin.file.as_str())
                  .or_insert_with(|| Arc::from(origin.file.as_str())),
              ),
              operation: Box::from(origin.operation.as_prefix()),
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
