import { IpcCallMeasurement } from "@/core/ipc/metrics/metrics.measurement";
import { isIpcProfilingEnabled } from "@/core/ipc/metrics/metrics.profiling";
import { IIpcCallSink, IIpcCommandMetrics, IIpcMetricsSnapshot } from "@/core/ipc/metrics/metrics.types";
import { Nullable, Optional } from "@/lib/types/general";

/** How a command names itself on the wire, which is not how it is counted. */
const WIRE_PREFIX: string = "plugin:";

/**
 * What the frontend has asked of the backend, since this window loaded or since it was last reset.
 */
export class IpcMetricsRecorder implements IIpcCallSink {
  private readonly commands: Map<string, IIpcCommandMetrics> = new Map();

  private startedAt: number = performance.now();
  private inFlight: number = 0;
  private peakInFlight: number = 0;

  /**
   * Begins counting one command call.
   *
   * @param command - Fully qualified command name, such as `plugin:visuals|read_geometry`.
   * @returns The measurement, to be settled exactly once.
   */
  public measure(command: string): IpcCallMeasurement {
    return new IpcCallMeasurement(this, command);
  }

  public enterFlight(): void {
    this.inFlight += 1;

    if (this.inFlight > this.peakInFlight) {
      this.peakInFlight = this.inFlight;
    }
  }

  public recordAnswer(command: string, duration: number, received: Nullable<number>, sent: Nullable<number>): void {
    const entry: IIpcCommandMetrics = this.leaveFlightInto(command);

    entry.calls += 1;
    entry.duration += duration;

    if (duration > entry.slowest) {
      entry.slowest = duration;
    }

    // Counted apart from the call, because a size is known for every byte command and only for a weighed JSON one.
    if (received !== null) {
      entry.received += received;
      entry.weighed += 1;
    }

    if (sent !== null) {
      entry.sent += sent;
    }
  }

  public recordFailure(command: string, duration: number): void {
    const entry: IIpcCommandMetrics = this.leaveFlightInto(command);

    entry.failures += 1;
    entry.failureDuration += duration;
  }

  /**
   * Reads what has been counted.
   *
   * Entries are copied, so a render cannot see a counter move underneath it.
   *
   * @returns Every command called at least once, and the totals over them.
   */
  public read(): IIpcMetricsSnapshot {
    const snapshot: IIpcMetricsSnapshot = {
      elapsed: performance.now() - this.startedAt,
      calls: 0,
      failures: 0,
      duration: 0,
      received: 0,
      sent: 0,
      weighed: 0,
      inFlight: this.inFlight,
      peakInFlight: this.peakInFlight,
      isProfiling: isIpcProfilingEnabled(),
      commands: [],
    };

    // Summed here rather than kept alongside: two sets of counters are two things that can disagree, and a hundred
    // entries add up once a second.
    for (const entry of this.commands.values()) {
      snapshot.calls += entry.calls;
      snapshot.failures += entry.failures;
      snapshot.duration += entry.duration;
      snapshot.received += entry.received;
      snapshot.sent += entry.sent;
      snapshot.weighed += entry.weighed;
      snapshot.commands.push({ ...entry });
    }

    return snapshot;
  }

  /**
   * Forgets everything counted so far and restarts the clock.
   *
   * Peak concurrency restarts at what is in flight right now rather than at zero, which is already true of this
   * instant.
   */
  public reset(): void {
    this.commands.clear();

    this.startedAt = performance.now();
    this.peakInFlight = this.inFlight;
  }

  /**
   * @param command - Fully qualified command name of the call that is over.
   * @returns The entry it is counted in, with the call no longer in flight.
   */
  private leaveFlightInto(command: string): IIpcCommandMetrics {
    this.inFlight -= 1;

    const name: string = command.startsWith(WIRE_PREFIX) ? command.slice(WIRE_PREFIX.length) : command;
    const known: Optional<IIpcCommandMetrics> = this.commands.get(name);

    if (known) {
      return known;
    }

    const entry: IIpcCommandMetrics = {
      command: name,
      calls: 0,
      failures: 0,
      duration: 0,
      failureDuration: 0,
      slowest: 0,
      received: 0,
      sent: 0,
      weighed: 0,
    };

    this.commands.set(name, entry);

    return entry;
  }
}

/**
 * The recorder every seam reports to.
 *
 * One instance rather than a bound service, because the generated bindings reach it on import, before any container
 * exists: a call made while the application is still starting is still a call.
 */
export const IPC_METRICS: IpcMetricsRecorder = new IpcMetricsRecorder();
