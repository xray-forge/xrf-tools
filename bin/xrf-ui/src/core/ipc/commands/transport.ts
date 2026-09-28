// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import { TransportEndpoint } from "@/core/ipc/types/xrf-app";

/** Commands */
export const transportCommands = {
  /** Where the transport listens and the token it expects, which the frontend asks once. */
  getEndpoint: () => __TAURI_INVOKE<TransportEndpoint>("plugin:transport|get_endpoint"),
};
