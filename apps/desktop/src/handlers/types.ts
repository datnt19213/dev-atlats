export type AppStatus = "ready" | "working" | "error";
export type ToastTone = "success" | "warning" | "error" | "neutral";

export interface ChatTurn {
  id: string;
  question: string;
  response?: {
    model: string;
    answer: string;
    citations: Array<{
      sourceId: string;
      path: string;
      startLine: number;
      endLine: number;
    }>;
  };
}

export interface ToastMessage {
  id: string;
  title: string;
  detail?: string;
  tone: ToastTone;
}

export interface ModalContent {
  title: string;
  subtitle?: string;
  content: string;
  format?: string;
}