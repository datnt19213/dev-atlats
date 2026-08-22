import type { ReactElement } from "react";
import { BookOpen, Copy, Eye, ShieldAlert } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Empty } from "@/components/ui/empty";
import { Typography } from "@/components/ui/typography";
import { useAppControllerContext } from "@/providers/AppControllerProvider";

export function DocumentationPage(): ReactElement {
  const controller = useAppControllerContext();

  return (
    <section className="grid gap-6">
      <div className="flex items-center justify-end rounded-none bg-card/85 p-4 backdrop-blur-xl">
        <Button type="button" onClick={controller.generateDocuments} disabled={controller.status === "working"}>
          <BookOpen size={16} />
          Generate semantic docs
        </Button>
      </div>

      {controller.documents.length === 0 ? (
        <Empty className="min-h-56">
          <BookOpen size={24} />
          <div>
            <Typography variant="h3">No documentation generated yet.</Typography>
            <Typography variant="muted">Generate docs after opening and analyzing a repository.</Typography>
          </div>
        </Empty>
      ) : null}

      {controller.documents.map((document) => (
        <Card key={document.id}>
          <CardHeader className="flex items-start justify-between gap-3 md:flex-row">
            <div className="min-w-0">
              <CardTitle>{document.path}</CardTitle>
              <CardDescription>{document.documentType} · {document.content.length} characters</CardDescription>
            </div>
            <div className="flex shrink-0 items-center gap-2.5">
              <Badge variant={document.quality.semanticScore >= 70 ? "default" : "secondary"}>{document.quality.semanticScore}% semantic</Badge>
              <Badge variant={document.quality.coverageScore >= 70 ? "default" : "secondary"}>{document.quality.coverageScore}% coverage</Badge>
              <Button aria-label={`Preview ${document.path}`} type="button" variant="icon" onClick={() => controller.onPreview({ title: document.path, subtitle: "Generated documentation", content: document.content })}>
                <Eye size={14} />
              </Button>
              <Button aria-label={`Copy ${document.path}`} type="button" variant="icon" onClick={() => controller.copyText("Document", document.content)}>
                <Copy size={14} />
              </Button>
            </div>
          </CardHeader>
          <CardContent className="grid gap-5">
            <QualitySummary document={document} />
            <EvidencePlan document={document} />
            <Typography className="rounded-none bg-muted p-4 text-muted-foreground">
              {document.content.slice(0, 520)}
            </Typography>
          </CardContent>
        </Card>
      ))}
    </section>
  );
}

function QualitySummary(props: { document: { quality: { coverageScore: number; semanticScore: number; sourceCount: number; symbolCount: number; graphEdgeCount: number; warnings: string[] } } }): ReactElement {
  return (
    <div className="grid gap-3 rounded-none border bg-muted/40 p-4">
      <div className="grid gap-2 md:grid-cols-5">
        <QualityMetric label="Coverage" value={props.document.quality.coverageScore} />
        <QualityMetric label="Semantic" value={props.document.quality.semanticScore} />
        <QualityMetric label="Sources" value={props.document.quality.sourceCount} />
        <QualityMetric label="Symbols" value={props.document.quality.symbolCount} />
        <QualityMetric label="Edges" value={props.document.quality.graphEdgeCount} />
      </div>
      {props.document.quality.warnings.length > 0 && (
        <div className="grid gap-2 rounded-none border border-warning/30 bg-warning/10 p-3 text-sm leading-6 text-warning">
          <div className="flex items-center gap-2 font-medium">
            <ShieldAlert size={14} />
            Documentation warnings
          </div>
          <ul className="m-0 list-disc space-y-1 pl-5">
            {props.document.quality.warnings.map((warning) => <li key={warning}>{warning}</li>)}
          </ul>
        </div>
      )}
    </div>
  );
}

function QualityMetric(props: { label: string; value: number }): ReactElement {
  return (
    <div className="rounded-none bg-card p-3">
      <div className="text-xs leading-5 text-muted-foreground">{props.label}</div>
      <strong className="text-lg leading-6 text-foreground">{props.value}</strong>
    </div>
  );
}

function EvidencePlan(props: { document: { semanticPlan: { audience: string; intent: string; sections: Array<{ title: string; purpose: string; evidenceType: string; requiredSignals: string[] }>; evidenceSources: string[] } } }): ReactElement {
  return (
    <div className="grid gap-4 rounded-none border bg-card/60 p-4 md:grid-cols-[minmax(0,1fr)_minmax(260px,0.7fr)]">
      <div className="grid gap-3">
        <div>
          <div className="text-xs font-medium uppercase tracking-[0.12em] text-muted-foreground">Audience</div>
          <div className="text-sm leading-6 text-foreground">{props.document.semanticPlan.audience}</div>
        </div>
        <div>
          <div className="text-xs font-medium uppercase tracking-[0.12em] text-muted-foreground">Intent</div>
          <div className="text-sm leading-6 text-foreground">{props.document.semanticPlan.intent}</div>
        </div>
        <div className="grid gap-2">
          <div className="text-xs font-medium uppercase tracking-[0.12em] text-muted-foreground">Sections</div>
          {props.document.semanticPlan.sections.map((section) => (
            <div className="rounded-none bg-muted p-3" key={section.title}>
              <div className="font-medium text-foreground">{section.title}</div>
              <div className="mt-1 text-sm leading-6 text-muted-foreground">{section.purpose}</div>
              <div className="mt-2 flex flex-wrap gap-1.5">
                {section.requiredSignals.map((signal) => <Badge key={signal} variant="outline">{signal}</Badge>)}
              </div>
            </div>
          ))}
        </div>
      </div>
      <div className="grid gap-2">
        <div className="text-xs font-medium uppercase tracking-[0.12em] text-muted-foreground">Evidence sources</div>
        {props.document.semanticPlan.evidenceSources.map((source) => (
          <div className="rounded-none bg-muted p-3 text-sm leading-6 text-muted-foreground" key={source}>{source}</div>
        ))}
      </div>
    </div>
  );
}
