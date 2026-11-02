# xrf-test-utils

Shared support for XRF Rust tests: where generated test files live, writing and reading them, checking what a refused
publication left, filesystem links and non-UTF-8 names for path tests, and skipping GPU tests on hosts without a device.
Add it as a dev-dependency.

## Write a generated fixture

```rust
use std::path::PathBuf;

use xrf_test_utils::GeneratedResources;

fn main() -> std::io::Result<()> {
  let path: PathBuf = GeneratedResources::write("readme/example.ltx", "[actor]\nhealth = 100\n")?;

  assert_eq!(path, GeneratedResources::path("readme/example.ltx"));
  assert_eq!(std::fs::read_to_string(path)?, "[actor]\nhealth = 100\n");

  Ok(())
}
```

`GeneratedResources::write` takes text or bytes, creates the parent directories, truncates any earlier file, closes it,
and returns its path. Every test in one process shares one root, so give each test its own relative name.

## Location and lifetime

Generated files live under the workspace's `target/test-resources/<target>-<pid>-<nanos>/`, chosen once per test
process: `<target>` is the test binary's name without Cargo's hash, `<nanos>` the start time. The first call in a
process also deletes run directories of that shape started more than an hour earlier; anything else under
`target/test-resources` is left alone. Cleanup is best effort and never immediate, so do not keep anything there a
later run needs. Never use `std::env::temp_dir()` for test output: nothing collects it and process ids are recycled.

Committed fixtures stay in their crates; this storage is for output a test generates. A fixture that describes a whole
tree should clear its root first, so a rerun does not see files from the last one. The root is derived from this
crate's manifest location, so it assumes the XRF workspace layout.

## Types

Every type is a unit struct exported at the crate root. A `GeneratedResources` resource is a name relative to the run's
directory; operations ending in `path` or `root` answer absolute paths.

| Operation                                     | Use                                                              |
| --------------------------------------------- | ---------------------------------------------------------------- |
| `GeneratedResources::path`                    | A path in this run's directory; creates nothing                  |
| `GeneratedResources::write`                   | Write one file and get its path                                  |
| `GeneratedResources::create_dir`              | An empty directory, cleared of an earlier run's files            |
| `GeneratedResources::overwrite`               | Create or truncate a generated file, with its parents            |
| `GeneratedResources::overwrite_path`          | The same for an already resolved path                            |
| `GeneratedResources::open`                    | Open a generated file                                            |
| `GeneratedResources::open_slice`              | Open a generated file as a `FileSlice`                           |
| `GeneratedResources::sample_name`             | A resource name grouped under the test source's file stem        |
| `GeneratedResources::sample_path`             | That grouped name made absolute                                  |
| `GeneratedResources::sample_root`             | The absolute directory a test source's names are grouped in      |
| `HostFilesystem::link_directory`, `link_file` | Create a link for a containment test                             |
| `HostFilesystem::non_unicode_file_name`       | A file name the host accepts and `Path::to_str` refuses          |
| `RefusedPublication::assert_left_previous`    | Assert a refused staged write left the previous file and no more |
| `GpuRequirement::skip_without_device`         | Skip a GPU test that found no device                             |

`FileSlice` from the `fileslice` crate is re-exported for binary-format tests. The operations write the bytes they are
given and validate no format. File operations return `std::io::Result`, keeping the error kind and naming the path. Read
an open file to the end with `std::io::read_to_string`.

`sample_name(file!(), "x.chunk")` gives `<stem>/x.chunk`, joined with the host's separator, so the tests of one source
file share a directory.

`HostFilesystem::non_unicode_file_name` is invalid UTF-8 on Unix and an unpaired surrogate on Windows. NTFS refuses to
create the Windows form, so a test that writes the file gates itself on `cfg(unix)`; a test about paths alone runs
everywhere.

`link_directory` and `link_file` return `false`, after printing why, when the host cannot create the link; the test
then skips. On Windows, `link_directory` falls back to a junction when symlinks need a privilege the account lacks,
while `link_file` has no fallback.

`GpuRequirement::skip_without_device(reason)` prints the reason and returns, so the test can return early. When the
test-only variable `GpuRequirement::VARIABLE` (`XRF_TEST_REQUIRE_GPU`) is set to anything but `0`, as Windows CI does
with WARP, it panics instead, so a missing device fails the run.

## Checks

Run `cargo test --locked -p xrf-test-utils` from the repository root. See the
[generated resources](src/generated_resources.rs), [host filesystem](src/host_filesystem.rs),
[refused publication](src/refused_publication.rs) and [GPU requirement](src/gpu_requirement.rs).
