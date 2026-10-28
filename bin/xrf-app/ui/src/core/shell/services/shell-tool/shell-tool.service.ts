import { Injectable, OnDeprovision } from "@wirestate/core";
import { BoundAction, Computed, Observable, RefObservable, runInAction } from "@wirestate/mobx";
import { Nullable } from "@xrf/types";

import { IToolDescriptor } from "@/core/routing/tool";

/** Leaves the open tool for the launcher at once. */
export type TShellToolLeave = () => void;

/**
 * What the window shows, as the route names it: the open tool, the launcher, or neither (a route nothing answers), and
 * leaving the tool for the launcher, which the route does. The one place outside React that knows what is open, so the
 * keymap, help and the shell's commands read it rather than each being told. The frame keeps it current while it is
 * mounted.
 */
@Injectable()
export class ShellToolService {
  /** The open tool, or null on the launcher or a route nothing answers. */
  @RefObservable()
  public tool: Nullable<IToolDescriptor> = null;

  /** Whether the launcher is on screen, rather than a tool or a route nothing answers. */
  @Observable()
  public isLauncher: boolean = false;

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
      this.isLauncher = false;
      this.leave = null;
    });
  }

  /**
   * Follows the route the frame shows.
   *
   * @param tool - The tool it opens, or null where it opens none.
   * @param isLauncher - Whether it is the launcher's.
   * @param leave - Leaves it for the launcher.
   */
  @BoundAction()
  public follow(tool: Nullable<IToolDescriptor>, isLauncher: boolean, leave: TShellToolLeave): void {
    this.tool = tool;
    this.isLauncher = isLauncher;
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
      this.isLauncher = false;
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
