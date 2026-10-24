use xrf_utils::to_camel_case;

/// The names a plugin's generated modules declare, spelled in one place so a module and the merged signatures that
/// import it cannot disagree.
///
/// For the `sprite-equipment` plugin: `spriteEquipmentCommands`, `ESpriteEquipmentCommand`,
/// `ISpriteEquipmentCommandSignatures`, and the `Raw` and `Bulk` forms of each.
pub(crate) struct PluginSymbols {
  /// The plugin name in camel case (`spriteEquipment`).
  camel: String,
  /// The plugin name in Pascal case (`SpriteEquipment`).
  pascal: String,
}

impl PluginSymbols {
  pub(crate) fn new(plugin: &str) -> Self {
    let camel: String = to_camel_case(plugin);
    let mut characters = camel.chars();
    let pascal: String = characters
      .next()
      .map(|first| first.to_uppercase().chain(characters).collect())
      .unwrap_or_default();

    Self { camel, pascal }
  }

  /// The object of typed command wrappers.
  pub(crate) fn commands(&self) -> String {
    format!("{}Commands", self.camel)
  }

  /// The enum of typed commands' wire names.
  pub(crate) fn command_enum(&self) -> String {
    format!("E{}Command", self.pascal)
  }

  /// The interface of typed commands' signatures.
  pub(crate) fn command_signatures(&self) -> String {
    format!("I{}CommandSignatures", self.pascal)
  }

  /// The object of raw command wrappers.
  pub(crate) fn raw_commands(&self) -> String {
    format!("{}RawCommands", self.camel)
  }

  /// The enum of raw commands' wire names.
  pub(crate) fn raw_command_enum(&self) -> String {
    format!("E{}RawCommand", self.pascal)
  }

  /// The interface of raw commands' signatures.
  pub(crate) fn raw_command_signatures(&self) -> String {
    format!("I{}RawCommandSignatures", self.pascal)
  }

  /// The object of bulk route wrappers.
  pub(crate) fn bulk_routes(&self) -> String {
    format!("{}BulkRoutes", self.camel)
  }

  /// The enum of bulk routes.
  pub(crate) fn bulk_route_enum(&self) -> String {
    format!("E{}BulkRoute", self.pascal)
  }

  /// The interface of bulk routes' signatures.
  pub(crate) fn bulk_route_signatures(&self) -> String {
    format!("I{}BulkRouteSignatures", self.pascal)
  }

  /// An enum member naming the command or route `name` (`open_level` is `OPEN_LEVEL`).
  pub(crate) fn member(name: &str) -> String {
    name.to_uppercase()
  }

  /// A wrapper's key for the command or route `name` (`open_level` is `openLevel`).
  pub(crate) fn wrapper(name: &str) -> String {
    to_camel_case(name)
  }
}

#[cfg(test)]
mod tests {
  use super::PluginSymbols;

  #[test]
  fn a_hyphenated_plugin_names_its_symbols_in_both_cases() {
    let symbols: PluginSymbols = PluginSymbols::new("sprite-equipment");

    assert_eq!(symbols.commands(), "spriteEquipmentCommands");
    assert_eq!(symbols.command_enum(), "ESpriteEquipmentCommand");
    assert_eq!(symbols.command_signatures(), "ISpriteEquipmentCommandSignatures");
    assert_eq!(symbols.raw_commands(), "spriteEquipmentRawCommands");
    assert_eq!(symbols.raw_command_enum(), "ESpriteEquipmentRawCommand");
    assert_eq!(symbols.raw_command_signatures(), "ISpriteEquipmentRawCommandSignatures");
    assert_eq!(symbols.bulk_routes(), "spriteEquipmentBulkRoutes");
    assert_eq!(symbols.bulk_route_enum(), "ESpriteEquipmentBulkRoute");
    assert_eq!(symbols.bulk_route_signatures(), "ISpriteEquipmentBulkRouteSignatures");
  }

  #[test]
  fn a_command_name_becomes_a_member_and_a_wrapper_key() {
    assert_eq!(
      PluginSymbols::member("describe_console_defaults"),
      "DESCRIBE_CONSOLE_DEFAULTS"
    );
    assert_eq!(
      PluginSymbols::wrapper("describe_console_defaults"),
      "describeConsoleDefaults"
    );
  }
}
