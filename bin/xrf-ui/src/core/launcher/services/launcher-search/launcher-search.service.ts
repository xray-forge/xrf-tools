import { Injectable, OnDeprovision } from "@wirestate/core";
import { BoundAction, Observable, runInAction } from "@wirestate/mobx";

import { Command } from "@/core/commands";
import { FOCUS_LAUNCHER_SEARCH_COMMAND } from "@/core/launcher/commands";

/**
 * Owns whether the home screen's search field can be reached, and asks it to take the caret.
 */
@Injectable()
export class LauncherSearchService {
  /** Whether a search field is currently mounted to receive the caret. */
  @Observable()
  public isMounted: boolean = false;

  /** Bumped per request; the mounted field focuses and selects on every change. */
  @Observable()
  public focusRevision: number = 0;

  @OnDeprovision()
  public onDeprovision(): void {
    runInAction(() => {
      this.isMounted = false;
      this.focusRevision = 0;
    });
  }

  /**
   * @param isMounted - Whether a search field is on screen.
   */
  @BoundAction()
  public setMounted(isMounted: boolean): void {
    this.isMounted = isMounted;
  }

  /** Asks the mounted search field for the caret. */
  @Command(FOCUS_LAUNCHER_SEARCH_COMMAND, {
    isEnabled: (service: LauncherSearchService) => service.isMounted,
  })
  @BoundAction()
  public focusSearch(): void {
    this.focusRevision += 1;
  }
}
