import { Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, RefObservable } from "@wirestate/mobx";

import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS, IVisualPreviewViewOptions } from "@/core/visuals/lib/scene";
import { DEFAULT_VISUAL_LIGHTING } from "@/core/visuals/lib/scene/visual-lighting";

/**
 * How a visual is looked at: what the toolbar has switched on, rather than what the model is.
 */
@Injectable()
export class VisualViewService {
  /** What the toolbar has switched on. */
  @RefObservable()
  public options: IVisualPreviewViewOptions = DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS;

  /** What the model is lit by, which is the viewer's own answer rather than anything the model carries. */
  @RefObservable()
  public lighting: IRenderLighting = DEFAULT_VISUAL_LIGHTING;

  /** How far down each submesh's collapse chain to draw, `0` being full detail and `1` the coarsest. */
  @RefObservable()
  public detail: number = 0;

  @BoundAction()
  public setOptions(options: IVisualPreviewViewOptions): void {
    this.options = options;
  }

  @BoundAction()
  public setLighting(lighting: IRenderLighting): void {
    this.lighting = lighting;
  }

  @BoundAction()
  public setDetail(detail: number): void {
    this.detail = detail;
  }

  /** Back to the defaults, so a visual opened again does not inherit the last one's toggles. */
  @OnDeactivation()
  @BoundAction()
  public clear(): void {
    this.options = DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS;
    this.lighting = DEFAULT_VISUAL_LIGHTING;
    this.detail = 0;
  }
}
