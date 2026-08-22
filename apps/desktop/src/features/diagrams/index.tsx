import type { ReactElement } from "react";
import { Copy, Eye, GitBranch } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Empty } from "@/components/ui/empty";
import { Typography } from "@/components/ui/typography";
import { useAppControllerContext } from "@/providers/AppControllerProvider";

export function DiagramsPage(): ReactElement {
  const controller = useAppControllerContext();

  return (
    <section className="grid gap-6">
      <div className="flex items-center justify-end rounded-none bg-card/85 p-4 backdrop-blur-xl">
        <Button type="button" onClick={controller.generateDiagrams} disabled={controller.status === "working"}>
          <GitBranch size={16} />
          Generate
        </Button>
      </div>

      {controller.diagrams.length === 0 ? (
        <Empty>
          <div>
            <Typography variant="h3">No diagrams generated yet.</Typography>
            <Typography variant="muted">Run Analyze from Scanner or Generate from this page.</Typography>
          </div>
        </Empty>
      ) : null}

      {controller.diagrams.map((diagram) => (
        <Card key={diagram.id}>
          <CardHeader className="flex items-start justify-between gap-3 md:flex-row">
            <div className="min-w-0">
              <CardTitle>{diagram.path}</CardTitle>
              <CardDescription>{diagram.diagramType} · {diagram.format}</CardDescription>
            </div>
            <div className="flex shrink-0 flex-wrap items-center justify-end gap-2.5">
              <Badge tone="brand">{diagram.diagramType}</Badge>
              <Badge>{diagram.format}</Badge>
              <Button aria-label={`Preview ${diagram.path}`} type="button" variant="icon" onClick={() => controller.onPreview({ title: diagram.path, subtitle: `${diagram.diagramType} ${diagram.format}`, content: diagram.content, format: diagram.format })}>
                <Eye size={14} />
              </Button>
              <Button aria-label={`Copy ${diagram.path}`} type="button" variant="icon" onClick={() => controller.copyText("Diagram", diagram.content)}>
                <Copy size={14} />
              </Button>
            </div>
          </CardHeader>
          <CardContent>
            <Typography className="rounded-none bg-muted p-4 text-muted-foreground">
              {diagram.content.slice(0, 320)}
            </Typography>
          </CardContent>
        </Card>
      ))}
    </section>
  );
}
