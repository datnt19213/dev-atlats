import type { ReactElement } from "react";
import { Copy, ScanLine } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Empty } from "@/components/ui/empty";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Table } from "@/components/ui/table";
import { Typography } from "@/components/ui/typography";
import { useAppControllerContext } from "@/providers/AppControllerProvider";

export function ExplorerPage(): ReactElement {
  const controller = useAppControllerContext();

  const files = controller.explorerFiles;
  const totalSize = files.reduce((total, file) => total + file.sizeBytes, 0);

  return (
    <section className="grid gap-6">
      <div className="grid gap-3 rounded-none bg-card/85 p-4 backdrop-blur-xl md:grid-cols-[minmax(220px,1fr)_180px_auto]">
        <Input aria-label="Search files" className="h-11" placeholder="Search files" />
        <Select aria-label="Filter by extension">
          <SelectTrigger className="h-11 rounded-none">
            <SelectValue placeholder="All files" />
          </SelectTrigger>
          <SelectContent className="rounded-none">
            <SelectItem value="all">All files</SelectItem>
            <SelectItem value="ts">TypeScript</SelectItem>
            <SelectItem value="tsx">React</SelectItem>
            <SelectItem value="json">JSON</SelectItem>
            <SelectItem value="css">Styles</SelectItem>
          </SelectContent>
        </Select>
        <Button type="button" onClick={controller.refreshFiles}>
          <ScanLine size={16} />
          Refresh
        </Button>
      </div>

      <div className="grid gap-5 md:grid-cols-3">
        <MetricCard label="Files Loaded" value={String(files.length)} />
        <MetricCard label="Visible Files" value={String(files.length)} />
        <MetricCard label="Total Size" value={`${totalSize} B`} />
      </div>

      <div className="grid gap-5 xl:grid-cols-[360px_minmax(0,1fr)]">
        <Card>
          <CardHeader>
            <CardTitle>File Tree</CardTitle>
            <CardDescription>Repository files from the current scan.</CardDescription>
          </CardHeader>
          <CardContent className="max-h-[560px] overflow-auto [scrollbar-width:thin] [&::-webkit-scrollbar]:w-1.5 [&::-webkit-scrollbar-track]:bg-transparent [&::-webkit-scrollbar-thumb]:rounded-none [&::-webkit-scrollbar-thumb]:bg-border/60">
            {files.length > 0 ? (
              <ul className="grid gap-2">
                {files.map((file) => (
                  <li key={file.path} className="rounded-none bg-muted p-3 text-sm leading-6 text-muted-foreground">
                    {file.path}
                  </li>
                ))}
              </ul>
            ) : (
              <Empty>No files loaded yet.</Empty>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Repository Files</CardTitle>
            <CardDescription>{files.length} files</CardDescription>
          </CardHeader>
          <CardContent className="max-h-[560px] overflow-auto [scrollbar-width:thin] [&::-webkit-scrollbar]:w-1.5 [&::-webkit-scrollbar-track]:bg-transparent [&::-webkit-scrollbar-thumb]:rounded-none [&::-webkit-scrollbar-thumb]:bg-border/60">
            {files.length > 0 ? (
              <Table>
                <tbody>
                  {files.map((file) => (
                    <tr key={file.path} className="align-top">
                      <td className="py-2.5 pr-4 text-sm leading-6">{file.path}</td>
                      <td className="py-2.5 pr-4 text-sm leading-6 text-muted-foreground">{file.extension ?? "-"}</td>
                      <td className="py-2.5 pr-4 text-sm leading-6 text-muted-foreground">{file.sizeBytes} B</td>
                      <td className="py-1.5">
                        <Button aria-label={`Copy ${file.path}`} type="button" variant="icon" onClick={() => controller.copyText("File path", file.path)}>
                          <Copy size={14} />
                        </Button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </Table>
            ) : (
              <Empty>No repository files available.</Empty>
            )}
          </CardContent>
        </Card>
      </div>
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
