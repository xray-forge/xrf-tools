//! One extern as the desktop app shows it: its name and description, where it is declared, and its contract.

pub(crate) mod export_contract_descriptor;
pub(crate) mod export_descriptor;
pub(crate) mod export_parameter_descriptor;
pub(crate) mod export_return_descriptor;
pub(crate) mod export_source_descriptor;

pub use crate::project::descriptor::export_contract_descriptor::ExportContractDescriptor;
pub use crate::project::descriptor::export_descriptor::ExportDescriptor;
pub use crate::project::descriptor::export_parameter_descriptor::ExportParameterDescriptor;
pub use crate::project::descriptor::export_return_descriptor::ExportReturnDescriptor;
pub use crate::project::descriptor::export_source_descriptor::ExportSourceDescriptor;
