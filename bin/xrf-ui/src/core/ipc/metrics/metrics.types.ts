import { Nullable } from "@/lib/types/general";

export interface IIpcCommandMetrics {
  /** Command as the panel names it, with the `plugin:` prefix stripped. */
  command: string;
  /** Calls that resolved. */
  calls: number;
  /** Calls that rejected. */
  failures: number;
  /** Milliseconds spent in resolved calls. Concurrent calls overlap, so this can exceed elapsed time. */
  duration: number;
  /** Milliseconds spent in rejected calls. */
  failureDuration: number;
  /** The longest resolved call, in milliseconds. */
  slowest: number;
  /** Response bytes: exact for byte commands, and known for JSON ones only while profiling was on. */
  received: number;
  /** Request bytes, known only for calls made while profiling was on. */
  sent: number;
  /** Resolved calls whose response size is known, which is the population `received` is true of. */
  weighed: number;
}

export interface IIpcMetricsSnapshot {
  /** Milliseconds since the recorder started, which a reset restarts. */
  elapsed: number;
  /** Resolved calls across every command. */
  calls: number;
  /** Rejected calls across every command. */
  failures: number;
  /** Milliseconds spent in resolved calls across every command. */
  duration: number;
  /** Response bytes measured across every command. */
  received: number;
  /** Request bytes measured across every command. */
  sent: number;
  /** Resolved calls whose response size is known. */
  weighed: number;
  /** Calls started and not yet settled. */
  inFlight: number;
  /** The most that were ever in flight at once, which is what lets `duration` exceed `elapsed`. */
  peakInFlight: number;
  /** Whether payloads are being weighed right now. */
  isProfiling: boolean;
  /** One entry per command called at least once, in no particular order. */
  commands: Array<IIpcCommandMetrics>;
}

/**
 * What a call in flight reports itself to.
 *
 * The measurement depends on this role rather than on the recorder, which keeps the two files free of a cycle and
 * says exactly what a call is allowed to do: enter the flight, and settle once.
 */
export interface IIpcCallSink {
  /** Counts one more call in flight, and raises the peak when this is the most there have been. */
  enterFlight(): void;
  /**
   * Records a call that answered.
   *
   * @param command - Command name as the wire spells it.
   * @param duration - How long it was in flight, in milliseconds.
   * @param received - Response bytes, or null when the response was not weighed.
   * @param sent - Request bytes, or null when the request was not weighed.
   */
  recordAnswer(command: string, duration: number, received: Nullable<number>, sent: Nullable<number>): void;
  /**
   * Records a call that rejected.
   *
   * @param command - Command name as the wire spells it.
   * @param duration - How long it was in flight, in milliseconds.
   */
  recordFailure(command: string, duration: number): void;
}
