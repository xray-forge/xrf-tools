import { Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, RefObservable } from "@wirestate/mobx";
import { RenderLoadReport } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";

import { KeybindCommand } from "@/core/commands";
import { CLEAR_LEVEL_SELECTION_KEYBIND_COMMAND } from "@/core/level/commands";
import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { TLevelPick } from "@/core/level/lib/pick/level-pick";

/**
 * What the viewport reports about itself while it draws, and what is selected in it.
 */
@Injectable()
export class LevelViewportService {
  /** Where the camera is and where it faces, or null until the viewport has drawn a frame. */
  @RefObservable()
  public camera: Nullable<ILevelCamera> = null;

  /** What the last click in the viewport picked, or null where it picked nothing of the level or none was made. */
  @RefObservable()
  public picked: Nullable<TLevelPick> = null;

  /**
   * Whether what is picked was found again in the open level's parts read anew, or let go with what it named, rather
   * than picked: what shows it follows it without showing it afresh.
   */
  @RefObservable()
  public isPickRelocated: boolean = false;

  /** Whether the scene has the keyboard, so keys meant for it act on what it shows rather than on a panel's. */
  @RefObservable()
  public isSceneFocused: boolean = false;

  /** How far the renderer has read the open level and put it on the GPU, or null until it says. */
  @RefObservable()
  public load: Nullable<RenderLoadReport> = null;

  /**
   * @param report - How far the renderer has read the open level, or null for a level about to open.
   */
  @BoundAction()
  public noteLoad(report: Nullable<RenderLoadReport>): void {
    this.load = report;
  }

  /**
   * @param pick - What a click picked, or null for nothing of the level.
   */
  @BoundAction()
  public notePicked(pick: Nullable<TLevelPick>): void {
    this.picked = pick;
    this.isPickRelocated = false;
  }

  /**
   * @param pick - What was picked, as the open level's parts read anew name it, or null where they name it no longer.
   */
  @BoundAction()
  public notePickRelocated(pick: Nullable<TLevelPick>): void {
    this.picked = pick;
    this.isPickRelocated = true;
  }

  /**
   * @param isFocused - Whether the scene has the keyboard now.
   */
  @BoundAction()
  public noteSceneFocused(isFocused: boolean): void {
    this.isSceneFocused = isFocused;
  }

  /** Drops what is selected, as `Escape` over the scene does. */
  @KeybindCommand(CLEAR_LEVEL_SELECTION_KEYBIND_COMMAND, {
    isEnabled: (service: LevelViewportService) => service.isSceneFocused && service.picked !== null,
  })
  @BoundAction()
  public clearPicked(): void {
    this.notePicked(null);
  }

  /**
   * @param camera - Where the camera is now, in the level's own coordinates.
   */
  @BoundAction()
  public noteCamera(camera: ILevelCamera): void {
    this.camera = camera;
  }

  /** Forgets the open level's telemetry, so a closed viewer reports nothing rather than its last frame. */
  @BoundAction()
  public clear(): void {
    this.load = null;
    this.camera = null;
    this.notePicked(null);
    this.isSceneFocused = false;
  }

  @OnDeactivation()
  public onDeactivation(): void {
    this.clear();
  }
}
