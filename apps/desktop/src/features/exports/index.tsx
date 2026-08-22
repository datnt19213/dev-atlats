import type { ReactElement } from "react";
import { Archive, Copy, FolderOpen } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Typography } from "@/components/ui/typography";
import { useAppControllerContext } from "@/providers/AppControllerProvider";

export function ExportsPage(): ReactElement {
  const controller = useAppControllerContext();

  const isWorking = controller.status === "working";

  return (
    <section className="grid gap-6">
      <div className="grid gap-3 rounded-none bg-card/85 p-4 backdrop-blur-xl md:grid-cols-[minmax(280px,1fr)_auto_auto]">
        <Input
          aria-label="Export output folder"
          placeholder="C:\path\to\output"
          value={controller.outputPath}
          onChange={(event) => controller.setOutputPath(event.target.value)}
        />
        <Button variant="secondary" type="button" onClick={controller.selectOutputFolder} disabled={isWorking}>
          <FolderOpen size={16} />
          Browse
        </Button>
        <Button type="button" onClick={controller.exportKnowledgePackage} disabled={isWorking || controller.outputPath.trim().length === 0}>
          <Archive size={16} />
          Export
        </Button>
      </div>

      <Card>
        <CardHeader>
          <div className="min-w-0">
            <CardTitle>Knowledge Package</CardTitle>
            <CardDescription>{controller.exportedPackage ? "Export completed" : "Waiting for export"}</CardDescription>
          </div>
          {controller.exportedPackage ? (
            <div className="flex shrink-0 items-center gap-2.5">
              <Button aria-label="Copy package path" type="button" variant="icon" onClick={() => controller.copyText("Package path", controller.exportedPackage?.path ?? "")}>
                <Copy size={14} />
              </Button>
            </div>
          ) : null}
        </CardHeader>
        <CardContent>
          <Typography className="break-all rounded-none bg-muted p-4 text-muted-foreground">
            {controller.exportedPackage?.path ?? "No package exported yet."}
          </Typography>
          {controller.exportedPackage ? (
            <dl className="grid gap-4 md:grid-cols-2">
              <div className="rounded-none bg-muted p-4">
                <dt className="text-xs font-semibold leading-5 text-muted-foreground">Artifacts</dt>
                <dd className="text-sm leading-6 text-foreground">{controller.exportedPackage.artifactCount}</dd>
              </div>
              <div className="rounded-none bg-muted p-4">
                <dt className="text-xs font-semibold leading-5 text-muted-foreground">Output Folder</dt>
                <dd className="text-sm leading-6 text-foreground">{controller.exportedPackage.artifactsDir}</dd>
              </div>
            </dl>
          ) : null}
        </CardContent>
      </Card>
    </section>
  );
}
