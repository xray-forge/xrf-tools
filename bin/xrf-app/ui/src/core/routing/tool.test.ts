import { describe, expect, it } from "@jest/globals";

import { getToolRoute } from "@/core/routing/tool";
import { TOOL_CATALOG } from "@/ToolCatalog";

describe("getToolRoute", () => {
  it("names the route every tool of the catalog answers on", () => {
    for (const tool of TOOL_CATALOG.tools) {
      expect(getToolRoute(tool.id)).toBe(tool.path);
    }
  });
});
