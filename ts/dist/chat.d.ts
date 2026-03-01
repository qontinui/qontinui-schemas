/**
 * Chat session types shared between web and runner frontends.
 */
/** State of a chat session. */
type ChatSessionState = "connecting" | "initializing" | "ready" | "processing" | "interrupting" | "closed" | "disconnected" | "error" | "not_found" | "restoring";
/** A single message in a chat conversation. */
interface ChatMessage {
    role: "user" | "ai" | "system";
    content: string;
    timestamp?: string;
}

export type { ChatMessage, ChatSessionState };
