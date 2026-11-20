import { Injectable } from "@wirestate/core";
import { BoundAction, Computed } from "@wirestate/mobx";
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
  ILevelEditorHelperToolChange,
  toLevelEditorHelperOptions,
} from "@/tools/level-editor/lib/helpers/level-editor-helper-options";

/**
 * The Show action's switches: helpers, collision-only surfaces, and marker-textured surfaces.
 *
 * Preferences kept across runs, apart from the level viewer's.
 */
@Injectable()
export class LevelEditorHelpersService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly stored: StoredValue<ILevelEditorHelperOptions> = new StoredValue(
    LEVEL_EDITOR_HELPERS_STORAGE_KEY,
    jsonCodec(toLevelEditorHelperOptions)
  );

  /** Off by default, as the game hides them: a tree's hull or invisible walls hide what they hold. */
  private readonly storedCollision: StoredValue<boolean> = new StoredValue(
    LEVEL_EDITOR_COLLISION_STORAGE_KEY,
    booleanCodec(false)
  );

  /** Off by default, as a player never sees them: a sun blocker or an occluder's sheet hides what it covers. */
  private readonly storedEditorOnly: StoredValue<boolean> = new StoredValue(
    LEVEL_EDITOR_EDITOR_ONLY_STORAGE_KEY,
    booleanCodec(false)
  );

  @Computed()
  public get helpers(): ILevelEditorHelperOptions {
    return this.stored.value;
  }

  /** Whether surfaces the compiler shader keeps out of the game's drawing, for collision or shadows alone, are drawn. */
  @Computed()
  public get isCollisionShown(): boolean {
    return this.storedCollision.value;
  }

  /** Whether surfaces in the SDK's marker textures, which only show the editor what is unseen, are drawn. */
  @Computed()
  public get isEditorOnlyShown(): boolean {
    return this.storedEditorOnly.value;
  }

  /** Flips the helpers' master switch, each tool keeping its choice. */
  @BoundAction()
  public toggleHelpers(): void {
    this.stored.set({ ...this.helpers, isShown: !this.helpers.isShown });
  }

  @BoundAction()
  public toggleCollision(): void {
    this.storedCollision.set(!this.isCollisionShown);
  }

  @BoundAction()
  public toggleEditorOnly(): void {
    this.storedEditorOnly.set(!this.isEditorOnlyShown);
  }

  /** Switches one helped tool's helpers, which draw only while the master switch is on. */
  @BoundAction()
  public setHelperTool(change: ILevelEditorHelperToolChange): void {
    const { tool, isOn } = change;

    this.stored.set({ ...this.helpers, tools: { ...this.helpers.tools, [tool]: isOn } });
  }
}
