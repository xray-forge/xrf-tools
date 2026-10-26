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

import { ApplicationLauncherCard } from "./ApplicationLauncherCard";

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

describe("ApplicationLauncherCard", () => {
  it("still opens after hover and focus encounter a failed preload", async () => {
    const load = jest.fn(async () => {
      throw new Error("Runtime unavailable");
    });
    const application = createApplicationDescriptor(mockApplication(), { load });
    const onOpen = jest.fn();
    const { getByRole } = renderWithProviders(
      <ApplicationLauncherCard application={application} group={GROUP} onOpen={onOpen} />
    );

    await userEvent.hover(getByRole("button"));
    await userEvent.tab();
    await userEvent.keyboard("{Enter}");

    expect(load).toHaveBeenCalledTimes(1);
    expect(onOpen).toHaveBeenCalledTimes(1);
    expect(onOpen).toHaveBeenCalledWith(application);
  });

  it("warms the chunk when the pointer arrives, before any click", async () => {
    const preload = jest.fn(async () => {});

    const { getByRole } = renderWithProviders(
      <ApplicationLauncherCard application={mockApplication({ preload })} group={GROUP} onOpen={jest.fn()} />
    );

    await userEvent.hover(getByRole("button"));

    // Intent runs ahead of the click, which is the whole point: the fetch is already in flight.
    expect(preload).toHaveBeenCalledTimes(1);
  });

  it("warms on keyboard focus too, so the mouse is not the only way in", async () => {
    const preload = jest.fn(async () => {});

    const { getByRole } = renderWithProviders(
      <ApplicationLauncherCard application={mockApplication({ preload })} group={GROUP} onOpen={jest.fn()} />
    );

    await userEvent.tab();

    expect(getByRole("button")).toHaveFocus();
    expect(preload).toHaveBeenCalled();
  });

  it("opens a planned application like any other, badged rather than barred", async () => {
    const onOpen = jest.fn();

    const { getByRole, getByText } = renderWithProviders(
      <ApplicationLauncherCard
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

  it("survives a statically imported application, which has nothing to warm", async () => {
    const { getByRole } = renderWithProviders(
      <ApplicationLauncherCard application={mockApplication()} group={GROUP} onOpen={jest.fn()} />
    );

    await userEvent.hover(getByRole("button"));

    expect(getByRole("button")).toBeInTheDocument();
  });

  it("still opens on click", async () => {
    const onOpen = jest.fn();

    const { getByRole } = renderWithProviders(
      <ApplicationLauncherCard application={mockApplication()} group={GROUP} onOpen={onOpen} />
    );

    await userEvent.click(getByRole("button"));

    expect(onOpen).toHaveBeenCalledTimes(1);
  });

  it("says nothing about the group while a section heading above it does", () => {
    const { queryByText } = renderWithProviders(
      <ApplicationLauncherCard application={mockApplication()} group={GROUP} onOpen={jest.fn()} />
    );

    expect(queryByText("Spawns")).not.toBeInTheDocument();
  });

  it("names its group where the heading is gone, which is every search result", () => {
    const { getByText } = renderWithProviders(
      <ApplicationLauncherCard application={mockApplication()} group={GROUP} isGroupNamed onOpen={jest.fn()} />
    );

    expect(getByText("Spawns")).toBeInTheDocument();
  });
});
