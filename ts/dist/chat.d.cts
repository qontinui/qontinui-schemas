/**
 * AI session types shared between web and runner frontends.
 */
/** State of an AI session. */
type AiSessionState = "connecting" | "initializing" | "ready" | "processing" | "interrupting" | "closed" | "disconnected" | "error" | "not_found" | "restoring";
/** A single message in an AI conversation. */
interface AiMessage {
    role: "user" | "ai" | "system";
    content: string;
    timestamp?: string;
}

export type { AiMessage, AiSessionState };
