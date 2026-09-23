import { Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, RefObservable } from "@wirestate/mobx";

import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { DEFAULT_TEXTURE_LIGHTING } from "@/core/textures/lib/texture-lighting";
import { DEFAULT_TEXTURE_PREVIEW_OPTIONS, ITexturePreviewOptions } from "@/core/textures/lib/texture-preview";

/**
 * How a texture is looked at: what the toolbar has switched on, rather than what the texture is.
 */
@Injectable()
export class TextureViewService {
  /** What the toolbar is asking for. */
  @RefObservable()
  public options: ITexturePreviewOptions = DEFAULT_TEXTURE_PREVIEW_OPTIONS;

  /** What the lit body is lit with, which the toolbar sets and a drag over the body also changes. */
  @RefObservable()
  public lighting: IRenderLighting = DEFAULT_TEXTURE_LIGHTING;

  @BoundAction()
  public setOptions(options: ITexturePreviewOptions): void {
    this.options = options;
  }

  @BoundAction()
  public setLighting(lighting: IRenderLighting): void {
    this.lighting = lighting;
  }

  /** Back to the defaults, so a texture opened again does not inherit the last one's toggles. */
  @OnDeactivation()
  @BoundAction()
  public clear(): void {
    this.options = DEFAULT_TEXTURE_PREVIEW_OPTIONS;
    this.lighting = DEFAULT_TEXTURE_LIGHTING;
  }
}
