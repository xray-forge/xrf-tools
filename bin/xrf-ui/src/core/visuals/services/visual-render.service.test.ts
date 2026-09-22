import { beforeAll, describe, expect, it, jest } from "@jest/globals";
import { Container } from "@wirestate/core";
import { makeAutoObservable, runInAction } from "@wirestate/mobx";
import { Texture } from "three";

import { BIND_POSE, IVisualRenderSource, VISUAL_RENDER_SOURCE } from "@/core/visuals/lib/render";
import { VisualViewService } from "@/core/visuals/services/visual-view.service";
import { mockVisualModelViews } from "@/fixtures/mocks/visual.mocks";
import { mockContainer } from "@/fixtures/utils/container";

const scene = {
  applyBump: jest.fn(),
  applyTexture: jest.fn(),
  applyViewOptions: jest.fn(),
  dispose: jest.fn(),
  dolly: jest.fn(),
  resetCamera: jest.fn(),
  setDetailLevel: jest.fn(),
  setFrameRateLimit: jest.fn(),
  setHiddenBones: jest.fn(),
  setHighlightedJoint: jest.fn(),
  setLighting: jest.fn(),
  setModel: jest.fn(),
  setPose: jest.fn(),
};

let VisualRenderService: typeof import("./visual-render.service").VisualRenderService;

beforeAll(async () => {
  // Stubbed at the GPU boundary; jsdom cannot construct a WebGL renderer, and nothing below it is under test.
  jest.doMock("@/core/visuals/lib/scene", () => ({ VisualPreviewScene: jest.fn(() => scene) }));

  ({ VisualRenderService } = await import("./visual-render.service"));
});

function mockSource(overrides: Partial<IVisualRenderSource> = {}): IVisualRenderSource {
  return makeAutoObservable<IVisualRenderSource>(
    { bumps: new Map(), model: null, textures: new Map(), ...overrides },
    {},
    { deep: false }
  );
}

function mockAttached(source: IVisualRenderSource): {
  service: InstanceType<typeof VisualRenderService>;
  viewService: VisualViewService;
} {
  const container: Container = mockContainer([
    VisualViewService,
    VisualRenderService,
    { factory: () => source, token: VISUAL_RENDER_SOURCE },
  ]);
  const service = container.get(VisualRenderService);

  service.attach(document.createElement("div"));

  return { service, viewService: container.get(VisualViewService) };
}

describe("VisualRenderService", () => {
  it("poses the model the way the source says it stands", () => {
    const transforms: Float32Array = new Float32Array(16);
    const source: IVisualRenderSource = mockSource({ pose: BIND_POSE });
    const { service } = mockAttached(source);

    expect(scene.setPose).toHaveBeenCalledWith(null, 0, 0);

    runInAction(() => (source.pose = { floatsPerBone: 16, frame: 4, transforms }));

    expect(scene.setPose).toHaveBeenCalledWith(transforms, 4, 16);

    service.detach();
  });

  // A source that plays nothing leaves the pose out, and a model that is never posed is the bind pose.
  it("stands a model that nothing poses in its bind pose", () => {
    const { service } = mockAttached(mockSource());

    expect(scene.setPose).toHaveBeenCalledWith(null, 0, 0);
    expect(scene.setHiddenBones).toHaveBeenCalledWith(new Set());
    expect(scene.setHighlightedJoint).toHaveBeenCalledWith(null);

    service.detach();
  });

  // A new model rebuilds the meshes, so everything hung on the old ones has to be hung on these.
  it("puts the textures back after the model is replaced", () => {
    const texture: Texture = new Texture();
    const source: IVisualRenderSource = mockSource({ textures: new Map([[0, texture]]) });
    const { service } = mockAttached(source);

    expect(scene.applyTexture).toHaveBeenCalledWith(0, texture);

    scene.applyTexture.mockClear();
    runInAction(() => (source.model = mockVisualModelViews()));

    expect(scene.applyTexture).toHaveBeenCalledWith(0, texture);

    service.detach();
  });

  // The loader publishes geometry and textures in one commit, so the two land in the same mobx batch: the meshes
  // have to be built before anything is hung on them.
  it("dresses a model published together with its textures", () => {
    const texture: Texture = new Texture();
    const model = mockVisualModelViews();
    const source: IVisualRenderSource = mockSource();
    const { service } = mockAttached(source);

    scene.applyTexture.mockClear();
    scene.setModel.mockClear();

    runInAction(() => {
      source.model = model;
      source.textures = new Map([[3, texture]]);
    });

    expect(scene.setModel).toHaveBeenCalledWith(model);
    expect(scene.applyTexture).toHaveBeenCalledWith(3, texture);
    expect(scene.setModel.mock.invocationCallOrder[0]).toBeLessThan(scene.applyTexture.mock.invocationCallOrder[0]);

    service.detach();
  });

  it("releases the scene and stops carrying anything to it", () => {
    const source: IVisualRenderSource = mockSource();
    const { service, viewService } = mockAttached(source);

    service.detach();

    expect(scene.dispose).toHaveBeenCalled();

    scene.setModel.mockClear();
    scene.setLighting.mockClear();

    runInAction(() => (source.model = mockVisualModelViews()));
    viewService.setDetail(0.5);

    expect(scene.setModel).not.toHaveBeenCalled();
    expect(scene.setLighting).not.toHaveBeenCalled();
  });

  it("answers the viewport controls before anything is attached", () => {
    const container: Container = mockContainer([
      VisualViewService,
      VisualRenderService,
      { factory: () => mockSource(), token: VISUAL_RENDER_SOURCE },
    ]);

    expect(() => {
      container.get(VisualRenderService).dolly(2);
      container.get(VisualRenderService).resetCamera();
      container.get(VisualRenderService).detach();
    }).not.toThrow();
  });
});
