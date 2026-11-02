import { BaseComponentProps } from "@xrf/ui/element-types";
import { List, ListSubheader } from "@xrf/ui/material/list";
import { cn } from "@xrf/ui/theme";
import { Fragment, ReactElement } from "react";

import { ToolLauncherRow } from "@/core/launcher/components/ToolLauncherRow";
import { ICatalogEntry, ICatalogSection } from "@/core/launcher/lib";
import { IToolDescriptor } from "@/core/routing/tool";

import { ToolLauncherGroupHeading } from "./ToolLauncherGroupHeading";

interface IToolLauncherRowListProps extends BaseComponentProps {
  sections: ReadonlyArray<ICatalogSection>;
  onOpen: (tool: IToolDescriptor) => void;
}

/**
 * The dense body: every section in one list, separated by the heading each section carries.
 */
export function ToolLauncherRowList({
  "data-testid": dataTestId = "tool-launcher-row-list",
  id,
  className,
  sections,
  onOpen,
}: IToolLauncherRowListProps): ReactElement {
  return (
    <List data-testid={dataTestId} id={id} className={className} aria-label={"Tools"} disablePadding={true}>
      {sections.map((section: ICatalogSection, sectionIndex: number) => (
        <Fragment key={section.group?.id ?? "ranked"}>
          {section.group ? (
            <ListSubheader
              disableGutters={true}
              disableSticky={true}
              className={cn("bg-transparent px-2 pb-1 leading-[unset]", sectionIndex === 0 ? "pt-0" : "pt-4")}
            >
              <ToolLauncherGroupHeading group={section.group} />
            </ListSubheader>
          ) : null}

          {section.entries.map(({ tool, group }: ICatalogEntry) => (
            <ToolLauncherRow
              key={tool.id}
              tool={tool}
              group={group}
              // A row names its group exactly where no heading above it does.
              isGroupNamed={section.group === null}
              onOpen={onOpen}
            />
          ))}
        </Fragment>
      ))}
    </List>
  );
}
