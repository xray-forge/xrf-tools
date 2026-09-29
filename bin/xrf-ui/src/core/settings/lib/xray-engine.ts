import { Nullable } from "@xrf/types";

import { EXrayEngine } from "@/core/ipc/types/xrf-engine-target";

/** What a first run reads configs as: most trees are the stock game or OpenXRay. */
export const DEFAULT_XRAY_ENGINE: EXrayEngine = EXrayEngine.VANILLA;

/**
 * @param value - Raw stored value, or `null` when absent.
 * @returns The matching engine, or `DEFAULT_XRAY_ENGINE` when there is none.
 */
export function toXrayEngine(value: Nullable<string>): EXrayEngine {
  return Object.values(EXrayEngine).find((engine: EXrayEngine) => engine === value) ?? DEFAULT_XRAY_ENGINE;
}
