import { describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { LevelWeatherTexture } from "@/core/ipc/types/xrf-app";
import { LevelWeatherTextureField } from "@/core/level/components/weather/LevelWeatherTextureField";
import { renderWithProviders } from "@/fixtures/utils/render";

function texture(reference: string): LevelWeatherTexture {
  return { texture: { logicalPath: `textures\\${reference}.dds`, reference }, uses: 1 };
}

const SKIES: ReadonlyArray<LevelWeatherTexture> = [
  texture("sky\\af1_foggy\\06-00"),
  texture("sky\\af3_a_clear\\05-30"),
  texture("sky\\af3_a_clear\\06-00"),
];

describe("LevelWeatherTextureField", () => {
  // The value is a reference and the options textures: unmatched, the list marked none and opened at its top.
  it("marks the texture drawn now among those listed", async () => {
    const { getByRole, findAllByRole } = renderWithProviders(
      <LevelWeatherTextureField
        label={"sky_texture"}
        value={"sky\\af3_a_clear\\05-30"}
        textures={SKIES}
        onChange={() => {}}
      />
    );

    await userEvent.click(getByRole("combobox", { name: "sky_texture" }));
    await userEvent.keyboard("{ArrowDown}");

    const selected: Array<HTMLElement> = (await findAllByRole("option")).filter(
      (option: HTMLElement) => option.getAttribute("aria-selected") === "true"
    );

    expect(selected).toHaveLength(1);
    expect(selected[0]).toHaveTextContent("sky\\af3_a_clear\\05-30");
  });
});
