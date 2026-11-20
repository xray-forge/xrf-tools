//! Configs read through a search: the VFS answer to `xrf_ltx::LtxDocumentSource`, and the listings it shares.

pub(crate) mod ltx_listing_cache;
pub(crate) mod ltx_vfs_source;

pub(crate) use crate::source::ltx_listing_cache::LtxListingCache;
pub(crate) use crate::source::ltx_vfs_source::LtxVfsSource;
