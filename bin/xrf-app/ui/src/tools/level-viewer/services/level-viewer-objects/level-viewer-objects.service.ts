import { inject, Injectable } from "@wirestate/core";
import { call, LatestFlow, TFlow } from "@xrf/flow";
import { levelsCommands } from "@xrf/ipc/commands/levels";
import { LevelObject, LevelSpawnObjectDetails } from "@xrf/ipc/types/xrf-app-levels";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { KeybindCommand } from "@/core/commands";
import { FRAME_LEVEL_SELECTION_KEYBIND_COMMAND, ISOLATE_LEVEL_SELECTION_KEYBIND_COMMAND } from "@/core/level/commands";
import { toLevelFramedGoTo } from "@/core/level/lib/camera/level-camera-frame";
import { getLevelObjectKind, getLevelSpawnIndex } from "@/core/level/lib/objects/level-object-kinds";
import {
  ILevelObjectSphere,
  toLevelObjectSphere,
  toLevelObjectsSphere,
} from "@/core/level/lib/objects/level-object-sphere";
import { LevelLoadService } from "@/core/level/services/level-load";
import { LevelRenderService } from "@/core/level/services/level-render";
import { LevelViewService } from "@/core/level/services/level-view";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { CommandOperation } from "@/core/operations/lib/command-operation";

/**
 * What the viewer does with the level objects chosen: what the backend says of a spawned one beyond what the listing
 * holds, framing them with the camera, and hiding the rest.
 */
@Injectable()
export class LevelViewerObjectsService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** What the backend says of the spawned object the Inspector shows, read when it is shown. */
  public readonly details: CommandOperation<LevelSpawnObjectDetails> = new CommandOperation(this.log);

  public constructor(
    private readonly loadService: LevelLoadService = inject(LevelLoadService),
    private readonly renderService: LevelRenderService = inject(LevelRenderService),
    private readonly viewService: LevelViewService = inject(LevelViewService),
    private readonly viewportService: LevelViewportService = inject(LevelViewportService)
  ) {}

  /** The level opening the objects belong to, or null for none open. */
  private get sessionId(): Nullable<string> {
    return this.loadService.level.value?.selected.sessionId ?? null;
  }

  /**
   * Reads what the backend says of one spawned object of the open level, the answer for one shown before it dropped.
   *
   * @param index - The object, by its index among the level's spawned objects, or null to forget what was read.
   */
  @LatestFlow()
  public *describe(index: Nullable<number>): TFlow {
    const sessionId: Nullable<string> = this.sessionId;

    this.details.reset();

    if (sessionId === null || index === null) {
      return;
    }

    yield* this.details.run({
      name: "Describe spawned object",
      subject: String(index),
      command: () => levelsCommands.describeSpawnObject(sessionId, index).then(({ value }) => value),
      failure: "Could not describe the spawned object",
    });
  }

  /**
   * Stands the camera framing objects: a drawn one by the sphere its model spans, as the renderer holds it once drawn,
   * a volume by its shapes, a patrol path by its points, anything else by a metre about its place.
   *
   * @param objects - The objects.
   */
  @LatestFlow()
  public *goTo(objects: ReadonlyArray<LevelObject>): TFlow {
    const sessionId: Nullable<string> = this.sessionId;
    const spheres: Array<ILevelObjectSphere> = [];

    for (const object of objects) {
      const index: Nullable<number> = getLevelSpawnIndex(object.id);
      const drawn: Nullable<[number, number, number, number]> =
        index !== null && getLevelObjectKind(object.kind).isDrawn
          ? yield* call(this.renderService.locateSpawnObject(index))
          : null;

      spheres.push(toLevelObjectSphere(object, drawn));
    }

    const sphere: Nullable<ILevelObjectSphere> = toLevelObjectsSphere(spheres);

    // Another level opened while the renderer answered: the objects are not where the camera now is.
    if (!sphere || this.sessionId !== sessionId) {
      return;
    }

    this.renderService.goTo(
      toLevelFramedGoTo(this.viewportService.camera, sphere, this.viewService.cameraOptions.fieldOfView)
    );
  }

  /** Stands the camera framing what is selected, as Unreal's `F` does. */
  @KeybindCommand(FRAME_LEVEL_SELECTION_KEYBIND_COMMAND, {
    isEnabled: (service: LevelViewerObjectsService) => service.viewportService.selected.length > 0,
  })
  public frameSelection(): void {
    void this.goTo(this.viewportService.selected);
  }

  /** Hides every object of the open level but the selected. */
  @KeybindCommand(ISOLATE_LEVEL_SELECTION_KEYBIND_COMMAND, {
    isEnabled: (service: LevelViewerObjectsService) =>
      service.viewportService.selected.length > 0 && service.loadService.objects !== null,
  })
  public isolateSelection(): void {
    this.viewportService.isolateSelected(this.loadService.objects?.objects ?? []);
  }
}
