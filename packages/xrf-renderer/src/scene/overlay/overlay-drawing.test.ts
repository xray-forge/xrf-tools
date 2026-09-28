import { describe, expect, it, jest } from "@jest/globals";
import { BufferGeometry, LineBasicNodeMaterial, LineSegments, Scene, Sprite, SpriteNodeMaterial } from "three/webgpu";

import { disposeOverlayObjects } from "#/scene/overlay/overlay-drawing";

describe("disposeOverlayObjects", () => {
  // Three shares one quad between every sprite: disposing it destroyed its buffers under the next sprite put, and the
  // overlay pass drawing that sprite was dropped whole, the grid with it.
  it("leaves the quad every sprite shares, and lets go of the sprite's material", () => {
    const scene: Scene = new Scene();
    const material: SpriteNodeMaterial = new SpriteNodeMaterial();
    const sprite: Sprite = new Sprite(material);
    const onGeometry = jest.fn();
    const onMaterial = jest.fn();

    scene.add(sprite);
    sprite.geometry.addEventListener("dispose", onGeometry);
    material.addEventListener("dispose", onMaterial);

    disposeOverlayObjects([sprite]);

    expect(onGeometry).not.toHaveBeenCalled();
    expect(onMaterial).toHaveBeenCalledTimes(1);
    expect(sprite.parent).toBeNull();
    expect(new Sprite(new SpriteNodeMaterial()).geometry).toBe(sprite.geometry);
  });

  it("lets go of a line overlay's own geometry and material", () => {
    const lines: LineSegments = new LineSegments(new BufferGeometry(), new LineBasicNodeMaterial());
    const onGeometry = jest.fn();

    lines.geometry.addEventListener("dispose", onGeometry);
    disposeOverlayObjects([lines]);

    expect(onGeometry).toHaveBeenCalledTimes(1);
  });
});
