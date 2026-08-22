import './styles/global.css';

import React from 'react';

import { createRoot } from 'react-dom/client';

import { App } from './app/page';
import { AppLayout } from './app/layout';
import { DomainEventProvider } from './providers/DomainEventProvider';
import { ErrorBoundary } from './providers/ErrorBoundary';
import { ThemeProvider } from './components/ui/theme-provider';
import { AppControllerProvider } from './providers/AppControllerProvider';
import { QueryProvider } from './providers/QueryProvider';

createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider attribute="class" defaultTheme="dark" enableSystem storageKey="devatlas-theme" disableTransitionOnChange>
      <ErrorBoundary>
        <QueryProvider>
          <AppControllerProvider>
            <DomainEventProvider>
              <AppLayout>
                <App />
              </AppLayout>
            </DomainEventProvider>
          </AppControllerProvider>
        </QueryProvider>
      </ErrorBoundary>
    </ThemeProvider>
  </React.StrictMode>,
);
