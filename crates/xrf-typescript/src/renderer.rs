use swc_common::sync::Lrc;
use swc_common::{DUMMY_SP, SourceMap};
use swc_ecma_ast::{Module, ModuleItem};
use swc_ecma_codegen::text_writer::JsWriter;
use swc_ecma_codegen::{Config, Emitter};
use xrf_error::{XrfError, XrfResult};

/// Render one module item as canonical TypeScript, carrying neither comments nor layout.
///
/// Emitting through a synthetic single-item module is what erases the formatting: SWC keeps comments
/// outside the tree, so a parsed item holds none, and the minifying emitter writes one deterministic
/// spelling. That is what lets two differently formatted copies of a declaration compare equal, which
/// comparing source text cannot do.
pub fn render_module_item(item: &ModuleItem, source_map: &Lrc<SourceMap>) -> XrfResult<String> {
  let module: Module = Module {
    span: DUMMY_SP,
    body: vec![item.clone()],
    shebang: None,
  };
  let mut buffer: Vec<u8> = Vec::new();

  {
    let mut emitter = Emitter {
      cfg: Config::default().with_minify(true),
      cm: source_map.clone(),
      comments: None,
      wr: JsWriter::new(source_map.clone(), "\n", &mut buffer, None),
    };

    emitter
      .emit_module(&module)
      .map_err(|error| XrfError::new_invalid_error(format!("Failed to render a TypeScript declaration: {error}")))?;
  }

  String::from_utf8(buffer)
    .map_err(|error| XrfError::new_invalid_error(format!("Rendered TypeScript is not valid UTF-8: {error}")))
}
