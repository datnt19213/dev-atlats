import type { ReactElement } from 'react';

import {
  Check,
  FileCheck,
  FolderCheck,
  Loader2,
  Lock,
} from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Empty } from '@/components/ui/empty';
import { Input } from '@/components/ui/input';
import { Modal } from '@/components/ui/modal';
import { Typography } from '@/components/ui/typography';
import { useAppControllerContext } from '@/providers/AppControllerProvider';

export function ScannerPage(): ReactElement {
  const controller = useAppControllerContext();
  const isWorking = controller.processingAction !== null;
  const projectTotalFiles = controller.explorerFiles.length;
  const scopeTotalFiles = controller.analysisScopeFiles.length;
  const analyzedFiles = controller.scan?.files ?? 0;
  const scopeFolders = getScopeFolders(controller.explorerFiles);
  const scopeModeLabel = controller.scopeMode === "folders" ? "Folders" : controller.scopeMode === "files" ? "Files" : "Folders & Extra files";
  const effectiveFolders: string[] = controller.scopeMode === "files" ? [] : controller.selectedFolders;
  const effectiveFiles: string[] = controller.scopeMode === "folders" ? [] : controller.selectedFiles;
  const steps = controller.scanSteps;
  const doneCount = steps.filter((step) => step.status === "done").length;
  const progressPct = Math.round((doneCount / steps.length) * 100);
  const activeStep = steps.find((step) => step.status === "current" || step.status === "working");
  const progressLabel = activeStep ? "Next: " + activeStep.label : "All steps complete";
  const workingText =
    controller.processingAction === "select" ? "Selecting folder..." :
    controller.processingAction === "open" ? "Opening repository..." :
    controller.processingAction === "detect" ? "Detecting technologies..." :
    controller.processingAction === "analyze" ? "Scanning and building graph..." : "";
  const stepActions: Record<string, { label: string; run: () => void }> = {
    open: { label: "Open Repository", run: controller.selectAndOpenRepository },
    detect: { label: "Detect", run: controller.detectTechnologies },
    analyze: { label: "Analyze", run: controller.openAnalyzeConfirm },
  };

  return (
    <section className="grid gap-6">
      <div className="grid gap-3 rounded-none bg-card/85 p-4 backdrop-blur-xl">
        <Input
          aria-label="Repository path"
          placeholder="C:\path\to\repository"
          className="h-11"
          value={controller.repositoryPath}
          onChange={(event) => controller.setRepositoryPath(event.target.value)}
        />

        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="grid gap-0.5">
            <Typography variant="h3">Repository scan</Typography>
            <Typography variant="muted">{progressLabel}</Typography>
          </div>
          <Badge tone={controller.processingAction ? "brand" : "neutral"}>{doneCount}/{steps.length} steps</Badge>
        </div>

        <div className="h-1.5 w-full overflow-hidden rounded-none bg-muted">
          <div className="h-full bg-primary transition-[width] duration-300" style={{ width: `${progressPct}%` }} />
        </div>

        <div className='h-5'>
          {controller.processingAction && (
            <div className="flex items-center gap-2 text-sm text-foreground">
              <Loader2 size={16} className="animate-spin" />
              <span>{workingText}</span>
            </div>
          )}
        </div>

        <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
          {steps.map((step, index) => {
            const action = stepActions[step.key];
            const disabled = step.status === "locked" || step.status === "working";
            const variant = step.status === "done" ? "secondary" : step.status === "current" ? "default" : "outline";
            return (
              <div
                key={step.key}
                className={[
                  "grid gap-3 rounded-none border p-4",
                  step.status === "done" ? "border-primary/40 bg-primary/5" : "border-border bg-card/70",
                  step.status === "current" ? "ring-2 ring-primary" : "",
                  step.status === "locked" ? "opacity-60" : "",
                ].join(" ")}
              >
                <div className="flex items-start gap-3">
                  <StepIcon status={step.status} index={index + 1} />
                  <div className="min-w-0">
                    <div className="text-sm font-semibold leading-5 text-foreground">{step.label}</div>
                    <div className="text-xs leading-4 text-muted-foreground">{step.description}</div>
                  </div>
                </div>
                <Button
                  variant={variant}
                  type="button"
                  disabled={disabled}
                  onClick={action.run}
                >
                  {step.status === "working" && <Loader2 size={16} className="animate-spin" />}
                  {step.status === "locked" && <Lock size={16} />}
                  {action.label}
                </Button>
              </div>
            );
          })}
        </div>
      </div>

      <div className="grid gap-5 md:grid-cols-2 xl:grid-cols-4">
        <MetricCard label="Project files" value={String(projectTotalFiles)} />
        <MetricCard label="Scope files" value={String(scopeTotalFiles)} />
        <MetricCard label="Analyzed files" value={String(analyzedFiles)} />
        <MetricCard label="Folders in scope" value={String(scopeFolders.length)} />
      </div>

      <Card>
        <CardHeader className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
          <div>
            <CardTitle>Analysis Scope</CardTitle>
            <CardDescription>Choose what to include in the next Analyze run.</CardDescription>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Button size="sm" type="button" variant={controller.scopeMode === "folders" ? "default" : "outline"} disabled={isWorking} onClick={() => controller.setScopeMode("folders")}>
              Folders
            </Button>
            <Button size="sm" type="button" variant={controller.scopeMode === "files" ? "default" : "outline"} disabled={isWorking} onClick={() => controller.setScopeMode("files")}>
              Files
            </Button>
            <Button size="sm" type="button" variant={controller.scopeMode === "folders-extra" ? "default" : "outline"} disabled={isWorking} onClick={() => controller.setScopeMode("folders-extra")}>
              Folders &amp; Extra files
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {projectTotalFiles === 0 ? (
            <Empty>Open or detect a repository to load selectable files and folders.</Empty>
          ) : (
            <>
              <div className="flex flex-wrap items-center justify-between gap-3">
                <div>
                  {controller.analysisScopePaths.length === 0 ? (
                    <AlertBox tone="neutral">Full repository scope. Analyze scans every listed file.</AlertBox>
                  ) : (
                    <AlertBox tone="brand">{controller.analysisScopeFiles.length} files will be analyzed in the current mode.</AlertBox>
                  )}
                </div>
                <div className="flex flex-wrap gap-2">
                  <Button variant="secondary" size="sm" type="button" onClick={controller.selectAllScope} disabled={isWorking || projectTotalFiles === 0}>
                    Select all
                  </Button>
                  <Button variant="secondary" size="sm" type="button" onClick={controller.clearScope} disabled={isWorking || controller.analysisScopePaths.length === 0}>
                    Clear scope
                  </Button>
                </div>
              </div>

              <div className="mt-5 grid gap-4 lg:grid-cols-2">
                <ScopeList title="Folders" emptyText="No folders found.">
                  {scopeFolders.map((folder) => (
                    <ScopeRow
                      active={controller.isScopeActive(folder, "folder")}
                      disabled={isWorking || controller.isScopeDisabled(folder, "folder")}
                      icon={<FolderCheck size={16} />}
                      key={folder}
                      label={folder}
                      meta={countFilesInFolder(controller.explorerFiles, folder) + " files"}
                      onToggle={() => controller.toggleScopeSelection(folder, "folder")}
                    />
                  ))}
                </ScopeList>

                <ScopeList title="Files" emptyText="No files found.">
                  {controller.explorerFiles.map((file) => (
                    <ScopeRow
                      active={controller.isScopeActive(file.path, "file")}
                      disabled={isWorking || controller.isScopeDisabled(file.path, "file")}
                      icon={<FileCheck size={16} />}
                      key={file.path}
                      label={file.path}
                      meta={file.extension ?? "file"}
                      onToggle={() => controller.toggleScopeSelection(file.path, "file")}
                    />
                  ))}
                </ScopeList>
              </div>
              {controller.scopeMode === "files" && (
                <Typography variant="muted" className="mt-3">Folder selection is disabled in Files mode.</Typography>
              )}
              {controller.scopeMode === "folders" && (
                <Typography variant="muted" className="mt-3">File selection is disabled in Folders mode.</Typography>
              )}
            </>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Detected Technologies</CardTitle>
          <CardDescription>Technology signatures found in the opened repository.</CardDescription>
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

      <Modal
        open={controller.analyzeConfirmOpen}
        onClose={controller.cancelAnalyze}
        title="Confirm Analysis"
        subtitle="Review the selected scope before analyzing."
        footer={
          <>
            <Button variant="secondary" type="button" onClick={controller.cancelAnalyze}>Cancel</Button>
            <Button type="button" onClick={controller.confirmAnalyze}>Confirm &amp; Analyse</Button>
          </>
        }
      >
        <div className="grid gap-4 text-sm">
          <div>
            <Typography variant="muted">Scope mode</Typography>
            <Typography variant="h3">{scopeModeLabel}</Typography>
          </div>
          {effectiveFolders.length === 0 && effectiveFiles.length === 0 ? (
            <AlertBox tone="neutral">No scope selected - the full repository will be analyzed.</AlertBox>
          ) : (
            <>
              <div>
                <Typography variant="muted">Folders ({effectiveFolders.length})</Typography>
                <ul className="mt-1 grid gap-1">
                  {effectiveFolders.map((folder) => (
                    <li key={folder} className="truncate rounded-none bg-muted px-2 py-1 text-foreground">{folder}</li>
                  ))}
                </ul>
              </div>
              <div>
                <Typography variant="muted">Extra files ({effectiveFiles.length})</Typography>
                <ul className="mt-1 grid gap-1">
                  {effectiveFiles.map((file) => (
                    <li key={file} className="truncate rounded-none bg-muted px-2 py-1 text-foreground">{file}</li>
                  ))}
                </ul>
              </div>
            </>
          )}
          <div className="rounded-none border border-primary bg-primary/10 p-3">
            <Typography variant="muted">Files to analyze</Typography>
            <Typography variant="h2">{controller.analysisScopeFiles.length}</Typography>
          </div>
        </div>
      </Modal>
    </section>
  );
}

function MetricCard(props: { label: string; value: string }): ReactElement {
  return (
    <Card compact>
      <CardDescription>{props.label}</CardDescription>
      <Typography variant="h2">{props.value}</Typography>
    </Card>
  );
}

function StepIcon(props: { index: number; status: "done" | "current" | "working" | "locked" }): ReactElement {
  if (props.status === "done") {
    return (
      <span className="grid h-7 w-7 place-items-center rounded-none border border-primary bg-primary text-primary-foreground">
        <Check size={16} />
      </span>
    );
  }
  if (props.status === "working") {
    return (
      <span className="grid h-7 w-7 place-items-center rounded-none border border-primary bg-primary/15 text-primary">
        <Loader2 size={16} className="animate-spin" />
      </span>
    );
  }
  if (props.status === "locked") {
    return (
      <span className="grid h-7 w-7 place-items-center rounded-none border border-border bg-muted text-muted-foreground">
        <Lock size={16} />
      </span>
    );
  }
  return (
    <span className="grid h-7 w-7 place-items-center rounded-none border border-primary bg-primary/10 text-sm font-semibold text-primary">
      {props.index}
    </span>
  );
}

function ScopeList(props: {
  children: ReactElement[];
  emptyText: string;
  title: string;
}): ReactElement {
  return (
    <div className="grid gap-3 rounded-none bg-muted p-4">
      <div className="flex items-center justify-between gap-3">
        <Typography variant="h3">{props.title}</Typography>
        <Badge>{props.children.length}</Badge>
      </div>
      {props.children.length === 0 ? (
        <Typography variant="muted">{props.emptyText}</Typography>
      ) : (
        <div className="grid max-h-[420px] gap-2 overflow-auto [scrollbar-width:thin] [&::-webkit-scrollbar]:w-1.5 [&::-webkit-scrollbar-track]:bg-transparent [&::-webkit-scrollbar-thumb]:rounded-none [&::-webkit-scrollbar-thumb]:bg-border/60">
          {props.children}
        </div>
      )}
    </div>
  );
}

function ScopeRow(props: {
  active: boolean;
  disabled: boolean;
  icon: ReactElement;
  label: string;
  meta: string;
  onToggle: () => void;
}): ReactElement {
  return (
    <button
      className={[
        "grid w-full grid-cols-[24px_minmax(0,1fr)_auto] items-center gap-3 rounded-none border p-3 text-left transition-colors",
        props.active ? "border-primary bg-primary/15 text-foreground" : "border-border bg-card/70 text-muted-foreground hover:bg-card",
        props.disabled && "pointer-events-none opacity-50",
      ].join(" ")}
      disabled={props.disabled}
      type="button"
      onClick={props.onToggle}
    >
      <span className={["grid h-6 w-6 place-items-center rounded-none", props.active ? "bg-primary text-primary-foreground" : "bg-secondary text-muted-foreground"].join(" ")}>
        {props.icon}
      </span>
      <span className="min-w-0">
        <span className="block truncate text-sm font-medium leading-5 text-foreground">{props.label}</span>
        <span className="block truncate text-xs leading-4 text-muted-foreground">{props.meta}</span>
      </span>
      {props.active ? <Badge tone="brand">Included</Badge> : <Badge>Excluded</Badge>}
    </button>
  );
}

function AlertBox(props: { children: React.ReactNode; tone: "neutral" | "brand" }): ReactElement {
  return (
    <div className={["rounded-none border p-4 text-sm leading-6", props.tone === "brand" ? "border-primary bg-primary/10 text-foreground" : "border-border bg-muted text-muted-foreground"].join(" ")}>
      {props.children}
    </div>
  );
}

function getScopeFolders(files: Array<{ path: string }>): string[] {
  const folders = new Set<string>();
  for (const file of files) {
    const scopePath = getScopePath(file.path);
    if (scopePath.length > 0) folders.add(scopePath);
  }
  return Array.from(folders).sort();
}

function countFilesInFolder(files: Array<{ path: string }>, folder: string): number {
  return files.filter((file) => file.path === folder || file.path.startsWith(`${folder}/`)).length;
}

function getScopePath(path: string): string {
  const parts = path.split(/[\\/]/).filter(Boolean);
  parts.pop();
  return parts.join("/");
}
