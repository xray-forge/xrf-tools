//! The commands and routes this crate's plugin answers, expanded from its token list in `domains.rs`.

use xrf_app_core::define_runtime_domains;

#[macro_use]
mod domains;

for_each_translations_domain!(define_runtime_domains);
