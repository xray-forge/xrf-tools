import { Injectable } from "@wirestate/core";
import { BoundAction, Computed } from "@wirestate/mobx";
import { SceneToolKind } from "@xrf/ipc/types/xrf-app-levels";
import { Logger } from "@xrf/logging";

import {
  booleanCodec,
  jsonCodec,
  LEVEL_EDITOR_COLLISION_STORAGE_KEY,
  LEVEL_EDITOR_EDITOR_ONLY_STORAGE_KEY,
  LEVEL_EDITOR_HELPERS_STORAGE_KEY,
  StoredValue,
} from "@/core/storage";
import {
  ILevelEditorHelperOptions,
  ILevelEditorHelperToolsChange,
  toLevelEditorHelperOptions,
} from "@/tools/level-editor/lib/helpers/level-editor-helper-options";

/**
 * What the Show action draws beside a scene's objects: which helpers over what a scene holds with no geometry, the
 * master switch and each helped tool's, whether the surfaces the game never draws are drawn, and whether those dressed
 * in the SDK's marker textures are; preferences of the editor's own, kept over runs and apart from the level viewer's.
 */
@Injectable()
export class LevelEditorHelpersService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly stored: StoredValue<ILevelEditorHelperOptions> = new StoredValue(
    LEVEL_EDITOR_HELPERS_STORAGE_KEY,
    jsonCodec(toLevelEditorHelperOptions)
  );

  /** Hidden until asked for, as the game hides them: a tree's hull or a level's invisible walls hide what they hold. */
  private readonly storedCollision: StoredValue<boolean> = new StoredValue(
    LEVEL_EDITOR_COLLISION_STORAGE_KEY,
    booleanCodec(false)
  );

  /** Hidden until asked for, as a player never sees them: a sun blocker or an occluder's sheet hides what it covers. */
  private readonly storedEditorOnly: StoredValue<boolean> = new StoredValue(
    LEVEL_EDITOR_EDITOR_ONLY_STORAGE_KEY,
    booleanCodec(false)
  );

  /** Which helpers are drawn: a preference, kept over runs. */
  @Computed()
  public get helpers(): ILevelEditorHelperOptions {
    return this.stored.value;
  }

  /**
   * Whether the surfaces whose compiler shader leaves them out of the game's drawing, kept for collision or shadows
   * alone, are drawn: a preference, kept over runs.
   */
  @Computed()
  public get isCollisionShown(): boolean {
    return this.storedCollision.value;
  }

  /**
   * Whether the surfaces dressed in the SDK's marker textures, which only show the editor what a level holds unseen,
   * are drawn: a preference, kept over runs.
   */
  @Computed()
  public get isEditorOnlyShown(): boolean {
    return this.storedEditorOnly.value;
  }

  /** Draws the helpers or leaves them all out, each tool keeping its choice. */
  @BoundAction()
  public toggleHelpers(): void {
    this.stored.set({ ...this.helpers, isShown: !this.helpers.isShown });
  }

  /** Draws the surfaces the game never draws, or leaves them out. */
  @BoundAction()
  public toggleCollision(): void {
    this.storedCollision.set(!this.isCollisionShown);
  }

  /** Draws the surfaces dressed in the SDK's marker textures, or leaves them out. */
  @BoundAction()
  public toggleEditorOnly(): void {
    this.storedEditorOnly.set(!this.isEditorOnlyShown);
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
