import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { mockAlphaSurfaceDescriptor, mockSurfaceDescriptor } from "@/fixtures/mocks/visual.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { VisualSubmeshSurface } from "./VisualSubmeshSurface";

function render(surface: XraySurfaceDescriptor | null): RenderResult {
  return renderWithProviders(<VisualSubmeshSurface surface={surface} />);
}

describe("VisualSubmeshSurface", () => {
  it("renders nothing for a submesh that declares no shader", () => {
    const { container } = render(null);

    expect(container).toBeEmptyDOMElement();
  });

  it("says a surface reads no alpha, and does not explain a pass it does not take", () => {
    const { getByText, queryByText } = render(mockSurfaceDescriptor());

    expect(getByText("Opaque")).toBeInTheDocument();
    expect(queryByText("Drawn")).not.toBeInTheDocument();
    expect(getByText(/class 'MODEL'/)).toHaveTextContent(
      "shaders.xr · class 'MODEL' · alpha channel unused · alpha ref 32"
    );
  });

  it("separates the authored reference from the one a cut-out is tested against", () => {
    // The whole point of the row: `models\model_aref` is authored at 128 and the game kills below 200.
    const { getByText } = render(mockAlphaSurfaceDescriptor());

    expect(getByText("Cut out")).toBeInTheDocument();
    expect(getByText(/killed below 200\/255/)).toBeInTheDocument();
    expect(getByText(/alpha ref 128/)).toBeInTheDocument();
  });

  it("says how a blended surface is composited and where the viewer sorts it differently", () => {
    const { getByText } = render(
      mockAlphaSurfaceDescriptor({
        declaration: {
          kind: "described",
          class: "MODEL",
          isAlphaUsed: true,
          alphaReference: 32,
          isStrictSorting: true,
        },
        draw: { kind: "blended", reference: 32 },
      })
    );

    expect(getByText("Blended")).toBeInTheDocument();
    expect(getByText(/depth tested and not written/)).toBeInTheDocument();
    expect(getByText(/strict sorting/)).toBeInTheDocument();
    expect(getByText(/composite in the other order/)).toBeInTheDocument();
  });

  it("tells the four reasons a surface has no blender apart", () => {
    // All four draw opaque, and each is a different fix: ship a library, repair it, define the shader, or accept that
    // this viewer draws that class flat.
    expect(
      render(mockSurfaceDescriptor({ declaration: { kind: "noLibrary" }, library: null })).getByText(
        "No shader library"
      )
    ).toBeInTheDocument();

    expect(
      render(mockSurfaceDescriptor({ declaration: { kind: "unreadable", reason: "chunk 2 is missing" } })).getByText(
        "Library unreadable"
      )
    ).toBeInTheDocument();

    expect(
      render(mockSurfaceDescriptor({ declaration: { kind: "undefined" } })).getByText("Shader undefined")
    ).toBeInTheDocument();

    expect(
      render(mockSurfaceDescriptor({ declaration: { kind: "unmodelled", class: "PARTICLE" } })).getByText(
        "Class not modelled"
      )
    ).toBeInTheDocument();
  });

  it("names the file that could not be read and why", () => {
    const { getByText } = render(
      mockSurfaceDescriptor({ declaration: { kind: "unreadable", reason: "chunk 2 is missing" } })
    );

    expect(getByText(/shaders.xr could not be read: chunk 2 is missing/)).toBeInTheDocument();
  });

  it("omits a knob the class does not write", () => {
    const { getByText } = render(
      mockSurfaceDescriptor({
        declaration: {
          kind: "described",
          class: "MODELEbB",
          isAlphaUsed: true,
          alphaReference: null,
          isStrictSorting: false,
        },
        draw: { kind: "blended", reference: 0 },
      })
    );

    expect(getByText(/class 'MODELEbB'/)).toHaveTextContent("shaders.xr · class 'MODELEbB' · alpha channel used");
  });
});
