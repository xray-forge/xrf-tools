// Auto-generated rust bindings. Do not edit it manually.

/** The engine a game data tree targets, which decides how its configs are read where the engines disagree. */
export enum EXrayEngine {
  /** OpenXRay and the stock Call of Pripyat engine (`xray-16`). */
  VANILLA = "vanilla",
  /** Anomaly's Monolith engine (`xray-monolith`) and the builds that share its readers. */
  EXTENDED = "extended",
}

/** Every `EXrayEngine` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type XrayEngine = `${EXrayEngine}`;

/** Which engine a caller asks a tree to be read as: one it names, or whatever the tree shows. */
export enum EXrayEngineChoice {
  /** Detected from the tree's installation layout and data. */
  AUTO = "auto",
  /** OpenXRay and the stock Call of Pripyat engine, whatever the tree shows. */
  VANILLA = "vanilla",
  /** Anomaly's Monolith engine, whatever the tree shows. */
  EXTENDED = "extended",
}

/** Every `EXrayEngineChoice` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type XrayEngineChoice = `${EXrayEngineChoice}`;

/** What decided a tree's engine. */
export enum EXrayEngineEvidence {
  /** The caller named the engine. */
  NAMED = "named",
  /** `AnomalyLauncher.exe`, `bin/AnomalyDX*.exe` or `bin/VerifiedDX11.exe` beside `fsgame.ltx`. */
  ANOMALY_EXECUTABLES = "anomalyExecutables",
  /** `fsgame.ltx` declares `$warfare_presets$`. */
  ANOMALY_FSGAME = "anomalyFsgame",
  /** `configs/environment/dynamic_weather_graphs.ltx` has a `[weather_cycles]` section. */
  ATMOSFEAR_CYCLES = "atmosfearCycles",
  /** Nothing Anomaly or Atmosfear was found. */
  NO_SIGNS = "noSigns",
}

/** Every `EXrayEngineEvidence` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type XrayEngineEvidence = `${EXrayEngineEvidence}`;

/** The engine a tree is read as, and what decided it. */
export type XrayEngineResolution = {
  engine: XrayEngine;
  evidence: XrayEngineEvidence;
  /** The file or logical path that showed it, where one did. */
  subject: string | null;
};
