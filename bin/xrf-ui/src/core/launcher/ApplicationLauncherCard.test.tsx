import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { ApplicationLauncherCard } from "@/core/launcher/ApplicationLauncherCard";
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

describe("ApplicationLauncherCard", () => {
  it("warms the chunk when the pointer arrives, before any click", async () => {
    const preload = jest.fn(async () => {});

    const { getByRole } = renderWithProviders(
      <ApplicationLauncherCard application={mockApplication({ preload })} group={GROUP} isEnabled onOpen={jest.fn()} />
    );

    await userEvent.hover(getByRole("button"));

    // Intent runs ahead of the click, which is the whole point: the fetch is already in flight.
    expect(preload).toHaveBeenCalledTimes(1);
  });

  it("warms on keyboard focus too, so the mouse is not the only way in", async () => {
    const preload = jest.fn(async () => {});

    const { getByRole } = renderWithProviders(
      <ApplicationLauncherCard application={mockApplication({ preload })} group={GROUP} isEnabled onOpen={jest.fn()} />
    );

    await userEvent.tab();

    expect(getByRole("button")).toHaveFocus();
    expect(preload).toHaveBeenCalled();
  });

  it("keeps a planned application legible without presenting a disabled button", async () => {
    const preload = jest.fn(async () => {});

    const { findByRole, getByText, queryByRole } = renderWithProviders(
      <ApplicationLauncherCard
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

    // A card that does nothing has to say why: there is no disabled control here to infer it from.
    expect(await findByRole("tooltip")).toHaveTextContent("Not implemented yet");
  });

  it("survives a statically imported application, which has nothing to warm", async () => {
    const { getByRole } = renderWithProviders(
      <ApplicationLauncherCard application={mockApplication()} group={GROUP} isEnabled onOpen={jest.fn()} />
    );

    await userEvent.hover(getByRole("button"));

    expect(getByRole("button")).toBeInTheDocument();
  });

  it("still opens on click", async () => {
    const onOpen = jest.fn();

    const { getByRole } = renderWithProviders(
      <ApplicationLauncherCard application={mockApplication()} group={GROUP} isEnabled onOpen={onOpen} />
    );

    await userEvent.click(getByRole("button"));

    expect(onOpen).toHaveBeenCalledTimes(1);
  });

  it("says nothing about the group while a section heading above it does", () => {
    const { queryByText } = renderWithProviders(
      <ApplicationLauncherCard application={mockApplication()} group={GROUP} isEnabled onOpen={jest.fn()} />
    );

    expect(queryByText("Spawns")).not.toBeInTheDocument();
  });

  it("names its group where the heading is gone, which is every search result", () => {
    const { getByText } = renderWithProviders(
      <ApplicationLauncherCard
        application={mockApplication()}
        group={GROUP}
        isEnabled
        isGroupNamed
        onOpen={jest.fn()}
      />
    );

    expect(getByText("Spawns")).toBeInTheDocument();
  });
});
