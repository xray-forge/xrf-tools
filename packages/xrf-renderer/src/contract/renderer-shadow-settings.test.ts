import { describe, expect, it } from "@jest/globals";

import { RENDERER_FEATURE_SCHEMA } from "#/contract/renderer-feature-schema";
import { RENDERER_MAX_SHADOW_CASCADES, RENDERER_SHADOW_CASCADE_WIDTHS } from "#/contract/renderer-shadow-settings";

describe("renderer shadow settings", () => {
  it("offers no more cascades than the sun can sample, and takes no more", () => {
    expect(RENDERER_SHADOW_CASCADE_WIDTHS.length).toBeLessThanOrEqual(RENDERER_MAX_SHADOW_CASCADES);
    expect(RENDERER_FEATURE_SCHEMA.shadows.cascades.most).toBe(RENDERER_MAX_SHADOW_CASCADES);
  });
});
