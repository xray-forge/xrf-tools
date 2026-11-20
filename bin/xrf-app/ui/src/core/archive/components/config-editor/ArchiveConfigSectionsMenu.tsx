import { EditorSideMenu, IEditorSideMenuItem } from "@xrf/ui/editor/side-menu";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { IArchiveConfigEditor } from "@/core/archive/services/archive-config-editor.service";
import { EditorPanelHeader } from "@/core/shell/chrome/EditorPanelHeader";

/** One section of an archive configuration, as its navigation lists it. */
export interface IArchiveConfigSection<TSection extends string> {
  id: TSection;
  label: string;
  description: string;
  icon: ReactElement;
}

/** Named once, because the panel and its own heading must not drift apart. */
export const ARCHIVE_CONFIG_SECTIONS_LABEL: string = "Configuration";

interface IArchiveConfigSectionsMenuProps<TSection extends string> extends BaseComponentProps {
  editor: IArchiveConfigEditor<TSection>;
  /** The sections, in the order they are edited. */
  sections: ReadonlyArray<IArchiveConfigSection<TSection>>;
}

/**
 * Navigation for an archive configuration, drawn by the shell as the tool's left panel.
 */
export function ArchiveConfigSectionsMenu<TSection extends string>({
  "data-testid": dataTestId = "archive-config-sections-menu",
  id,
  className,
  editor,
  sections,
}: IArchiveConfigSectionsMenuProps<TSection>): ReactElement {
  const items: Array<IEditorSideMenuItem> = sections.map((section: IArchiveConfigSection<TSection>) => ({
    label: section.label,
    description: section.description,
    icon: section.icon,
    isSelected: editor.section === section.id,
    onClick: () => editor.openSection(section.id),
  }));

  return (
    <EditorSideMenu
      data-testid={dataTestId}
      id={id}
      className={className}
      header={<EditorPanelHeader title={ARCHIVE_CONFIG_SECTIONS_LABEL} />}
      sections={items}
    />
  );
}
