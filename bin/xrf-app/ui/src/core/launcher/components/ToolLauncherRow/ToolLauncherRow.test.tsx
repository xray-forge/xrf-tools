import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor, IToolGroup } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ToolLauncherRow } from "./ToolLauncherRow";

function mockTool(overrides: Partial<IToolDescriptor> = {}): IToolDescriptor {
  return {
    Component: () => null,
    description: "Browse and edit a packed spawn file",
    group: EToolGroupId.SPAWNS,
    icon: <span />,
    id: EToolId.SPAWN_EDITOR,
    label: "Spawn editor",
    path: "/spawn-editor",
    status: EToolStatus.READY,
    ...overrides,
  };
}

const GROUP: IToolGroup = {
  accent: { light: "#677516", dark: "#afcb54" },
  id: EToolGroupId.SPAWNS,
  icon: <span />,
  label: "Spawns",
};

describe("ToolLauncherRow", () => {
  it("still opens after hover and focus encounter a failed preload", async () => {
    const load = jest.fn(async () => {
      throw new Error("Runtime unavailable");
    });
    const tool = createToolDescriptor(mockTool(), { load });
    const onOpen = jest.fn();
    const { getByRole } = renderWithProviders(<ToolLauncherRow tool={tool} group={GROUP} onOpen={onOpen} />);

    await userEvent.hover(getByRole("button"));
    await userEvent.tab();
    await userEvent.keyboard("{Enter}");

    expect(load).toHaveBeenCalledTimes(1);
    expect(onOpen).toHaveBeenCalledTimes(1);
    expect(onOpen).toHaveBeenCalledWith(tool);
  });

  it("carries the tool and what it does", () => {
    const { getByText } = renderWithProviders(<ToolLauncherRow tool={mockTool()} group={GROUP} onOpen={jest.fn()} />);

    expect(getByText("Spawn editor")).toBeInTheDocument();
    expect(getByText("Browse and edit a packed spawn file")).toBeInTheDocument();
  });

  it("says nothing about the group while a separator above the run does", () => {
    const { queryByText } = renderWithProviders(<ToolLauncherRow tool={mockTool()} group={GROUP} onOpen={jest.fn()} />);

    expect(queryByText("Spawns")).not.toBeInTheDocument();
  });

  it("names its group where the run is gone, which is every search result", () => {
    const { getByText } = renderWithProviders(
      <ToolLauncherRow tool={mockTool()} group={GROUP} isGroupNamed onOpen={jest.fn()} />
    );

    expect(getByText("Spawns")).toBeInTheDocument();
  });

  it("is one control rather than a row containing one, so the whole row answers the keyboard", async () => {
    const onOpen = jest.fn();

    const { getByRole } = renderWithProviders(<ToolLauncherRow tool={mockTool()} group={GROUP} onOpen={onOpen} />);

    await userEvent.tab();

    expect(getByRole("button", { name: "Spawn editor" })).toHaveFocus();

    await userEvent.keyboard("{Enter}");

    expect(onOpen).toHaveBeenCalledTimes(1);
  });

  it("opens once on click, not once per nested handler", async () => {
    const onOpen = jest.fn();

    const { getByRole } = renderWithProviders(<ToolLauncherRow tool={mockTool()} group={GROUP} onOpen={onOpen} />);

    await userEvent.click(getByRole("button", { name: "Spawn editor" }));

    expect(onOpen).toHaveBeenCalledTimes(1);
  });

  it("warms the chunk when the pointer arrives, before any click", async () => {
    const preload = jest.fn(async () => {});

    const { getByRole } = renderWithProviders(
      <ToolLauncherRow tool={mockTool({ preload })} group={GROUP} onOpen={jest.fn()} />
    );

    await userEvent.hover(getByRole("button", { name: "Spawn editor" }));

    expect(preload).toHaveBeenCalledTimes(1);
  });

  it("opens a planned tool like any other, badged rather than barred", async () => {
    const onOpen = jest.fn();

    const { getByRole, getByText } = renderWithProviders(
      <ToolLauncherRow tool={mockTool({ status: EToolStatus.PLANNED })} group={GROUP} onOpen={onOpen} />
    );

    await userEvent.click(getByRole("button", { name: "Spawn editor" }));

    // The badge says what the tool is; the screen it opens says the rest.
    expect(getByText("Planned")).toBeInTheDocument();
    expect(onOpen).toHaveBeenCalledTimes(1);
  });

  it("says nothing about status for a tool that is simply ready", () => {
    const { queryByText } = renderWithProviders(<ToolLauncherRow tool={mockTool()} group={GROUP} onOpen={jest.fn()} />);

    expect(queryByText("Planned")).not.toBeInTheDocument();
  });
});
