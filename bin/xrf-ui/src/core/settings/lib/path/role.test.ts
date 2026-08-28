import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { exists } from "@tauri-apps/plugin-fs";

import { EApplicationId } from "@/core/routing/application";
import { EPathRole } from "@/core/settings/lib/path/path-role";
import {
  configuredAssetRoots,
  resolveExistingPathRole,
  resolveOutputPath,
  resolvePathRole,
} from "@/core/settings/lib/path/role";
import { createEmptyWorkspacePaths, EWorkspacePath, TWorkspacePaths } from "@/core/settings/lib/workspace-path";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { Nullable } from "@/lib/types/general";

describe("path roles", () => {
  const GAMEDATA: string = "D:\\mods\\my-mod\\gamedata";
  const INSTALLATION: string = "C:\\Games\\stalker";

  function paths(overrides: Partial<TWorkspacePaths> = {}): TWorkspacePaths {
    return { ...createEmptyWorkspacePaths(), ...overrides };
  }

  beforeEach(() => {
    resetMockInvoke();
    jest.mocked(exists).mockResolvedValue(true);
  });

  describe("deriving from game data alone", () => {
    it.each([
      [EPathRole.CONFIGS, `${GAMEDATA}\\configs`],
      [EPathRole.SYSTEM_LTX, `${GAMEDATA}\\configs\\system.ltx`],
      [EPathRole.TRANSLATIONS, `${GAMEDATA}\\configs\\text`],
      [EPathRole.ALL_SPAWN, `${GAMEDATA}\\spawns\\all.spawn`],
      [EPathRole.EQUIPMENT_SPRITE, `${GAMEDATA}\\textures\\ui\\ui_icon_equipment.dds`],
      // Mounted whole, with the editors narrowing to `configs\gameplay` and `configs\text` themselves.
      [EPathRole.CONTENT_ROOT, GAMEDATA],
      [EPathRole.GAMEDATA, GAMEDATA],
      [EPathRole.VISUALS, GAMEDATA],
    ])("answers %s from a loose gamedata root", async (role: EPathRole, expected: string) => {
      await expect(resolvePathRole(role, paths({ [EWorkspacePath.GAMEDATA]: GAMEDATA }))).resolves.toBe(expected);
    });

    it("answers nothing at all when no path is configured", async () => {
      for (const role of Object.values(EPathRole)) {
        await expect(resolvePathRole(role, paths())).resolves.toBeNull();
      }
    });
  });

  describe("overrides", () => {
    it("prefers a configured configs directory over the one game data implies", async () => {
      const configured: TWorkspacePaths = paths({
        [EWorkspacePath.GAMEDATA]: GAMEDATA,
        [EWorkspacePath.CONFIGS]: "C:\\src\\engine\\configs",
      });

      await expect(resolvePathRole(EPathRole.CONFIGS, configured)).resolves.toBe("C:\\src\\engine\\configs");
      await expect(resolvePathRole(EPathRole.SYSTEM_LTX, configured)).resolves.toBe(
        "C:\\src\\engine\\configs\\system.ltx"
      );
    });

    it("mounts the parent of a configured configs directory as the content root", async () => {
      // A source tree keeps translations beside configs rather than beneath them, so the root is one level up in both
      // layouts and the editors' own prefixes do the rest.
      const configured: TWorkspacePaths = paths({ [EWorkspacePath.CONFIGS]: "C:\\src\\engine\\configs" });

      await expect(resolvePathRole(EPathRole.CONTENT_ROOT, configured)).resolves.toBe("C:\\src\\engine");
    });

    it("prefers a configured translations directory over the built one", async () => {
      const configured: TWorkspacePaths = paths({
        [EWorkspacePath.GAMEDATA]: GAMEDATA,
        [EWorkspacePath.TRANSLATIONS]: "C:\\src\\engine\\translations",
      });

      await expect(resolvePathRole(EPathRole.TRANSLATIONS, configured)).resolves.toBe("C:\\src\\engine\\translations");
    });

    it("falls back to game data when an override is absent", async () => {
      const configured: TWorkspacePaths = paths({ [EWorkspacePath.GAMEDATA]: GAMEDATA });

      await expect(resolvePathRole(EPathRole.TRANSLATIONS, configured)).resolves.toBe(`${GAMEDATA}\\configs\\text`);
    });
  });

  describe("chains crossing two configured paths", () => {
    it("reads archives from the installation only, because a loose tree has none", async () => {
      await expect(
        resolvePathRole(EPathRole.ARCHIVES, paths({ [EWorkspacePath.GAMEDATA]: GAMEDATA }))
      ).resolves.toBeNull();

      await expect(
        resolvePathRole(EPathRole.ARCHIVES, paths({ [EWorkspacePath.GAME_INSTALLATION]: INSTALLATION }))
      ).resolves.toBe(INSTALLATION);
    });

    it("reads built translations from the installation first and game data behind it", async () => {
      const both: TWorkspacePaths = paths({
        [EWorkspacePath.GAMEDATA]: GAMEDATA,
        [EWorkspacePath.GAME_INSTALLATION]: INSTALLATION,
      });

      await expect(resolvePathRole(EPathRole.BUILT_TRANSLATIONS, both)).resolves.toBe(INSTALLATION);
      await expect(
        resolvePathRole(EPathRole.BUILT_TRANSLATIONS, paths({ [EWorkspacePath.GAMEDATA]: GAMEDATA }))
      ).resolves.toBe(GAMEDATA);
    });

    it("browses visuals in game data first and the installation behind it", async () => {
      const both: TWorkspacePaths = paths({
        [EWorkspacePath.GAMEDATA]: GAMEDATA,
        [EWorkspacePath.GAME_INSTALLATION]: INSTALLATION,
      });

      await expect(resolvePathRole(EPathRole.VISUALS, both)).resolves.toBe(GAMEDATA);
      await expect(
        resolvePathRole(EPathRole.VISUALS, paths({ [EWorkspacePath.GAME_INSTALLATION]: INSTALLATION }))
      ).resolves.toBe(INSTALLATION);
    });

    it("layers game data in front of the installation for resolving references", async () => {
      expect(
        configuredAssetRoots(
          paths({ [EWorkspacePath.GAMEDATA]: GAMEDATA, [EWorkspacePath.GAME_INSTALLATION]: INSTALLATION })
        )
      ).toEqual([GAMEDATA, INSTALLATION]);
    });
  });

  describe("suggesting only what is there", () => {
    it("keeps a derived path that exists", async () => {
      await expect(
        resolveExistingPathRole(EPathRole.ALL_SPAWN, paths({ [EWorkspacePath.GAMEDATA]: GAMEDATA }))
      ).resolves.toBe(`${GAMEDATA}\\spawns\\all.spawn`);
    });

    it("suggests nothing rather than a path with an error under it", async () => {
      jest.mocked(exists).mockResolvedValue(false);

      await expect(
        resolveExistingPathRole(EPathRole.ALL_SPAWN, paths({ [EWorkspacePath.GAMEDATA]: GAMEDATA }))
      ).resolves.toBeNull();
    });
  });

  describe("output", () => {
    it("gives each application a directory of its own beneath the configured root", async () => {
      const configured: TWorkspacePaths = paths({ [EWorkspacePath.OUTPUT]: "D:\\work\\out" });

      await expect(resolveOutputPath(EApplicationId.ARCHIVES_PACKER, configured)).resolves.toBe(
        "D:\\work\\out\\archives-packer"
      );
      await expect(resolveOutputPath(EApplicationId.SPAWN_PACKER, configured, "all.spawn")).resolves.toBe(
        "D:\\work\\out\\spawn-packer\\all.spawn"
      );
    });

    it("asks the backend where to write when nothing is configured", async () => {
      setMockInvokeResponses({ ["plugin:system|get_default_output_root"]: "C:\\app\\target" });

      await expect(resolveOutputPath(EApplicationId.ARCHIVES_UNPACKER, paths())).resolves.toBe(
        "C:\\app\\target\\archives-unpacker"
      );
    });

    it("suggests nothing when the backend cannot answer", async () => {
      setMockInvokeResponses({
        ["plugin:system|get_default_output_root"]: () => {
          throw new Error("no writable directory");
        },
      });

      const resolved: Nullable<string> = await resolveOutputPath(EApplicationId.ARCHIVES_UNPACKER, paths());

      expect(resolved).toBeNull();
    });
  });
});
