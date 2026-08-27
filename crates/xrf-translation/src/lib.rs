//! Reading, editing, and building S.T.A.L.K.E.R. translation tables.
//!
//! The tree follows the data rather than the operations: `xml` and `json` each own one file format
//! end to end, and `project` owns whole trees of them - discovery, description, editing, and the
//! build, verify and initialize passes.

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
pub use crate::project::build::options::ProjectBuildOptions;
pub use crate::project::build::result::ProjectBuildResult;
pub use crate::project::build::run::{build_dir, build_file};
pub use crate::project::descriptor::{
  TranslationFile, TranslationFinding, TranslationProjectDescriptor, TranslationProjectMode, TranslationSource,
};
pub use crate::project::edit::{apply_edits, apply_edits_to_asset, find_unwritable_character};
pub use crate::project::gamedata_read::{read_gamedata, read_gamedata_in};
pub use crate::project::initialize::options::ProjectInitializeOptions;
pub use crate::project::initialize::result::ProjectInitializeResult;
pub use crate::project::initialize::run::{initialize_dir, initialize_file};
pub use crate::project::layout::{detect_mode, detect_mode_in};
pub use crate::project::parse::options::ProjectParseOptions;
pub use crate::project::parse::result::{ProjectParseCensus, ProjectParseResult};
pub use crate::project::parse::run::{parse_translations, parse_translations_in};
pub use crate::project::source_read::{read_source, read_source_in};
pub use crate::project::verify::options::ProjectVerifyOptions;
pub use crate::project::verify::result::ProjectVerifyResult;
pub use crate::project::verify::run::{verify_dir, verify_file};
pub use crate::types::{TranslationEntry, TranslationJson, TranslationVariant};
