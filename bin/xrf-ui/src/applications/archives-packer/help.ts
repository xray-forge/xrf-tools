import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const ARCHIVES_PACKER_HELP: IApplicationHelp = {
  summary:
    "Builds game archive volumes (`.db*` / `.xdb*`) out of a directory, the way `xrCompress` does. What goes " +
    "in is edited here and can be read from or written back to an xrCompress `ltx` configuration; where it " +
    "comes from, where it lands, and how it is written belong to the run rather than to that file.",
  workflow: [
    "Under `Source and output`, pick the directory to pack, the output directory, and the base name of the " +
      "volumes.",
    "Narrow what goes in under `Selection`, or leave it empty to pack the whole source tree.",
    "Check `Header` - the entry point is what decides where the engine mounts the contents - and `Options` " +
      "for compression, volume size, and extension.",
    "`Pack` writes the volumes and reports what it produced. The two icons beside it import and export the " +
      "selection rules as an xrCompress configuration.",
  ],
  nuances: [
    "A configuration file carries only the selection rules and the header. Source, output, name, compression " +
      "mode, volume size, extension, and the skip-list switch are per run and never written to one, which is " +
      "also why changing them never raises `unsaved changes`.",
    "Importing layers over what is open rather than replacing it: a section the file does not carry keeps the " +
      "value it already had.",
    "Selecting nothing packs the whole source directory. An empty selection is not an empty archive - it is " +
      "what `xrCompress` does when handed a directory and no configuration.",
    "The switch means opposite things on the two directory lists: an included directory recurses into its " +
      "children, while an excluded one matches by prefix. A non-recursive exclusion drops only the exact path.",
    "Files under `Included files` bypass every exclusion and the skip list, and a name that does not resolve " +
      "fails the run rather than being passed over. Excluded extension patterns are matched against the " +
      "extension with its dot, case-insensitively, with `*` and `?` as the wildcards.",
    "`Skip editor leftovers` applies the list `xrCompress` hard-codes: `textures\\lod` and `textures\\det`, " +
      "terrain tiles that are not `_mask`, `_nmap` textures, `build.aimap` / `.cform` / `.details` / `.prj`, " +
      "`do_light.ltx`, `.txt` `.tga` `.db` `.smf` `.vcproj` `.sln` `.old` `.rc`, and backup names whose " +
      "extension starts with `~` or `_`.",
    "Only `xml`, `ltx`, and `script` payloads are compressed, and only when compression saves more than 16 " +
      "bytes - the engine expects everything else stored, because models, textures, and sounds already carry " +
      "their own compression. Identical files inside one volume are written once and shared by every name " +
      "pointing at them, which is what `aliased` counts.",
    "A volume is closed once it has passed the size ceiling, so the file that crosses it is still written " +
      "into it: a volume overruns the ceiling by its last payload. A set that ends up with one volume is " +
      "named `<name>.db`, several are `<name>.db0` and up.",
    "Entry names and the header are written as Windows-1251, which is what the engine reads; a name that " +
      "cannot be encoded fails the run.",
    "Without an entry point the engine assumes a `db` archive is an encrypted Shadow of Chernobyl one and " +
      "decrypts it into nonsense. An `xdb` extension is never mistaken that way, which is what the warning on " +
      "`Header` offers as the alternative.",
  ],
  limitations: [
    "Volumes of the same name are overwritten without asking, and nothing else in the output directory is " +
      "cleaned up: a run that produces fewer volumes than the last leaves the extra ones behind, where the " +
      "engine will still mount them.",
    "No progress and no cancellation - packing runs to completion.",
    "The volume ceiling is the engine's own 1900 MB and cannot be raised past it.",
    "A single file larger than 4 GB is more than an archive entry can describe, and fails the run.",
  ],
  relatedTools: [EApplicationId.ARCHIVES_UNPACKER, EApplicationId.ARCHIVES_EXPLORER],
};
