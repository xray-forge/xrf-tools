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
