import { describe, expect, it } from "vitest";

import { queryKeysForDomainEvent, statusForDomainEvent } from "../services/events";
import type { DomainEvent } from "../types/contracts";

describe("statusForDomainEvent", () => {
  it("marks a started scan as working", () => {
    expect(statusForDomainEvent("Scanner.ScanStarted")).toBe("working");
  });

  it("marks completed workflow events as ready", () => {
    expect(statusForDomainEvent("Scanner.ScanCompleted")).toBe("ready");
    expect(statusForDomainEvent("Export.ExportCompleted")).toBe("ready");
    expect(statusForDomainEvent("AI.ChatMessageSaved")).toBe("ready");
  });

  it("maps persisted domain events to cache query keys", () => {
    expect(
      queryKeysForDomainEvent(
        domainEvent({
          eventType: "Graph.SnapshotSaved",
          payload: {
            type: "GraphSnapshotSaved",
            data: {
              repositoryId: "repo-1",
              snapshotId: "graph-snapshot-1",
              nodeCount: 1,
              edgeCount: 0,
            },
          },
        }),
      ),
    ).toEqual([["graphSnapshots", "repo-1"]]);

    expect(
      queryKeysForDomainEvent(
        domainEvent({
          eventType: "AI.ChatMessageSaved",
          payload: {
            type: "AiChatMessageSaved",
            data: {
              repositoryId: "repo-1",
              sessionId: "chat-session-1",
              messageId: "chat-message-1",
              role: "User",
            },
          },
        }),
      ),
    ).toEqual([["chatMessages", "chat-session-1"]]);
  });
});

function domainEvent(
  input: Pick<DomainEvent, "eventType" | "payload">,
): DomainEvent {
  return {
    eventId: "event-1",
    correlationId: "repo-1",
    eventType: input.eventType,
    version: "1.0",
    timestampMs: 1,
    payload: input.payload,
  };
}
