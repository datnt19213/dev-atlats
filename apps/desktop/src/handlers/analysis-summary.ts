import type { ReactElement } from "react";

export interface KnowledgePackageInsight {
  documentCount: number;
  diagramCount: number;
  diagramRelationshipCount: number;
  generatedDocumentPaths: string[];
}

export function buildKnowledgePackageInsight(
  documents: Array<{ path: string }>,
  diagrams: Array<{ id: string }>
): KnowledgePackageInsight {
  return {
    documentCount: documents.length,
    diagramCount: diagrams.length,
    diagramRelationshipCount: 0,
    generatedDocumentPaths: documents.map((doc) => doc.path),
  };
}

export interface DiagramInsight {
  id: string;
  signal: string;
  relationshipCount: number;
}

export function buildDiagramInsights(diagrams: Array<{ id: string; diagramType: string; format: string }>): DiagramInsight[] {
  return diagrams.map((diagram) => ({
    id: diagram.id,
    signal: `${diagram.diagramType} diagram with ${diagram.format} format`,
    relationshipCount: 0,
  }));
}