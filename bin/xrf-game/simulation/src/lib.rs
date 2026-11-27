#![doc = include_str!("../README.md")]

pub(crate) mod config;
pub(crate) mod facts;
pub(crate) mod props;
pub(crate) mod smart;
pub(crate) mod squad;
pub(crate) mod start;

#[cfg(test)]
mod tests;

pub use crate::config::simulation_config::SimulationConfig;
pub use crate::config::simulation_config_source::SimulationConfigSource;
pub use crate::facts::simulation_facts::SimulationFacts;
pub use crate::props::simulation_props::SimulationProps;
pub use crate::smart::smart_config::SmartConfig;
pub use crate::smart::smart_respawn::SmartRespawn;
pub use crate::smart::smart_spawn::SmartSpawn;
pub use crate::squad::squad_behaviour::SquadBehaviour;
pub use crate::squad::squad_description::SquadDescription;
pub use crate::squad::squad_size::SquadSize;
pub use crate::start::simulation_place::SimulationPlace;
pub use crate::start::simulation_start::SimulationStart;
pub use crate::start::simulation_start_report::SimulationStartReport;
pub use crate::start::start_squad::StartSquad;
