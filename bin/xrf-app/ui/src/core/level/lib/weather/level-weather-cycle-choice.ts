import { formatCount } from "@xrf/format";
import { EnvironmentCycleEntry, LevelWeatherDescription } from "@xrf/ipc/types/xrf-app-levels";
import { EWeatherCycleKind, LevelWeatherOption } from "@xrf/ipc/types/xrf-environment";
import { Nullable } from "@xrf/types";

/**
 * One cycle a viewer can play.
 */
export interface ILevelWeatherCycleChoice {
  name: string;
  /** Whether the level's `weathers` resolves to it, which puts it first. */
  isOffered: boolean;
  /** The graph states or Atmosfear presets that play it, as `graph: state`. */
  states: Array<string>;
  keyframes: number;
  findings: number;
}

/**
 * @param description - The open level's weather.
 * @returns The cycles the level offers, in the order it offers them, then every other cycle of the game by name.
 */
export function listLevelWeatherCycles(description: LevelWeatherDescription): Array<ILevelWeatherCycleChoice> {
  const offered: Array<ILevelWeatherCycleChoice> = description.offered.map((cycle) => ({
    findings: cycle.findings.length,
    isOffered: true,
    keyframes: cycle.keyframes.length,
    name: cycle.name,
    states: description.weather.options
      .filter((option: LevelWeatherOption) => option.cycle === cycle.name && option.graph !== null)
      .map((option: LevelWeatherOption) => (option.state ? `${option.graph}: ${option.state}` : String(option.graph))),
  }));
  const names: Set<string> = new Set(offered.map((it) => it.name));
  const others: Array<ILevelWeatherCycleChoice> = description.cycles
    .filter((entry: EnvironmentCycleEntry) => entry.kind === EWeatherCycleKind.CYCLE && !names.has(entry.name))
    .sort((a: EnvironmentCycleEntry, b: EnvironmentCycleEntry) => a.name.localeCompare(b.name))
    .map((entry: EnvironmentCycleEntry) => ({
      findings: entry.findings,
      isOffered: false,
      keyframes: entry.keyframes,
      name: entry.name,
      states: [],
    }));

  return [...offered, ...others];
}

/** @returns How many keyframes a cycle or effect plays, as a list of them says beside its name. */
export function formatLevelWeatherKeyframes(keyframes: number): string {
  return `${formatCount(keyframes)} ${keyframes === 1 ? "keyframe" : "keyframes"}`;
}

/** @returns How many problems reading a cycle or effect found, as a list of them says beside its name; null for none. */
export function formatLevelWeatherFindings(findings: number): Nullable<string> {
  return findings ? `${formatCount(findings)} ${findings === 1 ? "finding" : "findings"}` : null;
}
