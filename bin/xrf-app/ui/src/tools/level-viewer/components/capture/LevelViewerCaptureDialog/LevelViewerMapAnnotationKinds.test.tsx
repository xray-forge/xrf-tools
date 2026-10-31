import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ELevelObjectKind } from "@xrf/ipc/types/xrf-app-levels";

import { renderWithProviders } from "@/fixtures/utils/render";

import { LevelViewerMapAnnotationKinds } from "./LevelViewerMapAnnotationKinds";

describe("LevelViewerMapAnnotationKinds", () => {
  it("switches a family's kinds together, shown mixed while only some are drawn, and a family of one by its kind", async () => {
    const onChange = jest.fn();
    const { getByRole, queryByRole } = renderWithProviders(
      <LevelViewerMapAnnotationKinds
        kinds={[ELevelObjectKind.SMART_TERRAINS, ELevelObjectKind.GRAPH_POINTS]}
        onChange={onChange}
      />
    );
    const volumes = getByRole("checkbox", { name: "Logic volumes" });

    expect(volumes).toHaveAttribute("data-indeterminate", "true");
    expect(getByRole("checkbox", { name: "Smart terrains" })).toBeChecked();
    expect(getByRole("checkbox", { name: "Graph points" })).toBeChecked();
    expect(queryByRole("checkbox", { name: "Navigation" })).not.toBeInTheDocument();

    await userEvent.click(volumes);

    expect(onChange).toHaveBeenLastCalledWith(
      [
        ELevelObjectKind.SMART_TERRAINS,
        ELevelObjectKind.RESTRICTORS,
        ELevelObjectKind.ANOMALIES,
        ELevelObjectKind.LEVEL_CHANGERS,
        ELevelObjectKind.CLIMBABLES,
      ],
      true
    );

    await userEvent.click(getByRole("checkbox", { name: "Lights" }));

    expect(onChange).toHaveBeenLastCalledWith([ELevelObjectKind.LIGHTS], true);
  });
});
