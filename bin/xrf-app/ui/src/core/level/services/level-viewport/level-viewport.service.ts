import { Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, Computed, RefObservable } from "@wirestate/mobx";
import { LevelObject } from "@xrf/ipc/types/xrf-app-levels";
import { RenderLoadReport } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";

import { KeybindCommand } from "@/core/commands";
import {
  CLEAR_LEVEL_SELECTION_KEYBIND_COMMAND,
  HIDE_LEVEL_SELECTION_KEYBIND_COMMAND,
  REVEAL_LEVEL_OBJECTS_KEYBIND_COMMAND,
} from "@/core/level/commands";
import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { toLevelObjectKey } from "@/core/level/lib/objects/level-object-kinds";
import { ELevelPick, TLevelPick } from "@/core/level/lib/pick/level-pick";

/** How a pick changes what is selected. */
export enum ELevelSelectionMode {
  /** It is the selection: its object alone, or nothing for anything else. */
  REPLACE = "replace",
  /** Its object joins the selection, or leaves it where it was in it. */
  TOGGLE = "toggle",
}

/**
 * What the viewport reports about itself while it draws, what is selected in it, and what is hidden of it.
 */
@Injectable()
export class LevelViewportService {
  /** Where the camera is and where it faces, or null until the viewport has drawn a frame. */
  @RefObservable()
  public camera: Nullable<ILevelCamera> = null;

  /**
   * What was picked last, the selection's active element: what the readout and the Inspector show. Null where nothing
   * of the level is picked.
   */
  @RefObservable()
  public picked: Nullable<TLevelPick> = null;

  /** The level objects selected, in the order they were chosen; the active one is `picked`'s. */
  @RefObservable()
  public selected: ReadonlyArray<LevelObject> = [];

  /** The level objects hidden until revealed or another level opens. */
  @RefObservable()
  public hidden: ReadonlyArray<LevelObject> = [];

  /**
   * Whether what is picked was found again in the open level's parts read anew, or let go with what it named, rather
   * than picked: what shows it follows it without showing it afresh.
   */
  @RefObservable()
  public isPickRelocated: boolean = false;

  /** Whether the next click in the viewport focuses the lens where it lands, rather than selecting what it meets. */
  @RefObservable()
  public isFocusPicking: boolean = false;

  /** Whether the scene has the keyboard, so keys meant for it act on what it shows rather than on a panel's. */
  @RefObservable()
  public isSceneFocused: boolean = false;

  /** How far the renderer has read the open level and put it on the GPU, or null until it says. */
  @RefObservable()
  public load: Nullable<RenderLoadReport> = null;

  /** The selected objects' keys. */
  @Computed()
  public get selectedKeys(): ReadonlySet<string> {
    return new Set(this.selected.map((object: LevelObject) => toLevelObjectKey(object.id)));
  }

  /** The hidden objects' keys. */
  @Computed()
  public get hiddenKeys(): ReadonlySet<string> {
    return new Set(this.hidden.map((object: LevelObject) => toLevelObjectKey(object.id)));
  }

  /**
   * @param report - How far the renderer has read the open level, or null for a level about to open.
   */
  @BoundAction()
  public noteLoad(report: Nullable<RenderLoadReport>): void {
    this.load = report;
  }

  /**
   * @param pick - What a click or a row picked, or null for nothing of the level.
   * @param mode - How it changes the selection: an object toggled in or out of it, or the selection replaced.
   */
  @BoundAction()
  public notePicked(pick: Nullable<TLevelPick>, mode: ELevelSelectionMode = ELevelSelectionMode.REPLACE): void {
    this.isPickRelocated = false;

    if (mode === ELevelSelectionMode.TOGGLE && pick?.kind === ELevelPick.OBJECT) {
      const key: string = toLevelObjectKey(pick.object.id);

      if (this.selectedKeys.has(key)) {
        const isActive: boolean =
          this.picked?.kind === ELevelPick.OBJECT && toLevelObjectKey(this.picked.object.id) === key;

        this.selected = this.selected.filter((object: LevelObject) => toLevelObjectKey(object.id) !== key);
        this.picked = isActive ? null : this.picked;
      } else {
        this.selected = [...this.selected, pick.object];
        this.picked = pick;
      }

      return;
    }

    this.picked = pick;
    this.selected = pick?.kind === ELevelPick.OBJECT ? [pick.object] : [];
  }

