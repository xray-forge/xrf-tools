# xrf-test-utils

Shared fixture paths, file helpers, and assertions for XRF Rust tests. Add it as a dev-dependency.

## Write a generated fixture

```rust
use xrf_test_utils::utils::{build_absolute_generated_test_resource_path, write_generated_test_resource};

fn main() -> std::io::Result<()> {
  let path = write_generated_test_resource("readme/example.ltx", "[actor]\nhealth = 100\n")?;

  assert_eq!(path, build_absolute_generated_test_resource_path("readme/example.ltx"));
  assert_eq!(std::fs::read_to_string(path)?, "[actor]\nhealth = 100\n");

  Ok(())
}
```

The writer creates parent directories, replaces an existing generated file, and returns its path. Use a unique relative
name for tests that may run concurrently; the same name within one test process addresses the same file.

## Resource helpers

Use `utils::build_absolute_generated_test_resource_path` for a generated file or tree, or
`utils::write_generated_test_resource` to write one resource. The sample helpers derive a resource subdirectory from
the test source filename. `assertions` contains shared comparisons; `file` contains file-reading helpers.

| Need                                     | Helper                                           |
| ---------------------------------------- | ------------------------------------------------ |
| Reserve a path without writing           | `build_absolute_generated_test_resource_path`    |
| Write bytes or text                      | `write_generated_test_resource`                  |
| Group a fixture by test source filename  | `build_absolute_generated_test_sample_file_path` |
| Read a generated fixture through a slice | `open_generated_test_resource_as_slice`          |
| Open a generated fixture as a file       | `open_generated_test_resource_as_file`           |

`FileSlice` is re-exported for binary-format tests. The path helpers do not determine the fixture's encoding or validate
its format; write the bytes the test is intended to exercise.

## Location and lifetime

Generated resources live under the workspace's `target/test-resources/<target>-<pid>-<nanos>` directory. Later runs
attempt to collect generated run directories older than one hour. The path helper allocates a name; callers creating
a tree must create its directories. Keep committed fixtures separate from generated output.

The generated root is chosen once per test process. Cleanup is best effort and applies only to run directories with
the expected naming shape; it is not immediate cleanup after each test. Fixtures needed beyond a test run should not
use this storage.

## Errors and checks

File helpers return `std::io::Result`, preserving the failure kind with path context where applicable. This crate is
for the XRF workspace layout: its generated root is derived from its own manifest location.

Run `cargo test --locked -p xrf-test-utils` from the repository root.

See the [resource helpers](src/utils.rs) and [assertions](src/assertions.rs).
