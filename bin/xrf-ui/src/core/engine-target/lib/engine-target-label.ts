import { Nullable } from "@xrf/types";

import { XrayEngine, XrayEngineEvidence, XrayEngineResolution } from "@/core/ipc/types/xrf-engine-target";

const ENGINE_LABELS: Record<XrayEngine, string> = {
  extended: "Extended",
  vanilla: "Vanilla",
};

const EVIDENCE_LABELS: Record<XrayEngineEvidence, string> = {
  anomalyExecutables: "Anomaly executables found",
  anomalyFsgame: "Anomaly fsgame.ltx found",
  atmosfearCycles: "Atmosfear weather cycles found",
  named: "named",
  noSigns: "no Anomaly or Atmosfear signs",
};

/**
 * @param engine - An engine.
 * @returns Its name as a person reads it.
 */
export function getEngineLabel(engine: XrayEngine): string {
  return ENGINE_LABELS[engine];
}

/**
 * @param resolution - An engine and what decided it.
 * @returns Both in one line, `Extended (Anomaly executables found)`.
 */
export function describeEngineResolution(resolution: XrayEngineResolution): string {
  return `${getEngineLabel(resolution.engine)} (${EVIDENCE_LABELS[resolution.evidence]})`;
}

/**
 * @param detection - What detection found, or null before it answered or where it could not.
 * @param isDetecting - Whether detection is still running.
 * @returns What Auto means for the roots: `Auto: Vanilla (no Anomaly or Atmosfear signs)`.
 */
export function toAutoEngineLabel(detection: Nullable<XrayEngineResolution>, isDetecting: boolean): string {
  if (isDetecting) {
    return "Auto: detecting";
  }

  return detection ? `Auto: ${describeEngineResolution(detection)}` : "Auto";
}
