import { TextureCatalogService } from "@/core/textures/services/catalog";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { Nullable } from "@/lib/types/general";

/** Which of the three things the picker is opening. */
export enum ETextureOpenMode {
  /** The game tree, listed by engine reference the way the engine would find it. */
  FOLDER = "folder",
  /** A plain directory, listed by path, for textures that are in no game tree. */
  LOOSE_FOLDER = "looseFolder",
  /** One texture and the descriptor beside it. */
  TEXTURE = "texture",
}

/** What a mode starts its session with. */
export interface ITextureOpenSession {
  catalogService: TextureCatalogService;
  selectionService: TextureSelectionService;
  /**
   * A further tree the session searches behind whatever it opened.
   */
  assetRoot: Nullable<string>;
}

/** Everything about one way in: what it is called, what it reads, and what opening it does. */
export interface ITextureOpenModeDescriptor {
  id: ETextureOpenMode;
  /** What the toggle says. */
  label: string;
  /** What this mode reads, said before it runs rather than after. */
  description: string;
  submitLabel: string;
  /** How the path row beneath the toggle reads. */
  field: { label: string; description: string };
  /**
   * Start the session this mode means.
   *
   * Awaited rather than returned, because a lane-decorated flow is typed as the generator it is written as and only
   * behaves as a promise once the decorator has wrapped it.
   *
   * Every mode starts a session rather than adding to one, so each closes whatever the last left open: a texture from
   * a previous root has nothing to do with the one being opened now.
   */
  open: (path: string, session: ITextureOpenSession) => Promise<void>;
}

/**
 * Every way into the explorer, in the order the toggle offers them.
 *
 * A table rather than a branch per surface, because a mode is six facts - its label, its description, its button, its
 * row, and what it does - and they were spelled in eight places that had to agree. What the toggle offers, what the
 * form says, and what the submit runs all read from here, so a fourth way in is a row rather than an audit.
 */
export const TEXTURE_OPEN_MODES: ReadonlyArray<ITextureOpenModeDescriptor> = [
  {
    description:
      "Lists every texture under the root, archives included, and reads what each descriptor declares. Files " +
      "outside the textures directory are counted rather than listed. Nothing is written.",
    field: { description: "Gamedata directory to browse", label: "Textures root" },
    id: ETextureOpenMode.FOLDER,
    label: "Folder",
    open: async (path: string, { assetRoot, catalogService, selectionService }: ITextureOpenSession): Promise<void> => {
      selectionService.setAssetRoot(assetRoot);

      await catalogService.openRoot(path, assetRoot);
    },
    submitLabel: "Browse",
  },
  {
    description:
      "Lists every dds under the folder by its own path, for textures that are not in a game tree and have no " +
      "engine reference. Descriptors are not swept, because there are no references to sweep them by. Nothing is " +
      "written.",
    field: { description: "Any directory holding dds files, in a game tree or not", label: "Textures folder" },
    id: ETextureOpenMode.LOOSE_FOLDER,
    label: "Loose folder",
    open: async (path: string, { assetRoot, catalogService, selectionService }: ITextureOpenSession): Promise<void> => {
      selectionService.setAssetRoot(assetRoot);

      await catalogService.openLooseDirectory(path);
    },
    submitLabel: "Browse",
  },
  {
    description: "Reads one texture and the descriptor beside it. Nothing is written.",
    field: { description: "Dds texture, or the thm descriptor beside it", label: "Texture file" },
    id: ETextureOpenMode.TEXTURE,
    label: "Texture",
    open: async (path: string, { assetRoot, catalogService, selectionService }: ITextureOpenSession): Promise<void> => {
      await catalogService.close();

      selectionService.setAssetRoot(assetRoot);

      await selectionService.openFile(path);
    },
    submitLabel: "Open",
  },
];

/** Every mode identity, for a remembered choice that has to reject a value the toggle no longer offers. */
export const TEXTURE_OPEN_MODE_IDS: ReadonlyArray<ETextureOpenMode> = TEXTURE_OPEN_MODES.map(
  (it: ITextureOpenModeDescriptor) => it.id
);

/**
 * @param id - The mode to describe.
 * @returns Everything about that way in.
 */
export function getTextureOpenMode(id: ETextureOpenMode): ITextureOpenModeDescriptor {
  return TEXTURE_OPEN_MODES.find((mode: ITextureOpenModeDescriptor) => mode.id === id) ?? TEXTURE_OPEN_MODES[0];
}
