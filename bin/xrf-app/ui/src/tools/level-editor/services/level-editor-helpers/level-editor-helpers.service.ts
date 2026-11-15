import { Injectable } from "@wirestate/core";
import { BoundAction, Computed } from "@wirestate/mobx";
import { SceneToolKind } from "@xrf/ipc/types/xrf-app-levels";
import { Logger } from "@xrf/logging";

import { jsonCodec, LEVEL_EDITOR_HELPERS_STORAGE_KEY, StoredValue } from "@/core/storage";
import {
  ILevelEditorHelperOptions,
  ILevelEditorHelperToolsChange,
  toLevelEditorHelperOptions,
} from "@/tools/level-editor/lib/helpers/level-editor-helper-options";

/**
 * Which helpers the editor draws over what a scene holds with no geometry: the master switch and each helped tool's, a
 * preference of the editor's own, kept over runs and apart from the level viewer's.
 */
@Injectable()
export class LevelEditorHelpersService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly stored: StoredValue<ILevelEditorHelperOptions> = new StoredValue(
    LEVEL_EDITOR_HELPERS_STORAGE_KEY,
    jsonCodec(toLevelEditorHelperOptions)
  );

  /** Which helpers are drawn: a preference, kept over runs. */
  @Computed()
  public get helpers(): ILevelEditorHelperOptions {
    return this.stored.value;
  }

  /** Draws the helpers or leaves them all out, each tool keeping its choice. */
  @BoundAction()
  public toggleHelpers(): void {
    this.stored.set({ ...this.helpers, isShown: !this.helpers.isShown });
  }

  /**
   * @param change - Helped tools, and whether their helpers are drawn from now on, while helpers are.
   */
  @BoundAction()
  public setHelperTools(change: ILevelEditorHelperToolsChange): void {
    const { tools, isOn } = change;

    this.stored.set({
      ...this.helpers,
      tools: { ...this.helpers.tools, ...Object.fromEntries(tools.map((tool: SceneToolKind) => [tool, isOn])) },
    });
  }
}
