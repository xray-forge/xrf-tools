#![doc = include_str!("../README.md")]

pub(crate) mod alife;
pub(crate) mod alife_context;
pub(crate) mod schedule;
pub(crate) mod settings;
pub(crate) mod switch;

#[cfg(test)]
mod tests;

pub use crate::alife::Alife;
pub use crate::alife_context::AlifeContext;
pub use crate::schedule::alife_schedule::AlifeSchedule;
pub use crate::schedule::alife_schedule_report::AlifeScheduleReport;
pub use crate::settings::alife_settings::AlifeSettings;
pub use crate::switch::alife_switch::AlifeSwitch;
pub use crate::switch::alife_switch_report::AlifeSwitchReport;
