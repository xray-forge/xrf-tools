import { Injectable } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { visualsCommands } from "@xrf/ipc/commands/visuals";
import { AssetSource, SessionId, SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { SelectedVisualDescription } from "@xrf/ipc/types/xrf-app-visuals";
import { RenderTextureReport } from "@xrf/ipc/types/xrf-renderer";
import { XrayRoots } from "@xrf/ipc/types/xrf-vfs";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { IRenderSurfaceDraw } from "@/core/render/lib/surface/render-surface-draw";
import { SessionRootResource } from "@/core/session/lib/session-root-resource";
import { IVisualRenderSource } from "@/core/visuals/lib/render/visual-render-source";
import { IVisualBumpStatus, toLoadableBumps } from "@/core/visuals/lib/visual-bump";
import { describeVisualSource } from "@/core/visuals/lib/visual-source";
import { createVisualSurfaces } from "@/core/visuals/lib/visual-surface";
import { IVisualTextureStatus } from "@/core/visuals/lib/visual-texture";
import { toVisualBumpStatuses, toVisualTextureStatuses } from "@/core/visuals/lib/visual-texture-report";
import { createVisualViews, IVisualModelViews } from "@/core/visuals/lib/visual-views";

/** A visual that is loaded: what it is, where it came from, and the views the scene draws. */
export interface IOpenVisual {
  selected: SessionSnapshot<SelectedVisualDescription>;
  views: IVisualModelViews;
}

/** What the renderer last said of the textures of one opening. */
interface IVisualTextureReports {
  sessionId: SessionId;
  reports: Array<RenderTextureReport>;
}

/**
 * Opens a visual's session and publishes what the viewer reads of it; the renderer reads its geometry and textures
 * through the session itself, and says what became of each texture once it has.
 */
@Injectable()
export class VisualLoadService implements IVisualRenderSource {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The open visual's session and description. */
  public readonly selection: SessionRootResource<SelectedVisualDescription> = new SessionRootResource(this.log, {
    subject: "visual",
    close: visualsCommands.closeModel,
    restore: visualsCommands.restoreModel,
  });

  /** What the renderer last said of the textures of the visual it drew. */
  @Observable()
  private textureReports: Nullable<IVisualTextureReports> = null;

  /** The visual being shown, with the views the scene draws of it. */
  @Computed()
  public get visual(): AsyncState<IOpenVisual> {
    return this.selection.mapSnapshot((selected: SessionSnapshot<SelectedVisualDescription>): IOpenVisual => {
      // Joined once, for the toolbar and the panels to read how each submesh is drawn.
      const surfaces: Map<number, IRenderSurfaceDraw> = createVisualSurfaces(
        selected.value.description.submeshes,
        selected.value.surfaces
      );

      return { selected, views: createVisualViews(selected.value.description, surfaces) };
    });
  }

  /**
   * @returns The model on screen, or null while nothing is open. What a viewport draws, as opposed to what the
   *   open itself is doing.
   */
  @Computed()
  public get model(): Nullable<IVisualModelViews> {
    return this.visual.value?.views ?? null;
  }

  /** What became of each submesh's texture, every located texture loading until the renderer says. */
  @Computed()
  public get textureStatuses(): ReadonlyMap<number, IVisualTextureStatus> {
    const selected: Nullable<SelectedVisualDescription> = this.selection.value;

    return selected ? toVisualTextureStatuses(selected, this.reports) : new Map();
  }

  /** What became of each submesh's bump inputs, each half on its own. */
  @Computed()
  public get bumpStatuses(): ReadonlyMap<number, IVisualBumpStatus> {
    const selected: Nullable<SelectedVisualDescription> = this.selection.value;

    return selected ? toVisualBumpStatuses(selected, this.reports) : new Map();
  }

  /**
   * @returns The open visual's session, which the renderer reads it through, or null while nothing is open.
   */
  @Computed()
  public get sessionId(): Nullable<string> {
    return this.selection.sessionId;
  }

  /**
   * @returns Whether any submesh binds a bump pair with both halves located; a dummy pair counts, since it is shaded.
   */
  @Computed()
  public get hasBump(): boolean {
    const selected: Nullable<SelectedVisualDescription> = this.selection.value;

    return Boolean(selected && toLoadableBumps(selected.dependencies.textures, selected.materials).length);
  }

  /**
   * @returns The path or entry the loaded visual was read from, or null when nothing is loaded.
   */
  @Computed()
  public get sourceLabel(): Nullable<string> {
    const selected: Nullable<SelectedVisualDescription> = this.selection.value;

    return selected ? describeVisualSource(selected.source) : null;
  }

  /**
   * @returns Whether the loaded visual animates from anything, referenced or embedded.
   */
  @Computed()
  public get hasMotions(): boolean {
    const selected: Nullable<SelectedVisualDescription> = this.selection.value;

    return Boolean(selected && (selected.dependencies.motions.length || selected.description.embeddedMotions.length));
  }

  /** What the renderer said of the visual held, or nothing for one it has not drawn yet. */
  @Computed()
  private get reports(): Array<RenderTextureReport> {
    return this.textureReports?.sessionId === this.selection.sessionId ? this.textureReports.reports : [];
  }

  /**
   * Load a visual and put it on screen.
   *
   * @param source - Visual source to open.
   * @param roots - Roots the source and its references are searched in.
   * @returns Whether the visual opened.
   */
  @BoundAction()
  public load(source: AssetSource, roots: XrayRoots): Promise<boolean> {
    return flowResult(this.selection.open(describeVisualSource(source), visualsCommands.openModel, source, roots));
  }

  /**
   * Restores the native selection unless a user action has taken the visual flow.
   */
  @BoundAction()
  public restore(): Promise<void> {
    return flowResult(this.selection.restore());
  }

  /**
   * Closes this loader's document without clearing a newer view.
   */
  @BoundAction()
  public async close(): Promise<void> {
    await flowResult(this.selection.close());
  }

  /**
   * Abandons pending loads and releases the current document and textures, for a surface leaving while its tool stays.
   */
  @BoundAction()
  public clear(): void {
    this.selection.release();
  }

  /**
   * Notes what the renderer made of each texture the open visual samples.
   *
   * @param sessionId - The session the renderer drew, so a report on a visual since replaced changes nothing.
   * @param reports - What it said of every texture.
   */
  @BoundAction()
  public noteTextures(sessionId: string, reports: Array<RenderTextureReport>): void {
    if (this.selection.sessionId === sessionId) {
      this.textureReports = { sessionId, reports };
    }
  }
}
