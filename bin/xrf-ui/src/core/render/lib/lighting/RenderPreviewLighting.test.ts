import { describe, expect, it } from "@jest/globals";
import { AmbientLight, DirectionalLight, Group, Object3D } from "three";

import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { DEFAULT_LIGHT_DISTANCE, RenderPreviewLighting } from "@/core/render/lib/lighting/RenderPreviewLighting";

const LIGHTING: IRenderLighting = {
  ambientColor: 0x223344,
  ambientIntensity: 1.5,
  sunAzimuth: 0,
  sunColor: 0xff8800,
  sunElevation: 90,
  sunIntensity: 2.5,
};

function mockLighting(overrides: Partial<IRenderLighting> = {}): {
  lights: RenderPreviewLighting;
  parent: Object3D;
  sun: DirectionalLight;
  ambient: AmbientLight;
} {
  const parent: Object3D = new Group();
  const lights: RenderPreviewLighting = new RenderPreviewLighting(parent, { ...LIGHTING, ...overrides });

  return {
    lights,
    parent,
    ambient: parent.children.find((it: Object3D) => it instanceof AmbientLight) as AmbientLight,
    sun: parent.children.find((it: Object3D) => it instanceof DirectionalLight) as DirectionalLight,
  };
}

describe("RenderPreviewLighting", () => {
  it("puts its two lights in the scene it was given", () => {
    const { parent, sun, ambient } = mockLighting();

    expect(parent.children).toHaveLength(2);
    expect(sun).toBeDefined();
    expect(ambient).toBeDefined();
  });

  it("is lit as it was told to be from the moment it exists", () => {
    const { sun, ambient } = mockLighting();

    expect(sun.intensity).toBe(2.5);
    expect(sun.color.getHex()).toBe(0xff8800);
    expect(ambient.intensity).toBe(1.5);
    expect(ambient.color.getHex()).toBe(0x223344);
  });

  it("stands the light along the bearing it was given", () => {
    const { sun } = mockLighting({ sunElevation: 90 });

    expect(sun.position.x).toBeCloseTo(0);
    expect(sun.position.y).toBeCloseTo(DEFAULT_LIGHT_DISTANCE);
    expect(sun.position.z).toBeCloseTo(0);
  });

  it("answers the bearing it stands along, at unit length", () => {
    const { lights } = mockLighting({ sunAzimuth: 90, sunElevation: 0 });

    expect(lights.direction.length()).toBeCloseTo(1);
    expect(lights.direction.x).toBeCloseTo(1);
  });

  it("takes a different light without moving anything else", () => {
    const { lights, sun, parent } = mockLighting();

    lights.apply({ ...LIGHTING, sunIntensity: 0.25 });

    expect(sun.intensity).toBe(0.25);
    expect(parent.children).toHaveLength(2);
  });

  // A light standing inside what it lights lights none of it, and a level is larger than any fixed distance.
  it("stands further out for something larger than the distance it starts at", () => {
    const { lights, sun } = mockLighting({ sunElevation: 90 });

    lights.setReach(DEFAULT_LIGHT_DISTANCE * 4);

    expect(sun.position.y).toBeGreaterThan(DEFAULT_LIGHT_DISTANCE);
  });

  it("keeps its distance for something smaller than that", () => {
    const { lights, sun } = mockLighting({ sunElevation: 90 });

    lights.setReach(1);

    expect(sun.position.y).toBeCloseTo(DEFAULT_LIGHT_DISTANCE);
  });

  it("takes both lights back out of the scene", () => {
    const { lights, parent } = mockLighting();

    lights.dispose();

    expect(parent.children).toHaveLength(0);
  });
});
