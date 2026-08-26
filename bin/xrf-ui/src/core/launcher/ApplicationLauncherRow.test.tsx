import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { ApplicationLauncherRow } from "@/core/launcher/ApplicationLauncherRow";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
  IApplicationGroup,
} from "@/core/routing/application";
import { renderWithProviders } from "@/fixtures/utils/render";

function mockApplication(overrides: Partial<IApplicationDescriptor> = {}): IApplicationDescriptor {
  return {
    Component: () => null,
    description: "Browse and edit a packed spawn file",
    group: EApplicationGroupId.SPAWNS,
    icon: <span />,
    id: EApplicationId.SPAWN_EDITOR,
    label: "Spawn editor",
    path: "/spawn-editor",
    status: EApplicationStatus.READY,
    ...overrides,
  };
}

const GROUP: IApplicationGroup = {
  accent: { light: "#677516", dark: "#afcb54" },
  id: EApplicationGroupId.SPAWNS,
  icon: <span />,
  label: "Spawns",
};

describe("ApplicationLauncherRow", () => {
  it("carries the tool and what it does", () => {
    const { getByText } = renderWithProviders(
      <ApplicationLauncherRow application={mockApplication()} group={GROUP} isEnabled onOpen={jest.fn()} />
    );

    expect(getByText("Spawn editor")).toBeInTheDocument();
    expect(getByText("Browse and edit a packed spawn file")).toBeInTheDocument();
  });

  it("says nothing about the group while a separator above the run does", () => {
    const { queryByText } = renderWithProviders(
      <ApplicationLauncherRow application={mockApplication()} group={GROUP} isEnabled onOpen={jest.fn()} />
    );

    expect(queryByText("Spawns")).not.toBeInTheDocument();
  });

  it("names its group where the run is gone, which is every search result", () => {
    const { getByText } = renderWithProviders(
      <ApplicationLauncherRow application={mockApplication()} group={GROUP} isEnabled isGroupNamed onOpen={jest.fn()} />
    );

    expect(getByText("Spawns")).toBeInTheDocument();
  });

  it("is one control rather than a row containing one, so the whole row answers the keyboard", async () => {
    const onOpen = jest.fn();

    const { getByRole } = renderWithProviders(
      <ApplicationLauncherRow application={mockApplication()} group={GROUP} isEnabled onOpen={onOpen} />
    );

    await userEvent.tab();

    expect(getByRole("button", { name: "Spawn editor" })).toHaveFocus();

    await userEvent.keyboard("{Enter}");

    expect(onOpen).toHaveBeenCalledTimes(1);
  });

  it("opens once on click, not once per nested handler", async () => {
    const onOpen = jest.fn();

    const { getByRole } = renderWithProviders(
      <ApplicationLauncherRow application={mockApplication()} group={GROUP} isEnabled onOpen={onOpen} />
    );

    await userEvent.click(getByRole("button", { name: "Spawn editor" }));

    expect(onOpen).toHaveBeenCalledTimes(1);
  });

  it("warms the chunk when the pointer arrives, before any click", async () => {
    const preload = jest.fn(async () => {});

    const { getByRole } = renderWithProviders(
      <ApplicationLauncherRow application={mockApplication({ preload })} group={GROUP} isEnabled onOpen={jest.fn()} />
    );

    await userEvent.hover(getByRole("button", { name: "Spawn editor" }));

    expect(preload).toHaveBeenCalledTimes(1);
  });

  it("keeps a planned tool legible without presenting a disabled control", async () => {
    const preload = jest.fn(async () => {});

    const { findByRole, getByText, queryByRole } = renderWithProviders(
      <ApplicationLauncherRow
        application={mockApplication({ preload, status: EApplicationStatus.PLANNED })}
        group={GROUP}
        isEnabled={false}
        onOpen={jest.fn()}
      />
    );

    await userEvent.hover(getByText("Spawn editor"));

    expect(getByText("Planned")).toBeInTheDocument();
    expect(queryByRole("button")).not.toBeInTheDocument();
    expect(preload).not.toHaveBeenCalled();

    // A row that does nothing has to say why: there is no disabled control here to infer it from.
    expect(await findByRole("tooltip")).toHaveTextContent("Not implemented yet");
  });

  it("still marks a planned tool that developer mode has opened up", () => {
    const { getByRole, getByText } = renderWithProviders(
      <ApplicationLauncherRow
        application={mockApplication({ status: EApplicationStatus.PLANNED })}
        group={GROUP}
        isEnabled
        onOpen={jest.fn()}
      />
    );

    expect(getByRole("button", { name: "Spawn editor" })).toBeInTheDocument();
    expect(getByText("Planned")).toBeInTheDocument();
  });

  it("says nothing about status for a tool that is simply ready", () => {
    const { queryByText } = renderWithProviders(
      <ApplicationLauncherRow application={mockApplication()} group={GROUP} isEnabled onOpen={jest.fn()} />
    );

    expect(queryByText("Planned")).not.toBeInTheDocument();
  });
});
