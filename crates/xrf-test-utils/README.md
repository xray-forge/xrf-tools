# xrf-test-utils

Shared support for XRF Rust tests: where generated test files live, writing and reading them, checking what a refused
publication left, filesystem links for containment tests, and skipping GPU tests on hosts without a device. Add it as a
dev-dependency.

## Write a generated fixture

```rust
use std::path::PathBuf;

use xrf_test_utils::utils::{build_absolute_generated_test_resource_path, write_generated_test_resource};

fn main() -> std::io::Result<()> {
  let path: PathBuf = write_generated_test_resource("readme/example.ltx", "[actor]\nhealth = 100\n")?;

  assert_eq!(path, build_absolute_generated_test_resource_path("readme/example.ltx"));
  assert_eq!(std::fs::read_to_string(path)?, "[actor]\nhealth = 100\n");

  Ok(())
}
```

`write_generated_test_resource` takes text or bytes, creates the parent directories, truncates any earlier file, closes
it, and returns its path. Every test in one process shares one root, so give each test its own relative name.

## Location and lifetime

Generated files live under the workspace's `target/test-resources/<target>-<pid>-<nanos>/`, chosen once per test
process: `<target>` is the test binary's name without Cargo's hash, `<nanos>` the start time. The first call in a
process also deletes run directories of that shape started more than an hour earlier; anything else under
`target/test-resources` is left alone. Cleanup is best effort and never immediate, so do not keep anything there a
later run needs. Never use `std::env::temp_dir()` for test output: nothing collects it and process ids are recycled.

Committed fixtures stay in their crates; this storage is for output a test generates. A fixture that describes a whole
tree should clear its root first, so a rerun does not see files from the last one. The root is derived from this
crate's manifest location, so it assumes the XRF workspace layout.

## Helpers

Helpers are in `utils` unless the name carries another module:

| Helper                                           | Use                                                              |
| ------------------------------------------------ | ---------------------------------------------------------------- |
| `build_absolute_generated_test_resource_path`    | A path in this run's directory; creates nothing                  |
| `write_generated_test_resource`                  | Write one file and get its path                                  |
| `create_generated_test_resource_dir`             | An empty directory, cleared of an earlier run's files            |
| `overwrite_generated_test_resource_as_file`      | Create or truncate a generated file, with its parents            |
| `overwrite_file`                                 | The same for any path                                            |
| `open_generated_test_resource_as_file`           | Open a generated file                                            |
| `open_generated_test_resource_as_slice`          | Open a generated file as a `FileSlice`                           |
| `build_relative_test_sample_file_path`           | A relative path grouped under the test source's file stem        |
| `build_relative_test_sample_file_directory`      | That file stem alone                                             |
| `build_absolute_generated_test_sample_file_path` | The grouped path made absolute                                   |
| `build_non_unicode_file_name`                    | A file name the host accepts and `Path::to_str` refuses          |
| `assert_publication_left_previous`               | Assert a refused staged write left the previous file and no more |
| `file::read_file_as_string`                      | Read an open file to the end                                     |
| `links::link_directory`, `links::link_file`      | Create a link for a containment test                             |
| `gpu::skip_without_gpu`                          | Skip a GPU test that found no device                             |

`FileSlice` from the `fileslice` crate is re-exported for binary-format tests. The helpers write the bytes they are
given and validate no format. File helpers return `std::io::Result`, keeping the error kind and naming the path.

`build_non_unicode_file_name` is invalid UTF-8 on Unix and an unpaired surrogate on Windows. NTFS refuses to create the
Windows form, so a test that writes the file gates itself on `cfg(unix)`; a test about paths alone runs everywhere.

`link_directory` and `link_file` return `false`, after printing why, when the host cannot create the link; the test
then skips. On Windows, `link_directory` falls back to a junction when symlinks need a privilege the account lacks,
while `link_file` has no fallback.

`skip_without_gpu(reason)` prints the reason and returns, so the test can return early. When the test-only variable
`XRF_TEST_REQUIRE_GPU` is set to anything but `0`, as Windows CI does with WARP, it panics instead, so a missing
device fails the run.

## Checks

Run `cargo test --locked -p xrf-test-utils` from the repository root. See the [resource helpers](src/utils.rs),
[links](src/links.rs) and the [GPU skip](src/gpu.rs).
