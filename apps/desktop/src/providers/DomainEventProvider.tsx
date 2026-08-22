import { useQueryClient } from "@tanstack/react-query";
import type { PropsWithChildren, ReactElement } from "react";
import { useEffect } from "react";

import {
  listenToDomainEvents,
  queryKeysForDomainEvent,
  statusForDomainEvent,
} from "../services/events";
import { useAppStore } from "../stores/app-store";

export function DomainEventProvider(props: PropsWithChildren): ReactElement {
  const queryClient = useQueryClient();
  const setStatus = useAppStore((state) => state.setStatus);

  useEffect(() => {
    let disposed = false;
    let removeListener: (() => void) | undefined;

    void listenToDomainEvents((event) => {
      setStatus(statusForDomainEvent(event.eventType));
      for (const queryKey of queryKeysForDomainEvent(event)) {
        void queryClient.invalidateQueries({ queryKey });
      }
    })
      .then((unlisten) => {
        if (disposed) {
          unlisten();
        } else {
          removeListener = unlisten;
        }
      })
      .catch(() => undefined);

    return () => {
      disposed = true;
      removeListener?.();
    };
  }, [queryClient, setStatus]);

  return <>{props.children}</>;
}
