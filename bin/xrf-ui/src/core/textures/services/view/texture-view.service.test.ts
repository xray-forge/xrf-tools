import { describe, expect, it } from "@jest/globals";

import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { DEFAULT_TEXTURE_LIGHTING } from "@/core/textures/lib/scene/texture-lighting";
import {
  DEFAULT_TEXTURE_PREVIEW_OPTIONS,
  ETexturePreviewMode,
  ITexturePreviewOptions,
} from "@/core/textures/lib/texture-preview";
import { TextureViewService } from "@/core/textures/services/view";
import { mockInjectedService } from "@/fixtures/utils/container";

describe("TextureViewService", () => {
  it("starts with the defaults", () => {
    const { service } = mockInjectedService(TextureViewService);

    expect(service.options).toEqual(DEFAULT_TEXTURE_PREVIEW_OPTIONS);
    expect(service.lighting).toEqual(DEFAULT_TEXTURE_LIGHTING);
  });

  it("takes what the toolbar asked for", () => {
    const { service } = mockInjectedService(TextureViewService);

    service.setOptions({ ...DEFAULT_TEXTURE_PREVIEW_OPTIONS, mode: ETexturePreviewMode.SURFACE });
    service.setLighting({ ...DEFAULT_TEXTURE_LIGHTING, sunIntensity: 3 });

    expect(service.options.mode).toBe(ETexturePreviewMode.SURFACE);
    expect(service.lighting.sunIntensity).toBe(3);
  });

  // The same rule the level's view obeys: these are replaced whole and held by reference, so a value can be
  // handed to whatever draws it without being a proxy of itself.
  it("holds what it was given rather than a copy of it", () => {
    const { service } = mockInjectedService(TextureViewService);

    const options: ITexturePreviewOptions = { ...DEFAULT_TEXTURE_PREVIEW_OPTIONS };
    const lighting: IRenderLighting = { ...DEFAULT_TEXTURE_LIGHTING };

    service.setOptions(options);
    service.setLighting(lighting);

    expect(service.options).toBe(options);
    expect(service.lighting).toBe(lighting);
  });

  it("forgets the last texture's toggles", () => {
    const { service } = mockInjectedService(TextureViewService);

    service.setOptions({ ...DEFAULT_TEXTURE_PREVIEW_OPTIONS, mode: ETexturePreviewMode.SURFACE });
    service.clear();

    expect(service.options).toEqual(DEFAULT_TEXTURE_PREVIEW_OPTIONS);
  });
});
