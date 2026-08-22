import type { ReactElement, ReactNode } from "react";
import { Blend, Image, Moon, RefreshCw, Sparkles, Sun } from "lucide-react";

import { Alert } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Field } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { Toggle } from "@/components/ui/toggle";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { Typography } from "@/components/ui/typography";
import { defaultUiPreferences, type UiPreferences } from "@/handlers/preferences";
import type { AppController } from "@/handlers/use-app-controller";
import { useAppControllerContext } from "@/providers/AppControllerProvider";

export function SettingsPage(): ReactElement {
  const controller = useAppControllerContext();

  return (
    <section className="grid gap-8">
      <Card className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_auto] lg:items-start">
        <div className="grid gap-2">
          <Badge tone="brand">Control Surface</Badge>
          <Typography variant="h2">Workspace Preferences</Typography>
          <Typography variant="muted">Adjust the shell, visual theme, animation, and repository workflow behavior.</Typography>
        </div>
        <Button type="button" onClick={() => controller.updateUiPreferences(defaultUiPreferences)}>
          <RefreshCw size={16} />
          Reset
        </Button>
      </Card>

      <section className="grid gap-8 xl:grid-cols-2">
        <AppearanceCard
          preferences={controller.uiPreferences}
          updatePreferences={controller.updateUiPreferences}
        />
        <BackgroundCard
          preferences={controller.uiPreferences}
          updatePreferences={controller.updateUiPreferences}
        />
        <LayoutCard
          preferences={controller.uiPreferences}
          updatePreferences={controller.updateUiPreferences}
        />
        <WorkflowCard
          preferences={controller.uiPreferences}
          updatePreferences={controller.updateUiPreferences}
        />
      </section>

      <Alert tone="warning">
        Browser preview is running without the Tauri native bridge. Run `yarn dev` from the workspace root and use the DevAtlas desktop window for repository scanning, dialogs, and exports.
      </Alert>
    </section>
  );
}

function AppearanceCard(props: {
  preferences: UiPreferences;
  updatePreferences: AppController["updateUiPreferences"];
}): ReactElement {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Appearance</CardTitle>
        <CardDescription>Theme and surface tone</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="grid gap-5 md:grid-cols-2">
          <ChoiceCard
            active={props.preferences.themeMode === "dark"}
            description="Low-glare workspace for long scan sessions."
            icon={<Moon size={15} />}
            previewClassName="bg-black h-full w-5!"
            title="Dark"
            onClick={() => props.updatePreferences({ themeMode: "dark" })}
          />
          <ChoiceCard
            active={props.preferences.themeMode === "light"}
            description="Higher contrast surface for daytime review."
            icon={<Sun size={15} />}
            previewClassName="bg-white h-full w-5!"
            title="Light"
            onClick={() => props.updatePreferences({ themeMode: "light" })}
          />
        </div>
      </CardContent>
    </Card>
  );
}

function BackgroundCard(props: {
  preferences: UiPreferences;
  updatePreferences: AppController["updateUiPreferences"];
}): ReactElement {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Background</CardTitle>
        <CardDescription>Right-side workspace texture</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="grid gap-5">
          <ChoiceCard
            active={props.preferences.backdropMode === "aurora"}
            description="Soft colored depth for the main surface."
            icon={<Sparkles size={15} />}
            previewClassName="bg-primary"
            title="Aurora"
            onClick={() => props.updatePreferences({ backdropMode: "aurora" })}
          />
          <ChoiceCard
            active={props.preferences.backdropMode === "mesh"}
            description="Subtle grid texture for technical review."
            icon={<Blend size={15} />}
            previewClassName="bg-muted"
            title="Mesh"
            onClick={() => props.updatePreferences({ backdropMode: "mesh" })}
          />
          <ChoiceCard
            active={props.preferences.backdropMode === "plain"}
            description="Flat surface with no decorative texture."
            icon={<Image size={15} />}
            previewClassName="bg-border/60"
            title="Plain"
            onClick={() => props.updatePreferences({ backdropMode: "plain" })}
          />
        </div>
      </CardContent>
    </Card>
  );
}

