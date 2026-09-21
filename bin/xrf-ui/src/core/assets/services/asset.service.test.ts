import { beforeEach, describe, expect, it, jest } from "@jest/globals";

import { AssetService } from "./asset.service";

describe("AssetService", () => {
  let revoked: Array<string> = [];

  beforeEach(() => {
    revoked = [];

    jest.spyOn(URL, "revokeObjectURL").mockImplementation((url: string) => {
      revoked.push(url);
    });
  });

  it("releases the previous url for a key when a new one replaces it", () => {
    const service: AssetService = new AssetService();

    const first: string = service.swap("sprite", new Blob());
    const second: string = service.swap("sprite", new Blob());

    expect(revoked).toEqual([first]);
    expect(second).not.toBe(first);
    expect(service.heldCount).toBe(1);
  });

  it("creates the replacement before revoking what it replaces", () => {
    const service: AssetService = new AssetService();

    const created: Array<string> = [];

    jest.spyOn(URL, "createObjectURL").mockImplementation(() => {
      const url: string = `blob:ordered/${created.length}`;

      created.push(url);

      return url;
    });

    service.swap("sprite", new Blob());
    service.swap("sprite", new Blob());

    // Revoking first leaves whatever is rendering the old url pointing at nothing while the
    // replacement is built, and permanently if building it throws.
    expect(created).toHaveLength(2);
    expect(revoked).toEqual([created[0]]);
  });

  it("releases a loose url only once and ignores anything it did not create", () => {
    const service: AssetService = new AssetService();

    const url: string = service.create(new Blob());

    service.release(url);
    service.release(url);
    service.release("blob:not-ours");
    service.release(null);

    expect(revoked).toEqual([url]);
    expect(service.heldCount).toBe(0);
  });

  it("sweeps everything still held when the editor is navigated away from", () => {
    const service: AssetService = new AssetService();

    const keyed: string = service.swap("sprite", new Blob());
    const loose: string = service.create(new Blob());

    expect(service.heldCount).toBe(2);

    service.onDeactivation();

    // Whatever a caller forgot costs one editor session rather than the life of the window.
    expect(revoked).toHaveLength(2);
    expect(revoked).toContain(keyed);
    expect(revoked).toContain(loose);
    expect(service.heldCount).toBe(0);
  });

  it("releases a key on request", () => {
    const service: AssetService = new AssetService();

    const url: string = service.swap("sprite", new Blob());

    service.releaseKey("sprite");
    service.releaseKey("sprite");

    expect(revoked).toEqual([url]);
    expect(service.heldCount).toBe(0);
  });
});
