// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@tauri-apps/api/core";

import { LtxProjectFormatResult, LtxProjectVerifyResult } from "@/core/bindings/types/xrf-ltx";
import { XrayWorldSpec } from "@/core/bindings/types/xrf-vfs";

/** Commands */
export const configsCommands = {
  /**
   * Report which LTX configs a world exposes are misformatted.
   *
   * Read-only, so an archived config is checked like any other.
   */
  checkDirectoryFormat: (world: XrayWorldSpec, prefix: string | null) =>
    __TAURI_INVOKE<LtxProjectFormatResult>("plugin:configs|check_directory_format", { world, prefix }),
  /**
   * Rewrite the LTX configs a world exposes.
   *
   * Writing needs a file, so this refuses a project holding archived winners — the refusal comes from
   * `xrf-ltx` itself. Formatting an installation is therefore a legitimate refusal, not a gap.
   */
  formatDirectory: (world: XrayWorldSpec, prefix: string | null) =>
    __TAURI_INVOKE<LtxProjectFormatResult>("plugin:configs|format_directory", { world, prefix }),
  /**
   * Verify the LTX configs a world exposes.
   *
   * Read-only, so it goes through the world and covers archived configs too. `xrf-ltx` draws the same
   * line: its read-only check reads through the VFS where its rewrite refuses archived winners.
   */
  verifyDirectory: (world: XrayWorldSpec, prefix: string | null) =>
    __TAURI_INVOKE<LtxProjectVerifyResult>("plugin:configs|verify_directory", { world, prefix }),
};
