use crate::definition::thunderbolt::thunderbolt_settings_key::ThunderboltSettingsKey;
use crate::section::EnvironmentSection;

/// `environment.ltx`'s `[environment]`: where the engine strikes every bolt from.
pub type ThunderboltSettings = EnvironmentSection<ThunderboltSettingsKey>;
