use xrf_app_core::session::Sessions;
use xrf_translation::TranslationProjectDescriptor;

/// The translations projects open, each held by the tool that opened it and read again by a commit that moved a file it
/// read.
pub type TranslationProjects = Sessions<TranslationProjectDescriptor>;
