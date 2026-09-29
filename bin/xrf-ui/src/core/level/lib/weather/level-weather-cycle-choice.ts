import { EnvironmentCycleEntry, LevelWeatherDescription } from "@/core/ipc/types/xrf-app";
import { EWeatherCycleKind, LevelWeatherOption } from "@/core/ipc/types/xrf-environment";

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
