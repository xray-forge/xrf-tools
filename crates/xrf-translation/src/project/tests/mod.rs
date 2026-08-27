mod edit;
mod gamedata_read;
mod layout;
mod source_read;

use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::{XrayMountMode, XrayRoots};

/// A one-entry string table, which is all most of these tests need on disk.
pub(crate) fn table(id: &str, text: &str) -> String {
  format!("<string_table>\n\t<string id=\"{id}\">\n\t\t<text>{text}</text>\n\t</string>\n</string_table>")
}

/// Mount a generated fixture tree as one root.
///
/// `Directory` rather than `Auto`: a temporary tree declares no installation, and letting the mount
/// search upward for one would make the test depend on whatever sits above the build directory.
pub(crate) fn roots(root: &str) -> XrayRoots {
  XrayRoots::one(
    build_absolute_generated_test_resource_path(root).display().to_string(),
    XrayMountMode::Directory,
  )
}
