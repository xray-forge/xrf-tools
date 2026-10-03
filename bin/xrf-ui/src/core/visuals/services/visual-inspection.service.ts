import { Computed } from "@wirestate/mobx";
import { Nullable } from "@xrf/types";

import { SelectedVisualDescription } from "@/core/ipc/types/xrf-app";
import { VisualBone } from "@/core/ipc/types/xrf-visual";
import { IVisualBoneControls, IVisualInspection } from "@/core/visuals/components/panels/visual-inspection";
import { IVisualBumpStatus } from "@/core/visuals/lib/visual-bump";
import { IVisualTextureStatus } from "@/core/visuals/lib/visual-texture";
import { IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { IOpenVisual, VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { AsyncState } from "@/lib/async-state";

/**
 * The visual an application has open, forwarded from the loader for the panels that inspect it.
 *
 * Forwarded rather than mirrored: two copies of one state is how a screen ends up disagreeing with itself. What an
 * application decides for itself - where a source is searched, how the model stands, whether bones can be marked -
 * stays with the application's own service.
 */
export abstract class VisualInspectionService implements IVisualInspection {
  protected constructor(protected readonly loadService: VisualLoadService) {}

  /** Bone marking and hiding, or null on a surface that offers neither. */
  public abstract get boneControls(): Nullable<IVisualBoneControls>;

  /**
   * @returns The visual being shown, straight from the loader.
   */
  @Computed()
  public get visual(): AsyncState<IOpenVisual> {
    return this.loadService.visual;
  }

  @Computed()
  public get model(): Nullable<IVisualModelViews> {
    return this.loadService.model;
  }

  @Computed()
  public get textureStatuses(): ReadonlyMap<number, IVisualTextureStatus> {
    return this.loadService.textureStatuses;
  }

  @Computed()
  public get sessionId(): Nullable<string> {
    return this.loadService.sessionId;
  }

  @Computed()
  public get hasBump(): boolean {
    return this.loadService.hasBump;
  }

  @Computed()
  public get bumpStatuses(): ReadonlyMap<number, IVisualBumpStatus> {
    return this.loadService.bumpStatuses;
  }

  @Computed()
  public get sourceLabel(): Nullable<string> {
    return this.loadService.sourceLabel;
  }

  @Computed()
  public get hasMotions(): boolean {
    return this.loadService.hasMotions;
  }

  /**
   * @returns What the backend reported about the open visual, or null when nothing is open.
   *
   * The one place the async state is unwrapped for its contents, so a panel asking what the model contains does not also
   * acquire an opinion about whether it is still arriving.
   */
  @Computed()
  public get selected(): Nullable<SelectedVisualDescription> {
    return this.visual.value?.selected.value ?? null;
  }

  /**
   * @returns The open model's skeleton, or no bones at all when nothing is open.
   */
  @Computed()
  public get bones(): Array<VisualBone> {
    return this.selected?.description.bones ?? [];
  }
}
