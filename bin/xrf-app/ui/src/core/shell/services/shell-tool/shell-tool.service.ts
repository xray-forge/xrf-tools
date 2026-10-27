import { Injectable, OnDeprovision } from "@wirestate/core";
import { BoundAction, Computed, RefObservable, runInAction } from "@wirestate/mobx";
import { Nullable } from "@xrf/types";

import { EToolId } from "@/core/routing/tool";

/** Leaves the open tool for the launcher at once. */
export type TShellToolLeave = () => void;

/**
 * Which tool the window shows, as the route names it, and leaving it for the launcher, which the route does. The frame
 * keeps both current while it is mounted.
 */
@Injectable()
export class ShellToolService {
  /** The open tool, or null on the launcher. */
  @RefObservable()
  public tool: Nullable<EToolId> = null;

  private leave: Nullable<TShellToolLeave> = null;

  /** Whether a tool is open rather than the launcher. */
  @Computed()
  public get isToolOpen(): boolean {
    return this.tool !== null;
  }

  @OnDeprovision()
  public onDeprovision(): void {
    runInAction(() => {
      this.tool = null;
      this.leave = null;
    });
  }

  /**
   * Follows the route the frame shows.
   *
   * @param tool - The tool it opens, or null for the launcher.
   * @param leave - Leaves it for the launcher.
   */
  @BoundAction()
  public follow(tool: Nullable<EToolId>, leave: TShellToolLeave): void {
    this.tool = tool;
    this.leave = leave;
  }

  /**
   * Stops following the route, as the frame that followed it goes.
   *
   * @param leave - The leave it followed with, so an older frame's release keeps a newer frame's route.
   */
  @BoundAction()
  public release(leave: TShellToolLeave): void {
    if (this.leave === leave) {
      this.tool = null;
      this.leave = null;
    }
  }

  /** Leaves the open tool for the launcher at once; whoever calls it has already asked about unsaved work. */
  @BoundAction()
  public close(): void {
    if (this.tool !== null) {
      this.leave?.();
    }
  }
}
