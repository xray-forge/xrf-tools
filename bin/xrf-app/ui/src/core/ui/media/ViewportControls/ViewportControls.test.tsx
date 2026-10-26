import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { ViewportControls } from "@/core/ui/media/ViewportControls";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("ViewportControls", () => {
  it("offers one reset, whatever the viewport is", () => {
    const onReset = jest.fn();
    const { getByLabelText, queryByLabelText } = renderWithProviders(
      <ViewportControls onZoomIn={jest.fn()} onZoomOut={jest.fn()} onReset={onReset} />
    );

    // The pair this replaced was `Actual size` and `Fit to view`, one of which was a camera control and the other a
    // magnification, drawn identically beside each other.
    expect(queryByLabelText("Fit to view")).not.toBeInTheDocument();
    expect(getByLabelText("Reset view")).toBeInTheDocument();
  });

  it("says nothing about magnification where there is none to report", () => {
    const { queryByLabelText, getByLabelText } = renderWithProviders(
      <ViewportControls onZoomIn={jest.fn()} onZoomOut={jest.fn()} onReset={jest.fn()} />
    );

    // A share of the distance to what a camera orbits cannot see an orbit or a pan, so a scene reports no number at
    // all rather than one that reads unchanged from a view nobody would recognise.
    expect(queryByLabelText("Actual size")).not.toBeInTheDocument();
    expect(getByLabelText("Zoom in")).toBeInTheDocument();
    expect(getByLabelText("Zoom out")).toBeInTheDocument();
  });

  it("makes the magnification it reports the way back to one to one", async () => {
    const onActualSize = jest.fn();
    const { getByLabelText } = renderWithProviders(
      <ViewportControls
        zoom={{ onActualSize, scale: 12.0649 }}
        onZoomIn={jest.fn()}
        onZoomOut={jest.fn()}
        onReset={jest.fn()}
      />
    );
    const readout: HTMLElement = getByLabelText("Actual size");

    expect(readout).toHaveTextContent("1206%");

    await userEvent.click(readout);

    expect(onActualSize).toHaveBeenCalledTimes(1);
  });
});
