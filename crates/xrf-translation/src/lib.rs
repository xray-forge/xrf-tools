//! Reading, editing, and building S.T.A.L.K.E.R. translation tables.
//!
//! The tree follows the data rather than the operations: `xml` and `json` each own one file format
//! end to end, and `project` owns whole trees of them - discovery, description, editing, and the
//! build, verify, initialize and format passes.

pub(crate) mod edit;
pub(crate) mod json;
pub(crate) mod language;
pub(crate) mod project;
pub(crate) mod source_file_name;
pub(crate) mod staged_write;
pub(crate) mod types;
pub(crate) mod xml;

pub use crate::edit::TranslationEdit;
pub use crate::language::TranslationLanguage;
pub use crate::project::build::translation_build_options::TranslationBuildOptions;
pub use crate::project::build::translation_build_result::{TranslationBuildLanguageSummary, TranslationBuildResult};
pub use crate::project::build::translation_builder::TranslationBuilder;
pub use crate::project::descriptor::{
  TranslationFile, TranslationFinding, TranslationProjectDescriptor, TranslationProjectMode, TranslationSource,
};
pub use crate::project::edit::{apply_edits, apply_edits_to_asset, find_unwritable_character};
pub use crate::project::format::translation_format_options::TranslationFormatOptions;
pub use crate::project::format::translation_format_result::TranslationFormatResult;
pub use crate::project::format::translation_formatter::TranslationFormatter;
pub use crate::project::gamedata_read::{read_gamedata, read_gamedata_in};
pub use crate::project::initialize::translation_initialize_options::TranslationInitializeOptions;
pub use crate::project::initialize::translation_initialize_result::TranslationInitializeResult;
pub use crate::project::initialize::translation_initializer::TranslationInitializer;
pub use crate::project::job_phases::{
  TRANSLATION_PHASE_BUILD, TRANSLATION_PHASE_FORMAT, TRANSLATION_PHASE_PARSE, TRANSLATION_PHASE_VERIFY,
};
pub use crate::project::layout::{detect_mode, detect_mode_in};
pub use crate::project::parse::translation_parse_options::TranslationParseOptions;
pub use crate::project::parse::translation_parse_result::{TranslationParseCensus, TranslationParseResult};
pub use crate::project::parse::translation_parser::TranslationParser;
pub use crate::project::source_read::{read_source, read_source_in};
pub use crate::project::verify::translation_verifier::TranslationVerifier;
pub use crate::project::verify::translation_verify_options::TranslationVerifyOptions;
pub use crate::project::verify::translation_verify_result::{
  TranslationVerifyLanguageSummary, TranslationVerifyResult,
};
pub use crate::types::{TranslationEntry, TranslationJson, TranslationVariant};
