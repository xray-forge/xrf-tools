import { describe, expect, it } from "@jest/globals";
import { ESceneLightType, ESceneShapeElement, ESceneShapeUsage, ESceneWayType } from "@xrf/ipc/types/xrf-app-levels";

import {
  describeSceneLightType,
  describeSceneShape,
  describeSceneShapeUsage,
  describeSceneWayType,
  formatSceneList,
  formatSceneMetres,
  formatSceneRotation,
  formatSceneScale,
} from "@/tools/level-editor/lib/inspector/scene-object-format";

describe("formatSceneList", () => {
  it("writes each entry a line", () => {
    expect(formatSceneList(["a", "b"], (it: string, index: number) => `${index}: ${it}`)).toBe("0: a\n1: b");
  });

  it("counts what lies past the limit instead of writing it, describing only what it writes", () => {
    const described: Array<number> = [];
    const items: Array<number> = Array.from({ length: 3000 }, (_, index: number) => index);

    const text: string = formatSceneList(
      items,
      (it: number) => {
        described.push(it);

        return String(it);
      },
      2
    );

    expect(text).toBe("0\n1\nand 2,998 more");
    expect(described).toEqual([0, 1]);
  });

  it("says an empty list holds nothing", () => {
    expect(formatSceneList([], String)).toBe("None");
  });
});

describe("scene object values", () => {
  it("writes angles in degrees, scales and lengths in two decimals, and an absent length as absent", () => {
    expect(formatSceneRotation({ x: 0, y: Math.PI / 2, z: null })).toBe("x 0.0° y 90.0° z 0.0°");
    expect(formatSceneScale({ x: 1, y: 0.5, z: null })).toBe("x 1.00 y 0.50 z —");
    expect(formatSceneMetres(2.5)).toBe("2.50 m");
    expect(formatSceneMetres(null)).toBe("—");
  });

  it("describes a shape's spheres and boxes by their size and place", () => {
    expect(
      describeSceneShape({
        elements: [
          { center: { x: 1, y: 2, z: 3 }, kind: ESceneShapeElement.SPHERE, radius: 1.5 },
          {
            kind: ESceneShapeElement.BOX,
            translation: { x: 0, y: 0, z: 0 },
            xAxis: { x: 2, y: 0, z: 0 },
            yAxis: { x: 0, y: 3, z: 0 },
            zAxis: { x: 0, y: 0, z: 4 },
          },
        ],
        usage: { kind: ESceneShapeUsage.COMMON },
      })
    ).toBe("Sphere, 1.50 m radius at x 1.0 y 2.0 z 3.0\nBox, 2.00 m × 3.00 m × 4.00 m at x 0.0 y 0.0 z 0.0");
  });

  it("names a light's type, a shape's usage and a way's type, and one the editor gives no name by its number", () => {
    expect(describeSceneLightType({ kind: ESceneLightType.SPOT })).toBe("Spot");
    expect(describeSceneLightType({ kind: ESceneLightType.UNKNOWN, value: 7 })).toBe("Type 7");
    expect(describeSceneShapeUsage({ kind: ESceneShapeUsage.LEVEL_BOUND })).toBe("Level bound");
    expect(describeSceneShapeUsage({ kind: ESceneShapeUsage.UNKNOWN, value: 9 })).toBe("Type 9");
    expect(describeSceneWayType({ kind: ESceneWayType.PATROL_PATH })).toBe("Patrol path");
    expect(describeSceneWayType({ kind: ESceneWayType.UNKNOWN, value: 4 })).toBe("Type 4");
  });
});