function LayoutCard(props: {
  preferences: UiPreferences;
  updatePreferences: AppController["updateUiPreferences"];
}): ReactElement {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Layout</CardTitle>
        <CardDescription>Navigation and motion</CardDescription>
      </CardHeader>
      <CardContent>
        <SwitchRow
          checked={props.preferences.sidebarCollapsed}
          description="Collapse navigation to icon-only mode."
          label="Collapsed sidebar"
          onChange={(checked) => props.updatePreferences({ sidebarCollapsed: checked })}
        />
        <ModeSelector
          description="Controls page and sidebar animation."
          label="Motion"
          options={[
            { active: props.preferences.motionEnabled, label: "Animated", onClick: () => props.updatePreferences({ motionEnabled: true }) },
            { active: !props.preferences.motionEnabled, label: "Reduced", onClick: () => props.updatePreferences({ motionEnabled: false }) },
          ]}
        />
      </CardContent>
    </Card>
  );
}

function WorkflowCard(props: {
  preferences: UiPreferences;
  updatePreferences: AppController["updateUiPreferences"];
}): ReactElement {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Workflow</CardTitle>
        <CardDescription>Scanner automation</CardDescription>
      </CardHeader>
      <CardContent>
        <SwitchRow
          checked={props.preferences.generateDocsOnAnalyze}
          description="Generate documentation as part of Analyze."
          label="Analyze generates docs"
          onChange={(checked) => props.updatePreferences({ generateDocsOnAnalyze: checked })}
        />
        <SwitchRow
          checked={props.preferences.generateDiagramsOnAnalyze}
          description="Generate diagrams as part of Analyze."
          label="Analyze generates diagrams"
          onChange={(checked) => props.updatePreferences({ generateDiagramsOnAnalyze: checked })}
        />
        <SettingField
          description="Leave empty for full source scan."
          label="Scan file limit"
        >
          <Input
            aria-label="Scan file limit"
            inputMode="numeric"
            min="1"
            placeholder="Full"
            type="number"
            value={props.preferences.scanMaxFiles}
            onChange={(event) => props.updatePreferences({ scanMaxFiles: event.target.value })}
          />
        </SettingField>
      </CardContent>
    </Card>
  );
}

function ChoiceCard(props: {
  active: boolean;
  description: string;
  icon: ReactNode;
  previewClassName: string;
  title: string;
  onClick: () => void;
}): ReactElement {
  return (
    <Toggle
      aria-pressed={props.active}
      className={"h-auto min-h-24 justify-start rounded-none bg-muted p-4 text-left"}
      pressed={props.active}
      onClick={props.onClick}
    >
      <span aria-hidden="true" className={cn("h-11 w-11 rounded-none", props.previewClassName)} />
      <span className="grid min-w-0 gap-1.5">
        <span className="flex items-center gap-2.5">
          {props.icon}
          <strong className="text-sm font-semibold leading-5">{props.title}</strong>
        </span>
        <em className="text-xs not-italic leading-5 text-muted-foreground">{props.description}</em>
      </span>
    </Toggle>
  );
}

function ModeSelector(props: {
  description: string;
  label: string;
  options: Array<{
    active: boolean;
    label: string;
    onClick: () => void;
  }>;
}): ReactElement {
  return (
    <div className="grid gap-4 rounded-none bg-muted p-5">
      <div>
        <strong className="text-sm font-semibold leading-5">{props.label}</strong>
        <em className="block text-xs not-italic leading-5 text-muted-foreground">{props.description}</em>
      </div>
      <ToggleGroup type="single" className="w-fit" defaultValue={props.options.find((option) => option.active)?.label}>
        {props.options.map((option) => (
          <ToggleGroupItem
            aria-pressed={option.active}
            className="px-4 text-sm font-medium leading-5"
            key={option.label}
            value={option.label}
            onClick={option.onClick}
          >
            {option.label}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>
    </div>
  );
}

function SwitchRow(props: {
  checked: boolean;
  description: string;
  label: string;
  onChange: (checked: boolean) => void;
}): ReactElement {
  return (
    <div className="grid grid-cols-[minmax(0,1fr)_auto] items-start gap-5 rounded-none bg-muted p-5">
      <div className="grid min-w-0 gap-2.5">
        <strong className="text-sm font-semibold leading-5">{props.label}</strong>
        <p className="m-0 text-xs leading-5 text-muted-foreground">{props.description}</p>
      </div>
      <Switch checked={props.checked} onClick={() => props.onChange(!props.checked)} />
    </div>
  );
}

function SettingField(props: {
  children: ReactNode;
  description: string;
  label: string;
}): ReactElement {
  return (
    <Field className="rounded-none bg-muted p-5" description={props.description} label={props.label}>
      {props.children}
    </Field>
  );
}

function cn(...classes: Array<string | false | null | undefined>): string {
  return classes.filter(Boolean).join(" ");
}
