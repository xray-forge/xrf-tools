import { describe, expect, it } from "@jest/globals";
import { render } from "@testing-library/react";
import { ERenderAntialiasing, ERenderBackend, RenderFrameReport } from "@xrf/ipc/types/xrf-renderer";

import { RenderFrameReadout } from "@/core/render/components/overlay/RenderFrameReadout";
import { EMPTY_RENDER_FRAME_REPORT, EMPTY_RENDER_STATIC_REPORT } from "@/core/render/lib/native/native-frame-report";

const REPORT: RenderFrameReport = {
  ...EMPTY_RENDER_FRAME_REPORT,
  adapter: "Test GPU",
  frameTime: 6.25,
  framesPerSecond: 160,
  height: 1930,
  outputHeight: 1930,
  outputWidth: 3217,
  renderHeight: 1930,
  renderWidth: 3217,
  staticDraws: { ...EMPTY_RENDER_STATIC_REPORT, commands: 886, keptTriangles: 6445460 },
  width: 3217,
};

describe("RenderFrameReadout", () => {
  it("reads out what the frame cost and the size it was paid over", () => {
    const { getByText } = render(<RenderFrameReadout report={REPORT} />);

    expect(getByText("160 fps · 6.3 ms")).toBeInTheDocument();
    expect(getByText("dx · 6,445,460 tris")).toBeInTheDocument();
  });

  it("tags the graphics API the frame was drawn with", () => {
    const { getByText } = render(<RenderFrameReadout report={{ ...REPORT, backend: ERenderBackend.VULKAN }} />);

    expect(getByText("vk · 6,445,460 tris")).toBeInTheDocument();
  });

  it("names no graphics API before a frame says which GPU it was drawn on", () => {
    const { getByText } = render(<RenderFrameReadout report={{ ...REPORT, adapter: "" }} />);

    expect(getByText("6,445,460 tris")).toBeInTheDocument();
  });

  it("says the size the frame was drawn at, and no scaling at the viewport's own size", () => {
    const { getByText, queryByText } = render(<RenderFrameReadout report={REPORT} />);

    expect(getByText("3217 × 1930")).toBeInTheDocument();
    expect(queryByText(/→/)).toBeNull();
  });

  it("says what upscaled the scene from which size to which: the temporal resolve to the output", () => {
    const { getByText } = render(
      <RenderFrameReadout
        report={{
          ...REPORT,
          antialiasing: ERenderAntialiasing.DLSS,
          outputHeight: 1440,
          outputWidth: 2400,
          renderHeight: 720,
          renderWidth: 1200,
        }}
      />
    );

    expect(getByText("DLSS 1200 × 720 → 2400 × 1440")).toBeInTheDocument();
  });

  it("names DLSS ray reconstruction where it upscaled the scene", () => {
    const { getByText } = render(
      <RenderFrameReadout
        report={{
          ...REPORT,
          antialiasing: ERenderAntialiasing.DLSS,
          isRayReconstructed: true,
          outputHeight: 1440,
          outputWidth: 2400,
          renderHeight: 720,
          renderWidth: 1200,
        }}
      />
    );

    expect(getByText("DLSS RR 1200 × 720 → 2400 × 1440")).toBeInTheDocument();
  });

  it("leaves the target out where the temporal resolve upscales to the viewport's own size", () => {
    const { getByText, queryByText } = render(
      <RenderFrameReadout
        report={{
          ...REPORT,
          antialiasing: ERenderAntialiasing.DLSS,
          outputHeight: 1930,
          outputWidth: 3217,
          renderHeight: 1286,
          renderWidth: 2144,
        }}
      />
    );

    expect(getByText("DLSS 2144 × 1286")).toBeInTheDocument();
    expect(queryByText(/→/)).toBeNull();
  });

  it("says a temporal resolve drawing at the output's size, larger than the viewport, by its size alone", () => {
    const { getByText } = render(
      <RenderFrameReadout
        report={{
          ...REPORT,
          antialiasing: ERenderAntialiasing.TAA,
          outputHeight: 3860,
          outputWidth: 6434,
          renderHeight: 3860,
          renderWidth: 6434,
        }}
      />
    );

    expect(getByText("TAA 6434 × 3860")).toBeInTheDocument();
  });

  it("names FSR 1 where no temporal resolve upscales the scene, to the viewport it leaves unsaid", () => {
    const { getByText } = render(
      <RenderFrameReadout
        report={{ ...REPORT, antialiasing: ERenderAntialiasing.SMAA, renderHeight: 965, renderWidth: 1609 }}
      />
    );

    expect(getByText("FSR 1 1609 × 965")).toBeInTheDocument();
  });

  it("carries what only the scene can say, under the rest", () => {
    const { getByText } = render(
      <RenderFrameReadout report={REPORT}>
        <div>118 sectors · 138 MB</div>
      </RenderFrameReadout>
    );

    expect(getByText("118 sectors · 138 MB")).toBeInTheDocument();
  });

  it("reads the spot meter out: its stops as exposed, the light and the exposure, then the surface under it", () => {
    const { getByText } = render(
      <RenderFrameReadout
        report={{
          ...REPORT,
          meter: {
            albedo: 0.18116,
            diffuse: 0.6471,
            distance: 2.788,
            ev100: null,
            exposure: 1.3404,
            gloss: 0.00784,
            luminance: 0.31359,
            specular: 0.0213,
            stops: 1.2235,
          },
        }}
      />
    );

    expect(getByText("Meter +1.22 EV · L 0.314 · ×1.34")).toBeInTheDocument();
    expect(getByText("albedo 0.181 · gloss 0.01 · 2.8 m")).toBeInTheDocument();
    expect(getByText("light 0.647 · specular 0.0213")).toBeInTheDocument();
  });

  it("reads the spot meter over the sky by its light alone", () => {
    const { getByText, queryByText } = render(
      <RenderFrameReadout
        report={{
          ...REPORT,
          meter: {
            albedo: null,
            diffuse: null,
            distance: null,
            ev100: null,
            exposure: 2,
            gloss: null,
            luminance: 0.02,
            specular: null,
            stops: -2.17,
          },
        }}
      />
    );

    expect(getByText("Meter -2.17 EV · L 0.0200 · ×2.00")).toBeInTheDocument();
    expect(queryByText(/albedo/)).toBeNull();
  });

  it("reads the physical light's spot in cd/m² and its exposure value at ISO 100", () => {
    const { getByText } = render(
      <RenderFrameReadout
        report={{
          ...REPORT,
          meter: {
            albedo: null,
            diffuse: null,
            distance: null,
            ev100: 13.6,
            exposure: 0.00014,
            gloss: null,
            luminance: 1250,
            specular: null,
            stops: 0.4,
          },
        }}
      />
    );

    expect(getByText("Meter +0.40 EV · L 1250 cd/m² · EV100 13.6")).toBeInTheDocument();
  });

  it("reads out nothing drawn rather than nothing at all", () => {
    const { getByText } = render(<RenderFrameReadout report={{ ...EMPTY_RENDER_FRAME_REPORT, frameTime: null }} />);

    expect(getByText("0 fps · 0.0 ms")).toBeInTheDocument();
  });

  it("lists every pass's GPU cost under the frame while passes are timed, and nothing while they are not", () => {
    const passes = [
      { gpuTime: 0.5, name: "gbuffer" },
      { gpuTime: 0.25, name: "antialias" },
    ];
    const timed = render(<RenderFrameReadout report={{ ...EMPTY_RENDER_FRAME_REPORT, isGpuTimed: true, passes }} />);

    expect(timed.getByTestId("render-frame-passes")).toHaveTextContent("GPU0.75 msgbuffer0.50antialias0.25");
    timed.unmount();

    const untimed = render(<RenderFrameReadout report={{ ...EMPTY_RENDER_FRAME_REPORT, passes }} />);

    expect(untimed.queryByTestId("render-frame-passes")).not.toBeInTheDocument();
  });

  it("sums a run of a family's passes into one row under the words they share", () => {
    const passes = [
      { gpuTime: 0.5, name: "g-buffer" },
      { gpuTime: 0.25, name: "reflection trace" },
      { gpuTime: 0.25, name: "reflection scene" },
      { gpuTime: 0.5, name: "reflection resolve" },
      { gpuTime: 0.125, name: "sky prefilter 1" },
      { gpuTime: 0.125, name: "sky prefilter 2" },
      { gpuTime: 0.25, name: "sky prefilter 3" },
      { gpuTime: 0.5, name: "sun shadow 0" },
      { gpuTime: 0.25, name: "sun shadow 1" },
    ];
    const { getByTestId } = render(
      <RenderFrameReadout report={{ ...EMPTY_RENDER_FRAME_REPORT, isGpuTimed: true, passes }} />
    );

    expect(getByTestId("render-frame-passes")).toHaveTextContent(
      "GPU2.75 msg-buffer0.50reflection ×31.00sky prefilter ×30.50sun shadow 00.50sun shadow 10.25"
    );
  });
});
