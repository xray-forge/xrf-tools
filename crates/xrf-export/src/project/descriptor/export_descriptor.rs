use crate::manifest::ExternExport;
use crate::parse::ParsedExtern;
use crate::project::descriptor::export_contract_descriptor::ExportContractDescriptor;
use crate::project::descriptor::export_parameter_descriptor::ExportParameterDescriptor;
use crate::project::descriptor::export_return_descriptor::ExportReturnDescriptor;
use crate::project::descriptor::export_source_descriptor::ExportSourceDescriptor;

/// One extern declaration projected for the application-facing exports project.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportDescriptor {
  pub name: String,
  pub description: Option<String>,
  pub source: ExportSourceDescriptor,
  #[serde(flatten)]
  pub contract: ExportContractDescriptor,
}

/// A parsed declaration as the app shows it: a callable's description and return description from its JSDoc, a value's
/// description alone.
impl From<ParsedExtern> for ExportDescriptor {
  fn from(parsed: ParsedExtern) -> Self {
    let source: ExportSourceDescriptor = ExportSourceDescriptor::from(parsed.location);

    match parsed.export {
      ExternExport::Callable(callable) => {
        let (description, return_description) = callable
          .doc
          .map(|documentation| (documentation.description, documentation.returns))
          .unwrap_or_default();

        Self {
          name: parsed.name,
          description,
          source,
          contract: ExportContractDescriptor::Callable {
            parameters: callable
              .params
              .into_iter()
              .map(ExportParameterDescriptor::from)
              .collect(),
            returns: ExportReturnDescriptor {
              typing: callable.returns,
              description: return_description,
            },
          },
        }
      }
      ExternExport::Value(value) => Self {
        name: parsed.name,
        description: value.doc.and_then(|documentation| documentation.description),
        source,
        contract: ExportContractDescriptor::Value {
          typing: value.type_name,
        },
      },
    }
  }
}
