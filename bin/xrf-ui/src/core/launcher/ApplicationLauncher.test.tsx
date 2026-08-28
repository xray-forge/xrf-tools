import { beforeEach, describe, expect, it } from "@jest/globals";
import { within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { ApplicationLauncher } from "@/core/launcher/ApplicationLauncher";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
  IApplicationGroup,
} from "@/core/routing/application";
import { renderWithProviders } from "@/fixtures/utils/render";

const APPLICATIONS: ReadonlyArray<IApplicationDescriptor> = [
  {
    Component: () => null,
    description: "Browse packed spawn files",
    group: EApplicationGroupId.SPAWNS,
    icon: <span />,
    id: EApplicationId.SPAWN_EDITOR,
    label: "Spawn editor",
    path: "/spawn-editor",
    status: EApplicationStatus.READY,
  },
  {
    Component: () => null,
    description: "Browse database archives",
    group: EApplicationGroupId.ARCHIVES,
    icon: <span />,
    id: EApplicationId.ARCHIVES_EXPLORER,
    label: "Archives editor",
    path: "/archives-explorer",
    status: EApplicationStatus.READY,
  },
  {
    Component: () => null,
    description: "Pack a directory into game archives",
    group: EApplicationGroupId.ARCHIVES,
    icon: <span />,
    id: EApplicationId.ARCHIVES_PACKER,
    label: "Archives packer",
    path: "/archives-packer",
    status: EApplicationStatus.PLANNED,
  },
];

const GROUPS: ReadonlyArray<IApplicationGroup> = [
  {
    accent: { light: "#000000", dark: "#ffffff" },
    id: EApplicationGroupId.ARCHIVES,
    icon: <span />,
    label: "Archives",
  },
  {
    accent: { light: "#000000", dark: "#ffffff" },
    id: EApplicationGroupId.SPAWNS,
    icon: <span />,
    label: "Spawns",
  },
];

function renderLauncher() {
  return renderWithProviders(<ApplicationLauncher applications={APPLICATIONS} groups={GROUPS} />);
}

/** Tool names in the order the catalog body lists them, ignoring the page's own controls. */
function getToolNames(catalog: HTMLElement): Array<string> {
  return within(catalog)
    .getAllByRole("button")
    .map((button: HTMLElement) => button.getAttribute("aria-label"))
    .filter((label: string | null): label is string => label !== null);
}

