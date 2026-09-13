import { Box } from "@mui/material";
import { ReactElement } from "react";

import { ApplicationLauncherCard } from "@/core/launcher/components/ApplicationLauncherCard";
import { ICatalogEntry, ICatalogSection } from "@/core/launcher/lib";
import { IApplicationDescriptor } from "@/core/routing/application";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ApplicationLauncherGroupHeading } from "./ApplicationLauncherGroupHeading";

interface IApplicationLauncherCardGridProps extends BaseComponentProps {
  sections: ReadonlyArray<ICatalogSection>;
  onOpen: (application: IApplicationDescriptor) => void;
}

/**
 * The roomy body: one grid of cards per section, under the heading the section carries.
 */
export function ApplicationLauncherCardGrid({
  "data-testid": dataTestId = "application-launcher-card-grid",
  id,
  className,
  sections,
  onOpen,
}: IApplicationLauncherCardGridProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ display: "flex", flexDirection: "column", gap: 4 }}
    >
      {sections.map((section: ICatalogSection) => (
        <Box key={section.group?.id ?? "ranked"} sx={{ display: "flex", flexDirection: "column", gap: 1 }}>
          {section.group ? (
            <ApplicationLauncherGroupHeading group={section.group} count={section.entries.length} />
          ) : null}

          <Box
            sx={{
              display: "grid",
              gridTemplateColumns: {
                xs: "repeat(1, minmax(0, 1fr))",
                sm: "repeat(2, minmax(0, 1fr))",
                md: "repeat(3, minmax(0, 1fr))",
                lg: "repeat(3, minmax(0, 1fr))",
                xl: "repeat(4, minmax(0, 1fr))",
              },
              gap: 1.5,
            }}
          >
            {section.entries.map(({ application, group }: ICatalogEntry) => (
              <ApplicationLauncherCard
                key={application.id}
                application={application}
                group={group}
                isGroupNamed={section.group === null}
                onOpen={onOpen}
              />
            ))}
          </Box>
        </Box>
      ))}
    </Box>
  );
}
