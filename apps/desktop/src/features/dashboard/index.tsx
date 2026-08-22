import type { ReactElement } from "react";
import { BoxSelect, Files, FolderOpen, GitGraph } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Empty } from "@/components/ui/empty";
import { Table } from "@/components/ui/table";
import { Typography } from "@/components/ui/typography";
import { useAppControllerContext } from "@/providers/AppControllerProvider";

export function DashboardPage(): ReactElement {
  const controller = useAppControllerContext();

  const metrics = [
    { label: "Repository", value: controller.repository?.name ?? "Not opened", icon: FolderOpen },
    { label: "Files", value: String(controller.scan?.files ?? 0), icon: Files },
    { label: "Folders", value: String(controller.scan?.folders ?? 0), icon: FolderOpen },
    { label: "Graph Nodes", value: String(controller.graph?.nodeCount ?? 0), icon: GitGraph },
    { label: "Graph Edges", value: String(controller.graph?.edgeCount ?? 0), icon: GitGraph },
    { label: "Documents", value: String(controller.documents.length), icon: BoxSelect },
    { label: "Diagrams", value: String(controller.diagrams.length), icon: BoxSelect },
    { label: "Diagram Links", value: String(controller.packageInsight.diagramRelationshipCount), icon: GitGraph },
  ];

  return (
    <section className="grid gap-6">
      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        {metrics.map((metric) => (
          <MetricCard key={metric.label} label={metric.label} value={metric.value} Icon={metric.icon} />
        ))}
      </div>

      <div className="grid gap-6 xl:grid-cols-3">
        <Card className="xl:col-span-2">
          <CardHeader>
            <CardTitle>Knowledge Package</CardTitle>
            <CardDescription>Generated documentation and diagrams from the current repository context.</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid gap-4 md:grid-cols-3">
              <Insight label="Generated docs" value={String(controller.packageInsight.documentCount)} />
              <Insight label="Generated diagrams" value={String(controller.packageInsight.diagramCount)} />
              <Insight label="Relationships" value={String(controller.packageInsight.diagramRelationshipCount)} />
            </div>
            {controller.packageInsight.generatedDocumentPaths.length > 0 ? (
              <div className="mt-5 grid gap-3 rounded-none bg-muted p-4">
                {controller.packageInsight.generatedDocumentPaths.map((path) => (
                  <span key={path} className="break-all text-sm leading-6 text-muted-foreground">{path}</span>
                ))}
              </div>
            ) : (
              <Empty className="mt-5 min-h-40">No knowledge artifacts generated yet.</Empty>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Detected Stack</CardTitle>
            <CardDescription>Technologies detected during analysis.</CardDescription>
          </CardHeader>
          <CardContent>
            {controller.technologies.length > 0 ? (
              <div className="flex flex-wrap gap-2">
                {controller.technologies.map((technology) => (
                  <Badge tone="brand" key={`${technology.category}-${technology.name}`}>
                    {technology.category}: {technology.name}
                  </Badge>
                ))}
              </div>
            ) : (
              <Empty>No technologies detected yet.</Empty>
            )}
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Repository Context</CardTitle>
          <CardDescription>Current repository summary used by the desktop shell.</CardDescription>
        </CardHeader>
        <CardContent>
          {controller.repositories.length > 0 ? (
            <Table>
              <tbody>
                {controller.repositories.map((repository) => (
                  <tr key={repository.repositoryId} className="align-top">
                    <td className="py-3 pr-6 font-semibold">{repository.name}</td>
                    <td className="py-3 break-all text-muted-foreground">{repository.path}</td>
                  </tr>
                ))}
              </tbody>
            </Table>
          ) : (
            <Empty>No repository opened yet.</Empty>
          )}
        </CardContent>
      </Card>
    </section>
  );
}

function MetricCard(props: {
  label: string;
  value: string;
  Icon: React.ElementType;
}): ReactElement {
  return (
    <Card compact>
      <CardHeader className="mb-3 flex items-center justify-between gap-3">
        <CardDescription>{props.label}</CardDescription>
        <props.Icon size={16} className="text-primary" />
      </CardHeader>
      <Typography variant="h2">{props.value}</Typography>
    </Card>
  );
}

function Insight(props: { label: string; value: string }): ReactElement {
  return (
    <div className="rounded-none bg-muted p-4">
      <Typography variant="caption">{props.label}</Typography>
      <Typography variant="h3">{props.value}</Typography>
    </div>
  );
}
