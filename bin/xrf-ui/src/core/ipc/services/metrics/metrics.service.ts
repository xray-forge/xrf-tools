import { Injectable, OnDeprovision, OnProvision, ProvisionId } from "@wirestate/core";
import { BoundAction, Observable } from "@wirestate/mobx";

import { IIpcMetricsSnapshot, IPC_METRICS, isIpcProfilingEnabled, setIpcProfilingEnabled } from "@/core/ipc/metrics";
import { Logger } from "@/lib/logging";

/**
 * What the frontend has asked of the backend, as a surface can read it.
 */
@Injectable()
export class IpcMetricsService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Whether payloads are weighed, mirrored so a view can render the switch. */
  @Observable()
  public isProfilingEnabled: boolean = isIpcProfilingEnabled();

  @OnProvision()
  public onProvision(provisionId: ProvisionId): void {
    this.log.info("Provisioning:", provisionId);
  }

  @OnDeprovision()
  public onDeprovision(provisionId: ProvisionId): void {
    this.log.info("Deprovisioning:", provisionId);
  }

  /**
   * Turns payload weighing on or off, and remembers the choice.
   *
   * @param isEnabled - Whether calls from here on weigh what they carry.
   */
  @BoundAction()
  public setProfilingEnabled(isEnabled: boolean): void {
    this.log.info("Set IPC profiling:", isEnabled);

    this.isProfilingEnabled = isEnabled;

    setIpcProfilingEnabled(isEnabled);
  }

  /** Forgets what has been counted and restarts the clock, leaving the switch as it stands. */
  @BoundAction()
  public reset(): void {
    this.log.info("Reset IPC metrics");

    IPC_METRICS.reset();
  }

  /** @returns What has been counted since the window loaded, or since the last reset. */
  public read(): IIpcMetricsSnapshot {
    return IPC_METRICS.read();
  }
}
