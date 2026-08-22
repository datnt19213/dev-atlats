import { useEffect, useState } from "react";

export interface UiPreferences {
  themeMode: "light" | "dark";
  sidebarCollapsed: boolean;
  motionEnabled: boolean;
  compactDensity: boolean;
  generateDocsOnAnalyze: boolean;
  generateDiagramsOnAnalyze: boolean;
  scanMaxFiles?: string;
  backdropMode: "aurora" | "mesh" | "plain";
}

export const defaultUiPreferences: UiPreferences = {
  themeMode: "dark",
  sidebarCollapsed: false,
  motionEnabled: true,
  compactDensity: false,
  generateDocsOnAnalyze: true,
  generateDiagramsOnAnalyze: true,
  backdropMode: "aurora",
};

export const uiPreferencesStorageKey = "devatlas-ui-preferences";

export function loadUiPreferences(): UiPreferences {
  if (typeof window === "undefined" || !window.localStorage) {
    return defaultUiPreferences;
  }
  try {
    const stored = window.localStorage.getItem(uiPreferencesStorageKey);
    return stored ? JSON.parse(stored) : defaultUiPreferences;
  } catch {
    return defaultUiPreferences;
  }
}

export function scanMaxFilesFromPreferences(preferences: UiPreferences): number | undefined {
  const value = preferences.scanMaxFiles;
  return value ? parseInt(value, 10) : undefined;
}