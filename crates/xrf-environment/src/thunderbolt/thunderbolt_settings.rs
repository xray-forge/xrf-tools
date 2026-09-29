use crate::section::EnvironmentSection;
use crate::thunderbolt::thunderbolt_settings_key::ThunderboltSettingsKey;

/// `environment.ltx`'s `[environment]`: where the engine strikes every bolt from.
pub type ThunderboltSettings = EnvironmentSection<ThunderboltSettingsKey>;
