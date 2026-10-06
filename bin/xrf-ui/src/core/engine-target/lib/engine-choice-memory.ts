import { Nullable } from "@xrf/types";

import { EXrayEngineChoice } from "@/core/ipc/types/xrf-engine-target";
import { ENGINE_CHOICES_STORAGE_KEY } from "@/core/storage";
import { parseLocalStorageValueSafe, setLocalStorageValueSafe } from "@/lib/local-storage";

/** Roots remembered at most, the least recently told let go first. */
const REMEMBERED_ROOTS: number = 32;

/**
 * @param root - A game root as a person picked it.
 * @returns What its choice is remembered under: case and trailing separators folded, as the host folds them.
 */
export function toEngineChoiceMemoryKey(root: string): string {
  return root.replace(/[\\/]+$/, "").toLowerCase();
}

/**
 * @param root - A game root, or null for none picked.
 * @returns The engine the root was last told to be read as, or Auto where it never was.
 */
export function readEngineChoice(root: Nullable<string>): EXrayEngineChoice {
  if (!root) {
    return EXrayEngineChoice.AUTO;
  }

  const stored: unknown = readChoices()[toEngineChoiceMemoryKey(root)];

  return isOverride(stored) ? stored : EXrayEngineChoice.AUTO;
}

/**
 * Remembers an override for a root, or forgets it for Auto, which is what an unremembered root reads as.
 *
 * @param root - A game root.
 * @param choice - What it is read as now.
 */
export function writeEngineChoice(root: string, choice: EXrayEngineChoice): void {
  const key: string = toEngineChoiceMemoryKey(root);
  const choices: Record<string, unknown> = readChoices();

  delete choices[key];

  // Insertion order is recency: the newest last, the oldest first to go.
  const kept: Array<[string, unknown]> = Object.entries(choices).filter(([, stored]) => isOverride(stored));

  if (choice !== EXrayEngineChoice.AUTO) {
    kept.push([key, choice]);
  }

  setLocalStorageValueSafe(
    ENGINE_CHOICES_STORAGE_KEY,
    JSON.stringify(Object.fromEntries(kept.slice(-REMEMBERED_ROOTS)))
  );
}

function readChoices(): Record<string, unknown> {
  const stored: unknown = parseLocalStorageValueSafe(ENGINE_CHOICES_STORAGE_KEY);

  return stored && typeof stored === "object" && !Array.isArray(stored)
    ? { ...(stored as Record<string, unknown>) }
    : {};
}

function isOverride(value: unknown): value is EXrayEngineChoice.VANILLA | EXrayEngineChoice.EXTENDED {
  return value === EXrayEngineChoice.VANILLA || value === EXrayEngineChoice.EXTENDED;
}
