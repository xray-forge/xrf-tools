// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import {
  EnvironmentCatalogDescription,
  EnvironmentCycleDescription,
  EnvironmentRequest,
} from "@/core/ipc/types/xrf-app";
import { EWeatherCycleKind } from "@/core/ipc/types/xrf-environment";

/** Commands */
export const environmentCommands = {
  /** Read a game's environment configs afresh as one engine reads them, and list what they hold. */
  readCatalog: (request: EnvironmentRequest) =>
    __TAURI_INVOKE<EnvironmentCatalogDescription>("plugin:environment|read_catalog", { request }),
  /** Read one cycle or effect as authored, recording where each value came from, with every finding in its config. */
  readCycle: (request: EnvironmentRequest, kind: EWeatherCycleKind, name: string) =>
    __TAURI_INVOKE<EnvironmentCycleDescription>("plugin:environment|read_cycle", { request, kind, name }),
};
