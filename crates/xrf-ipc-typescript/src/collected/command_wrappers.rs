use std::borrow::Cow;
use std::sync::{Arc, Mutex};

use specta::Types;
use specta::datatype::{DataType, Field, Function, NamedReference, NamedReferenceType, Reference, Struct};
use specta_typescript::{Error, Exporter, FrameworkExporter, define};
use tauri_specta::{BuilderConfiguration, ErrorHandlingMode};
use xrf_utils::to_camel_case;

use crate::collected::{CommandTypescript, PluginCommands, SpectaCommand};
use crate::typescript::format::TypeScriptFormat;
use crate::write::{GeneratedOutput, PluginSymbols};

/// Renders one plugin's command wrappers from the builder's own function data, through Specta's exporter, so no text
/// Specta writes is read back.
pub(crate) struct CommandWrappers;

impl CommandWrappers {
  /// The name Tauri gives its `Channel` among the collected types.
  const CHANNEL: &'static str = "TAURI_CHANNEL";

  /// The name and module of the `Result` a command answers, whose success alone a throwing wrapper resolves to.
  const RESULT: &'static str = "Result";
  const RESULT_MODULES: [&'static str; 2] = ["std::result", "core::result"];

  /// The commands `config` declares, with their wrappers rendered.
  ///
  /// # Panics
  ///
  /// Panics on a builder configured otherwise than the wrappers are rendered for.
  pub fn render(config: &BuilderConfiguration) -> Result<PluginCommands, Error> {
    Self::assert_supported(config);

    let plugin: &str = config.plugin_name.unwrap_or_default();
    let functions: Arc<Vec<Function>> = Arc::new(config.commands.clone());
    let command_enum: String = PluginSymbols::new(plugin).command_enum();
    let rendered: Arc<Mutex<Option<(String, bool)>>> = Arc::new(Mutex::new(None));
    let exporter: Exporter = Exporter::from(CommandTypescript::configured()).framework_runtime({
      let functions: Arc<Vec<Function>> = Arc::clone(&functions);
      let rendered: Arc<Mutex<Option<(String, bool)>>> = Arc::clone(&rendered);

      move |exporter: FrameworkExporter| {
        let wrappers: (String, bool) = Self::render_object(&functions, &command_enum, &exporter)?;

        *rendered.lock().expect("Rendered wrappers lock is poisoned") = Some(wrappers);

        Ok(Cow::Borrowed(""))
      }
    });

    // Only the runtime callback's rendering is kept; the types the exporter writes beside it are written elsewhere.
    exporter.export(&config.types, TypeScriptFormat::default())?;

    let (wrappers, uses_channel): (String, bool) = rendered
      .lock()
      .expect("Rendered wrappers lock is poisoned")
      .take()
      .unwrap_or_else(|| panic!("Specta rendered no wrappers for the `{plugin}` plugin"));

    Ok(PluginCommands {
      commands: functions.iter().map(SpectaCommand::from).collect(),
      wrappers: GeneratedOutput::normalize_doc_indentation(&wrappers),
      uses_channel,
    })
  }

  /// The wrappers object, one field per command under its docs, and whether any wrapper carries a `Channel`.
  fn render_object(
    functions: &[Function],
    command_enum: &str,
    exporter: &FrameworkExporter,
  ) -> Result<(String, bool), Error> {
    let mut object = Struct::named();
    let mut uses_channel: bool = false;

    for function in functions {
      let mut parameters: Vec<String> = Vec::with_capacity(function.args().len());
      let mut names: Vec<String> = Vec::with_capacity(function.args().len());

      for (name, data_type) in function.args() {
        let name: String = to_camel_case(name);

        uses_channel |= Self::channel_of(data_type, exporter.types).is_some();
        parameters.push(format!("{name}: {}", Self::render_type(data_type, exporter)?));
        names.push(name);
      }

      let returns: String = match function.result() {
        Some(result) => {
          let resolved: &DataType = Self::success_of(result, exporter.types);

          uses_channel |= Self::channel_of(resolved, exporter.types).is_some();
          Self::render_type(resolved, exporter)?
        }
        None => String::from("void"),
      };
      let payload: String = if names.is_empty() {
        String::new()
      } else {
        format!(", {{ {} }}", names.join(", "))
      };
      let mut field: Field = Field::new(
        define(format!(
          "({}) => __TAURI_INVOKE<{returns}>({command_enum}.{}{payload})",
          parameters.join(", "),
          PluginSymbols::member(function.name())
        ))
        .into(),
      );

      field.docs = function.docs.clone();
      field.deprecated = function.deprecated.clone();
      object = object.field(PluginSymbols::wrapper(function.name()), field);
    }

    Ok((exporter.inline(&object.build())?, uses_channel))
  }

  /// A type as a wrapper spells it: Tauri's channel as `Channel<Payload>`, anything else as Specta references or
  /// inlines it.
  fn render_type(data_type: &DataType, exporter: &FrameworkExporter) -> Result<String, Error> {
    if let Some(channel) = Self::channel_of(data_type, exporter.types) {
      let payload: String = match &channel.inner {
        NamedReferenceType::Reference { generics, .. } => match generics.first() {
          Some((_, DataType::Reference(reference))) => exporter.reference(reference)?,
          Some((_, payload)) => exporter.inline(payload)?,
          None => String::from("never"),
        },
        NamedReferenceType::Inline { .. } | NamedReferenceType::Recursive(_) => String::from("never"),
      };

      return Ok(format!("Channel<{payload}>"));
    }

    match data_type {
      DataType::Reference(reference) => exporter.reference(reference),
      other => exporter.inline(other),
    }
  }

  /// The reference `data_type` makes to Tauri's channel, where it is one.
  fn channel_of<'a>(data_type: &'a DataType, types: &Types) -> Option<&'a NamedReference> {
    let DataType::Reference(Reference::Named(reference)) = data_type else {
      return None;
    };
    let named = types.get(reference)?;

    (named.name == Self::CHANNEL && named.module_path.starts_with("tauri::")).then_some(reference)
  }

  /// What a command resolves to when it answers a `Result`, which a throwing wrapper rejects on error; the answer
  /// itself otherwise.
  fn success_of<'a>(data_type: &'a DataType, types: &Types) -> &'a DataType {
    if let DataType::Reference(Reference::Named(reference)) = data_type
      && let Some(named) = types.get(reference)
      && named.name == Self::RESULT
      && Self::RESULT_MODULES.contains(&named.module_path.as_ref())
      && let NamedReferenceType::Reference { generics, .. } = &reference.inner
      && let [(_, success), _, ..] = generics.as_slice()
    {
      return success;
    }

    data_type
  }

  /// Fails on a builder whose wrappers would need what is not rendered here: typed errors, serde phases, wide
  /// integers, events, semantic types or constants.
  fn assert_supported(config: &BuilderConfiguration) {
    let plugin: &str = config.plugin_name.unwrap_or_default();

    assert!(
      config.error_handling == ErrorHandlingMode::Throw
        && config.disable_serde_phases
        && config.dangerously_cast_bigints_to_number
        && config.semantic_types.is_none()
        && config.events.is_empty()
        && config.constants.is_empty(),
      "The `{plugin}` builder is configured otherwise than its wrappers are rendered for: it must throw on error, \
       disable serde phases, cast big integers to numbers, and declare no events, semantic types or constants"
    );
  }
}
