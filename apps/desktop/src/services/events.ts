import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type { DomainEvent, DomainEventType } from "../types/contracts";
import type { AppStatus } from "../stores/app-store";

export const DOMAIN_EVENT_NAME = "devatlas://domain-event";
export type QueryKey = readonly string[];

const WORKING_EVENT_TYPES = new Set<DomainEventType>(["Scanner.ScanStarted"]);

export async function listenToDomainEvents(
  handler: (event: DomainEvent) => void,
): Promise<UnlistenFn> {
  return listen<DomainEvent>(DOMAIN_EVENT_NAME, (event) => handler(event.payload));
}

export function statusForDomainEvent(eventType: DomainEvent["eventType"]): AppStatus {
  return WORKING_EVENT_TYPES.has(eventType) ? "working" : "ready";
}

export function queryKeysForDomainEvent(event: DomainEvent): QueryKey[] {
  switch (event.payload.type) {
    case "RepositoryOpened":
      return [["repositories"]];
    case "GraphSnapshotSaved":
      return [["graphSnapshots", event.payload.data.repositoryId]];
    case "AiVectorSnapshotSaved":
      return [["aiVectorSnapshots", event.payload.data.repositoryId]];
    case "AiChatSessionStarted":
      return [["chatSessions", event.payload.data.repositoryId]];
    case "AiChatMessageSaved":
      return [["chatMessages", event.payload.data.sessionId]];
    default:
      return [];
  }
}
