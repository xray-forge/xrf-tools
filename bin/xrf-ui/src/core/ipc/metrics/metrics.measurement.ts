import { isIpcProfilingEnabled } from "@/core/ipc/metrics/metrics.profiling";
import { IIpcCallSink } from "@/core/ipc/metrics/metrics.types";
import { Nullable } from "@/lib/types/general";

export class IpcCallMeasurement {
  /** Whether this call weighs its payloads, as the switch stood when it started. */
  public readonly isWeighing: boolean = isIpcProfilingEnabled();

  private readonly sink: IIpcCallSink;
  private readonly command: string;
  private readonly startedAt: number = performance.now();

  /**
   * @param sink - What this call settles into.
   * @param command - Fully qualified command name, such as `plugin:visuals|read_geometry`.
   */
  public constructor(sink: IIpcCallSink, command: string) {
    this.sink = sink;
    this.command = command;

    sink.enterFlight();
  }

  /**
   * Settles a call that answered.
   *
   * @param received - Response bytes, or null when the response was not weighed.
   * @param sent - Request bytes, or null when the request was not weighed.
   */
  public recordAnswer(received: Nullable<number>, sent: Nullable<number>): void {
    this.sink.recordAnswer(this.command, this.elapsed(), received, sent);
  }

  /** Settles a call that rejected, into the pool kept apart from the successes. */
  public recordFailure(): void {
    this.sink.recordFailure(this.command, this.elapsed());
  }

  /** @returns How long this call has been in flight, read before anything else so both outcomes measure one span. */
  private elapsed(): number {
    return performance.now() - this.startedAt;
  }
}
