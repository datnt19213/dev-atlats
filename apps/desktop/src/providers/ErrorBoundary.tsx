import type { ErrorInfo, PropsWithChildren, ReactElement } from "react";
import { Component } from "react";

import { Alert } from "../components/ui/alert";
import { Button } from "../components/ui/button";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "../components/ui/card";
import { Typography } from "../components/ui/typography";

interface ErrorBoundaryState {
  errorMessage?: string;
}

export class ErrorBoundary extends Component<PropsWithChildren, ErrorBoundaryState> {
  override state: ErrorBoundaryState = {};

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { errorMessage: error.message };
  }

  override componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    console.error("Unhandled React error", error, errorInfo.componentStack);
  }

  override render(): ReactElement {
    if (this.state.errorMessage) {
      return (
        <main className="grid min-h-screen place-items-center bg-background p-6 text-foreground" role="alert">
          <Card className="fatal-error-panel max-w-2xl border-destructive bg-card">
            <CardHeader>
              <CardTitle>DevAtlas encountered a presentation error.</CardTitle>
            </CardHeader>
            <CardContent className="grid gap-4">
              <Alert tone="danger">
                <Typography>{this.state.errorMessage}</Typography>
              </Alert>
              <Button type="button" onClick={() => window.location.reload()}>
                Reload
              </Button>
            </CardContent>
          </Card>
        </main>
      );
    }

    return <>{this.props.children}</>;
  }
}
