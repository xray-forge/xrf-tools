import { Nullable } from "@xrf/types";
import { useCallback } from "react";

import {
  ARCHIVE_CONFIG_EXPORT_FILTERS,
  ARCHIVE_CONFIG_EXTENSIONS,
  withArchiveConfigExtension,
} from "@/core/archive/lib/config-format";
import { IArchiveConfigEditor } from "@/core/archive/services/archive-config-editor.service";
import { IFileFilter } from "@/core/path/file-filters";
import { askToOpen, askToSave } from "@/core/path/native-dialog";

/**
 * The open dialog's filter: one entry listing every format, so browsing shows all configurations at once.
 *
 * @param noun - What the configuration is called, such as "packing configuration".
 */
function toImportFilters(noun: string): Array<IFileFilter> {
  return [{ name: noun.charAt(0).toUpperCase() + noun.slice(1), extensions: [...ARCHIVE_CONFIG_EXTENSIONS] }];
}

export interface IArchiveConfigFileOptions {
  editor: IArchiveConfigEditor;
  /** What the configuration is called in the dialogs' titles, such as "packing configuration". */
  noun: string;
}

export interface IArchiveConfigFile {
  /** Asks for a configuration file and reads it over the open one; nothing where the dialog is dismissed. */
  askToImport: () => Promise<void>;
  /** Asks where to write the open configuration and writes it there; nothing where the dialog is dismissed. */
  askToExport: () => Promise<void>;
}

/**
 * The native dialogs in front of an archive configuration's import and export, which open at the file last read or
 * written. The dialogs stay outside the editor's busy lane: an unanswered dialog is not configuration I/O.
 */
export function useArchiveConfigFile({ editor, noun }: IArchiveConfigFileOptions): IArchiveConfigFile {
  const askToImport = useCallback(async () => {
    const selected: Nullable<string> = await askToOpen({
      title: `Import ${noun}`,
      start: editor.configPath,
      filters: toImportFilters(noun),
    });

    if (selected) {
      await editor.importConfig(selected);
    }
  }, [editor, noun]);

  const askToExport = useCallback(async () => {
    const selected: Nullable<string> = await askToSave({
      title: `Export ${noun}`,
      start: editor.configPath,
      filters: ARCHIVE_CONFIG_EXPORT_FILTERS,
    });

    if (selected) {
      await editor.exportConfig(withArchiveConfigExtension(selected));
    }
  }, [editor, noun]);

  return { askToImport, askToExport };
}
