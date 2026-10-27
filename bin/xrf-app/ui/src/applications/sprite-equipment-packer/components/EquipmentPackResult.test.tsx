import { describe, expect, it } from "@jest/globals";
import { EJobOutcome } from "@xrf/ipc/types/xrf-job";

import { mockEquipmentSpritePackResult } from "@/core/sprite-equipment/fixtures/sprite.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { EquipmentPackResult } from "./EquipmentPackResult";

describe("EquipmentPackResult", () => {
  it("counts what a finished pack drew, and the icons it found no image for", () => {
    const { getByTestId } = renderWithProviders(
      <EquipmentPackResult result={mockEquipmentSpritePackResult({ packedCount: 1200, skippedCount: 2 })} />
    );

    expect(getByTestId("equipment-pack-result")).toHaveTextContent("Packed 1,200 icon(s), 2 without an image");
    expect(getByTestId("equipment-pack-result")).toHaveTextContent("1024 × 512sprite");
  });

  it("says a stopped pack wrote nothing rather than counting what it drew", () => {
    const { getByTestId } = renderWithProviders(
      <EquipmentPackResult result={mockEquipmentSpritePackResult({ outcome: EJobOutcome.CANCELLED })} />
    );

    expect(getByTestId("equipment-pack-result")).toHaveTextContent("Stopped before writing");
    expect(getByTestId("equipment-pack-result")).not.toHaveTextContent("packed");
  });
});