describe("ApplicationLauncher", () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  it("packs the caller's catalog in stable group order", () => {
    const { getByTestId, getByText } = renderLauncher();

    expect(getByText("Archives editor")).toBeInTheDocument();
    expect(getByText("Spawn editor")).toBeInTheDocument();
    expect(getToolNames(getByTestId("launcher-catalog"))).toEqual([
      "Archives editor",
      "Archives packer",
      "Spawn editor",
    ]);
  });

  it("calls the listed capabilities tools, which is what the rest of the page has always called them", () => {
    const { getByRole } = renderLauncher();

    expect(getByRole("heading", { level: 1 })).toHaveTextContent("Tools");
  });

  it("counts what is ready separately, since almost half the roster is not", () => {
    const { getByText } = renderLauncher();

    expect(getByText("3 tools · 2 ready · 2 groups")).toBeInTheDocument();
  });

  it("counts the group it was narrowed to rather than the catalog it came from", async () => {
    const { getByRole, getByText } = renderLauncher();

    await userEvent.click(getByRole("button", { name: "Archives 2" }));

    // No group count: with one chosen it could only say "1 group", which the chip already says.
    expect(getByText("2 tools · 1 ready")).toBeInTheDocument();
  });

  it("heads each group's run of cards, so the taxonomy is visible without reading colours", async () => {
    const { getAllByRole, getByRole } = renderLauncher();

    await userEvent.click(getByRole("button", { name: "Grid view" }));

    expect(getAllByRole("heading", { level: 2 }).map((heading: HTMLElement) => heading.textContent)).toEqual([
      "Archives",
      "Spawns",
    ]);
  });

  it("drops the sections while searching and ranks what is left", async () => {
    const { getByLabelText, getByText, queryByRole, queryByText } = renderLauncher();

    await userEvent.type(getByLabelText("Search tools"), "spawn");

    expect(getByText("1 match")).toBeInTheDocument();
    expect(getByText("Spawn editor")).toBeInTheDocument();
    expect(queryByText("Archives editor")).not.toBeInTheDocument();
    expect(queryByRole("heading", { level: 2 })).not.toBeInTheDocument();
  });

  it("names the group on a result card, which no heading is left to say", async () => {
    const { getByLabelText, getByRole, getByTestId } = renderLauncher();

    await userEvent.click(getByRole("button", { name: "Grid view" }));
    await userEvent.type(getByLabelText("Search tools"), "spawn");

    expect(within(getByTestId("launcher-catalog")).getByText("Spawns")).toBeInTheDocument();
  });

  it("matches a group name that no label of its own mentions", async () => {
    const { getByLabelText, getByText, queryByText } = renderLauncher();

    await userEvent.type(getByLabelText("Search tools"), "spawns");

    expect(getByText("Spawn editor")).toBeInTheDocument();
    expect(queryByText("Archives editor")).not.toBeInTheDocument();
  });

  it("says so rather than showing an empty grid when nothing matches", async () => {
    const { getByLabelText, getByText } = renderLauncher();

    await userEvent.type(getByLabelText("Search tools"), "nothing-here");

    expect(getByText("No tools match")).toBeInTheDocument();
    // Quoted: unquoted, a query like "nothing here" reads as part of the sentence carrying it.
    expect(getByText('Nothing in the catalog matches "nothing-here".')).toBeInTheDocument();
  });

  it("offers a way out of a search that found nothing", async () => {
    const { getByLabelText, getByRole, getByText } = renderLauncher();

    await userEvent.type(getByLabelText("Search tools"), "nothing-here");
    await userEvent.click(getByRole("button", { name: "Clear search" }));

    expect(getByText("Spawn editor")).toBeInTheDocument();
    expect(getByLabelText("Search tools")).toHaveValue("");
  });

  it("narrows to one group by chip, and the same chip lets go again", async () => {
    const { getByRole, getByText, queryByText } = renderLauncher();

    await userEvent.click(getByRole("button", { name: "Archives 2" }));

    expect(getByText("Archives editor")).toBeInTheDocument();
    expect(queryByText("Spawn editor")).not.toBeInTheDocument();

    await userEvent.click(getByRole("button", { name: "Archives 2" }));

    expect(getByText("Spawn editor")).toBeInTheDocument();
  });

  it("scopes a search to the chosen group rather than replacing it", async () => {
    const { getByLabelText, getByRole, getByText, queryByText } = renderLauncher();

    await userEvent.click(getByRole("button", { name: "Archives 2" }));
    await userEvent.type(getByLabelText("Search tools"), "editor");

    expect(getByText("Archives editor")).toBeInTheDocument();
    expect(queryByText("Spawn editor")).not.toBeInTheDocument();
  });

  it("focuses the search field from the keyboard, without a pointer", async () => {
    const { getByLabelText } = renderLauncher();

    await userEvent.keyboard("{Control>}k{/Control}");

    expect(getByLabelText("Search tools")).toHaveFocus();
  });

  it("opens on the dense rows, so the whole catalog is readable before anything is chosen", () => {
    const { getByRole } = renderLauncher();

    expect(getByRole("list", { name: "Tools" })).toBeInTheDocument();
  });

  it("takes the view it was left in", () => {
    window.localStorage.setItem("xrf-catalog-view", "grid");

    const { queryByRole } = renderLauncher();

    expect(queryByRole("list", { name: "Tools" })).not.toBeInTheDocument();
  });

  it("falls back to the dense rows for a view it does not recognise", () => {
    window.localStorage.setItem("xrf-catalog-view", "spreadsheet");

    const { getByRole } = renderLauncher();

    expect(getByRole("list", { name: "Tools" })).toBeInTheDocument();
  });

  it("breaks the list where the group changes, rather than repeating it down a column", () => {
    const { getAllByRole } = renderLauncher();

    // The same headings the grid shows, so the taxonomy does not depend on which view is chosen.
    expect(getAllByRole("heading", { level: 2 }).map((heading: HTMLElement) => heading.textContent)).toEqual([
      "Archives",
      "Spawns",
    ]);
  });

  it("names the group on a row only once the separators are gone", async () => {
    const { getByLabelText, getByTestId } = renderLauncher();

    await userEvent.type(getByLabelText("Search tools"), "spawn");

    const catalog = getByTestId("launcher-catalog");

    expect(within(catalog).queryByRole("heading", { level: 2 })).not.toBeInTheDocument();
    expect(within(catalog).getByText("Spawns")).toBeInTheDocument();
  });

  it("keeps drawing rows once a search narrows them", async () => {
    const { getByLabelText, getByRole, getByTestId } = renderLauncher();

    await userEvent.type(getByLabelText("Search tools"), "spawn");

    expect(getByRole("list", { name: "Tools" })).toBeInTheDocument();
    expect(getToolNames(getByTestId("launcher-catalog"))).toEqual(["Spawn editor"]);
  });

  it("swaps the body for the card grid, and remembers being asked", async () => {
    const { getByRole, getByTestId, queryByRole } = renderLauncher();

    await userEvent.click(getByRole("button", { name: "Grid view" }));

    expect(queryByRole("list", { name: "Tools" })).not.toBeInTheDocument();
    expect(window.localStorage.getItem("xrf-catalog-view")).toBe("grid");
    // The same tools in the same catalog order: only their drawing changed.
    expect(getToolNames(getByTestId("launcher-catalog"))).toEqual([
      "Archives editor",
      "Archives packer",
      "Spawn editor",
    ]);
  });

  it("goes back to the rows, leaving nothing of the grid behind", async () => {
    const { getByRole } = renderLauncher();

    await userEvent.click(getByRole("button", { name: "Grid view" }));
    await userEvent.click(getByRole("button", { name: "Row view" }));

    expect(getByRole("list", { name: "Tools" })).toBeInTheDocument();
    expect(window.localStorage.getItem("xrf-catalog-view")).toBe("rows");
  });
});