  /**
   * Selects a run of objects at once, as a range chosen in the Outliner does.
   *
   * @param objects - The objects selected.
   * @param active - The one picked last, which the readout and the Inspector show.
   */
  @BoundAction()
  public noteSelected(objects: ReadonlyArray<LevelObject>, active: TLevelPick): void {
    this.picked = active;
    this.selected = objects;
    this.isPickRelocated = false;
  }

  /**
   * @param pick - What was picked, as the open level's parts read anew name it, or null where they name it no longer.
   * @param selected - The objects selected, as they name them, those gone left out.
   * @param hidden - The objects hidden, as they name them, those gone left out.
   */
  @BoundAction()
  public notePickRelocated(
    pick: Nullable<TLevelPick>,
    selected: ReadonlyArray<LevelObject> = this.selected,
    hidden: ReadonlyArray<LevelObject> = this.hidden
  ): void {
    this.picked = pick;
    this.selected = selected;
    this.hidden = hidden;
    this.isPickRelocated = true;
  }

  /**
   * @param isPicking - Whether the next click focuses the lens rather than selecting.
   */
  @BoundAction()
  public setFocusPicking(isPicking: boolean): void {
    this.isFocusPicking = isPicking;
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

  /** Hides the selected objects, which stay selected. */
  @KeybindCommand(HIDE_LEVEL_SELECTION_KEYBIND_COMMAND, {
    isEnabled: (service: LevelViewportService) => service.selected.length > 0,
  })
  @BoundAction()
  public hideSelected(): void {
    const hidden: ReadonlySet<string> = this.hiddenKeys;

    this.hidden = [
      ...this.hidden,
      ...this.selected.filter((object: LevelObject) => !hidden.has(toLevelObjectKey(object.id))),
    ];
  }

  /**
   * Hides one object, or shows it again where it is hidden, as an Outliner row's eye does.
   *
   * @param object - The object.
   */
  @BoundAction()
  public toggleHidden(object: LevelObject): void {
    const key: string = toLevelObjectKey(object.id);

    this.hidden = this.hiddenKeys.has(key)
      ? this.hidden.filter((it: LevelObject) => toLevelObjectKey(it.id) !== key)
      : [...this.hidden, object];
  }

  /**
   * Hides every object but the selected, and shows the selected where they were hidden.
   *
   * @param objects - Every object of the open level.
   */
  @BoundAction()
  public isolateSelected(objects: ReadonlyArray<LevelObject>): void {
    const selected: ReadonlySet<string> = this.selectedKeys;

    this.hidden = objects.filter((object: LevelObject) => !selected.has(toLevelObjectKey(object.id)));
  }

  /** Shows every hidden object again. */
  @KeybindCommand(REVEAL_LEVEL_OBJECTS_KEYBIND_COMMAND, {
    isEnabled: (service: LevelViewportService) => service.hidden.length > 0,
  })
  @BoundAction()
  public revealAll(): void {
    this.hidden = [];
  }

  /**
   * @param camera - Where the camera is now, in the level's own coordinates.
   */
  @BoundAction()
  public noteCamera(camera: ILevelCamera): void {
    this.camera = camera;
  }

  /** Forgets the open level's telemetry, selection and hidden objects, so a closed viewer reports nothing. */
  @BoundAction()
  public clear(): void {
    this.load = null;
    this.camera = null;
    this.notePicked(null);
    this.hidden = [];
    this.isFocusPicking = false;
    this.isSceneFocused = false;
  }

  @OnDeactivation()
  public onDeactivation(): void {
    this.clear();
  }
}
