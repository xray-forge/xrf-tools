import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
  IApplicationGroup,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ApplicationLauncherRow } from "./ApplicationLauncherRow";

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
  it("still opens after hover and focus encounter a failed preload", async () => {
    const load = jest.fn(async () => {
      throw new Error("Runtime unavailable");
    });
    const application = createApplicationDescriptor(mockApplication(), { load });
    const onOpen = jest.fn();
    const { getByRole } = renderWithProviders(
      <ApplicationLauncherRow application={application} group={GROUP} onOpen={onOpen} />
    );

    await userEvent.hover(getByRole("button"));
    await userEvent.tab();
    await userEvent.keyboard("{Enter}");

    expect(load).toHaveBeenCalledTimes(1);
    expect(onOpen).toHaveBeenCalledTimes(1);
    expect(onOpen).toHaveBeenCalledWith(application);
  });

  it("carries the tool and what it does", () => {
    const { getByText } = renderWithProviders(
      <ApplicationLauncherRow application={mockApplication()} group={GROUP} onOpen={jest.fn()} />
    );

    expect(getByText("Spawn editor")).toBeInTheDocument();
    expect(getByText("Browse and edit a packed spawn file")).toBeInTheDocument();
  });

  it("says nothing about the group while a separator above the run does", () => {
    const { queryByText } = renderWithProviders(
      <ApplicationLauncherRow application={mockApplication()} group={GROUP} onOpen={jest.fn()} />
    );

    expect(queryByText("Spawns")).not.toBeInTheDocument();
  });

  it("names its group where the run is gone, which is every search result", () => {
    const { getByText } = renderWithProviders(
      <ApplicationLauncherRow application={mockApplication()} group={GROUP} isGroupNamed onOpen={jest.fn()} />
    );

    expect(getByText("Spawns")).toBeInTheDocument();
  });

  it("is one control rather than a row containing one, so the whole row answers the keyboard", async () => {
    const onOpen = jest.fn();

    const { getByRole } = renderWithProviders(
      <ApplicationLauncherRow application={mockApplication()} group={GROUP} onOpen={onOpen} />
    );

    await userEvent.tab();

    expect(getByRole("button", { name: "Spawn editor" })).toHaveFocus();

    await userEvent.keyboard("{Enter}");

    expect(onOpen).toHaveBeenCalledTimes(1);
  });

  it("opens once on click, not once per nested handler", async () => {
    const onOpen = jest.fn();

    const { getByRole } = renderWithProviders(
      <ApplicationLauncherRow application={mockApplication()} group={GROUP} onOpen={onOpen} />
    );

    await userEvent.click(getByRole("button", { name: "Spawn editor" }));

    expect(onOpen).toHaveBeenCalledTimes(1);
  });

  it("warms the chunk when the pointer arrives, before any click", async () => {
    const preload = jest.fn(async () => {});

    const { getByRole } = renderWithProviders(
      <ApplicationLauncherRow application={mockApplication({ preload })} group={GROUP} onOpen={jest.fn()} />
    );

    await userEvent.hover(getByRole("button", { name: "Spawn editor" }));

    expect(preload).toHaveBeenCalledTimes(1);
  });

  it("opens a planned tool like any other, badged rather than barred", async () => {
    const onOpen = jest.fn();

    const { getByRole, getByText } = renderWithProviders(
      <ApplicationLauncherRow
        application={mockApplication({ status: EApplicationStatus.PLANNED })}
        group={GROUP}
        onOpen={onOpen}
      />
    );

    await userEvent.click(getByRole("button", { name: "Spawn editor" }));

    // The badge says what the tool is; the screen it opens says the rest.
    expect(getByText("Planned")).toBeInTheDocument();
    expect(onOpen).toHaveBeenCalledTimes(1);
  });

  it("says nothing about status for a tool that is simply ready", () => {
    const { queryByText } = renderWithProviders(
      <ApplicationLauncherRow application={mockApplication()} group={GROUP} onOpen={jest.fn()} />
    );

    expect(queryByText("Planned")).not.toBeInTheDocument();
  });
});
