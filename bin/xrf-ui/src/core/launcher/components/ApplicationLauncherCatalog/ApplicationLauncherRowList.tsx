import { List, ListSubheader } from "@mui/material";
import { Fragment, ReactElement } from "react";

import { ApplicationLauncherRow } from "@/core/launcher/components/ApplicationLauncherRow";
import { ICatalogEntry, ICatalogSection } from "@/core/launcher/lib";
import { IApplicationDescriptor } from "@/core/routing/application";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ApplicationLauncherGroupHeading } from "./ApplicationLauncherGroupHeading";

interface IApplicationLauncherRowListProps extends BaseComponentProps {
  sections: ReadonlyArray<ICatalogSection>;
  onOpen: (application: IApplicationDescriptor) => void;
}

/**
 * The dense body: every section in one list, separated by the heading each section carries.
 */
export function ApplicationLauncherRowList({
  "data-testid": dataTestId = "application-launcher-row-list",
  id,
  className,
  sections,
  onOpen,
}: IApplicationLauncherRowListProps): ReactElement {
  return (
    <List data-testid={dataTestId} id={id} className={className} aria-label={"Tools"} disablePadding={true}>
      {sections.map((section: ICatalogSection, sectionIndex: number) => (
        <Fragment key={section.group?.id ?? "ranked"}>
          {section.group ? (
            <ListSubheader
              disableGutters={true}
              disableSticky={true}
              sx={{
                backgroundColor: "transparent",
                lineHeight: "unset",
                paddingX: 1,
                paddingTop: sectionIndex === 0 ? 0 : 2,
                paddingBottom: 0.5,
              }}
            >
              <ApplicationLauncherGroupHeading group={section.group} count={section.entries.length} />
            </ListSubheader>
          ) : null}

          {section.entries.map(({ application, group }: ICatalogEntry) => (
            <ApplicationLauncherRow
              key={application.id}
              application={application}
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
