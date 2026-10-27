import { inject, Injectable } from "@wirestate/core";
import { call, LatestFlow, TFlow } from "@xrf/flow";
import { levelsCommands } from "@xrf/ipc/commands/levels";
import { LevelSpawnObject, LevelSpawnObjectDetails } from "@xrf/ipc/types/xrf-app-levels";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { toLevelFramedGoTo } from "@/core/level/lib/camera/level-camera-frame";
import { toLevelSpawnSphere } from "@/core/level/lib/spawn/level-spawn-sphere";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelRenderService } from "@/core/level/services/level-render.service";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { LevelViewportService } from "@/core/level/services/level-viewport.service";
import { CommandOperation } from "@/core/operations/lib/command-operation";

/**
 * The spawned object the Spawn panel has chosen: what the backend says of it beyond what the listing holds, and going
 * to it with the camera.
 */
@Injectable()
export class LevelViewerSpawnService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** What the backend says of the object chosen, read when it is chosen. */
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
   * Reads what the backend says of one object of the open level, the answer for one chosen before it dropped.
   *
   * @param index - The object, by its place among the level's spawned objects, or null to forget what was read.
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
   * Stands the camera framing an object by the sphere its model spans, as the renderer holds it: asked when gone to,
   * since it stands only once drawn.
   *
   * @param object - The object.
   */
  @LatestFlow()
  public *goTo(object: LevelSpawnObject): TFlow {
    const sessionId: Nullable<string> = this.sessionId;
    const sphere: Nullable<[number, number, number, number]> = yield* call(
      this.renderService.locateSpawnObject(object.index)
    );

    // Another level opened while the renderer answered: the object is not where the camera now is.
    if (this.sessionId !== sessionId) {
      return;
    }

    this.renderService.goTo(
      toLevelFramedGoTo(
        this.viewportService.camera,
        toLevelSpawnSphere(object, sphere),
        this.viewService.cameraOptions.fieldOfView
      )
    );
  }
}
