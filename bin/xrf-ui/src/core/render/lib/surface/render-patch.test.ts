import { describe, expect, it } from "@jest/globals";
import { MeshStandardMaterial, WebGLProgramParametersWithUniforms } from "three";

import { applyRenderPatch, removeRenderPatch } from "@/core/render/lib/surface/render-patch";

function mockShader(fragmentShader: string = "void main() {}"): WebGLProgramParametersWithUniforms {
  return { fragmentShader, uniforms: {}, vertexShader: "" } as unknown as WebGLProgramParametersWithUniforms;
}

function compile(material: MeshStandardMaterial, shader: WebGLProgramParametersWithUniforms): string {
  material.onBeforeCompile(shader, null as never);

  return shader.fragmentShader;
}

describe("applyRenderPatch", () => {
  it("runs one patch over the shader", () => {
    const material: MeshStandardMaterial = new MeshStandardMaterial();

    applyRenderPatch(material, { name: "first" }, (shader) => {
      shader.fragmentShader = shader.fragmentShader.replace("main", "first");
    });

    expect(compile(material, mockShader())).toBe("void first() {}");
    expect(material.customProgramCacheKey()).toBe("first");
  });

  // The one thing this exists for: three.js documents assigning the hook, and a second assignment would drop the
  // first patch without a word.
  it("keeps every patch a material already carries", () => {
    const material: MeshStandardMaterial = new MeshStandardMaterial();

    applyRenderPatch(material, { name: "first" }, (shader) => {
      shader.fragmentShader += "/first/";
    });
    applyRenderPatch(material, { name: "second" }, (shader) => {
      shader.fragmentShader += "/second/";
    });

    expect(compile(material, mockShader(""))).toBe("/first//second/");
  });

  it("names the patches in the cache key, in the order they were applied", () => {
    const material: MeshStandardMaterial = new MeshStandardMaterial();

    applyRenderPatch(material, { name: "first" }, () => undefined);
    applyRenderPatch(material, { name: "second" }, () => undefined);

    expect(material.customProgramCacheKey()).toBe("first|second");
  });

  it("keys a patch by what it compiled in, where that is not its name", () => {
    const material: MeshStandardMaterial = new MeshStandardMaterial();

    applyRenderPatch(material, { key: "constant:0.5", name: "constant" }, () => undefined);

    expect(material.customProgramCacheKey()).toBe("constant:0.5");
  });

  // A patch closes over what it was built with, so a surface re-dressed from one texture to another would otherwise
  // carry both and compile the older one.
  it("replaces a patch of the same name, in place", () => {
    const material: MeshStandardMaterial = new MeshStandardMaterial();

    applyRenderPatch(material, { name: "first" }, () => undefined);
    applyRenderPatch(material, { name: "second" }, (shader) => {
      shader.fragmentShader += "/second/";
    });
    applyRenderPatch(material, { name: "first" }, (shader) => {
      shader.fragmentShader += "/again/";
    });

    expect(compile(material, mockShader(""))).toBe("/again//second/");
    expect(material.customProgramCacheKey()).toBe("first|second");
  });

  // Three.js's own default is `onBeforeCompile.toString()`, which is one and the same source for every material
  // patched through here: chaining it in would put the wrapper's text in every key and tell nothing apart.
  it("keys a patched material by its patches alone", () => {
    const material: MeshStandardMaterial = new MeshStandardMaterial();

    applyRenderPatch(material, { name: "only" }, () => undefined);

    expect(material.customProgramCacheKey()).not.toContain("shader");
  });

  it("keeps two materials' patches apart", () => {
    const one: MeshStandardMaterial = new MeshStandardMaterial();
    const other: MeshStandardMaterial = new MeshStandardMaterial();

    applyRenderPatch(one, { name: "one" }, () => undefined);
    applyRenderPatch(other, { name: "other" }, () => undefined);

    expect(one.customProgramCacheKey()).toBe("one");
    expect(other.customProgramCacheKey()).toBe("other");
  });

  it("asks for the program to be compiled again", () => {
    const material: MeshStandardMaterial = new MeshStandardMaterial();

    material.needsUpdate = false;
    applyRenderPatch(material, { name: "only" }, () => undefined);

    expect(material.version).toBeGreaterThan(0);
  });
});

describe("removeRenderPatch", () => {
  it("takes one patch off and leaves the rest", () => {
    const material: MeshStandardMaterial = new MeshStandardMaterial();

    applyRenderPatch(material, { name: "kept" }, (shader) => {
      shader.fragmentShader += "/kept/";
    });
    applyRenderPatch(material, { name: "dropped" }, (shader) => {
      shader.fragmentShader += "/dropped/";
    });

    removeRenderPatch(material, "dropped");

    expect(compile(material, mockShader(""))).toBe("/kept/");
    expect(material.customProgramCacheKey()).toBe("kept");
  });

  it("has nothing to say about a patch a material never carried", () => {
    const material: MeshStandardMaterial = new MeshStandardMaterial();

    applyRenderPatch(material, { name: "kept" }, () => undefined);
    removeRenderPatch(material, "never");
    removeRenderPatch(new MeshStandardMaterial(), "never");

    expect(material.customProgramCacheKey()).toBe("kept");
  });
});
