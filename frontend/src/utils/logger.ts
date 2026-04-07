import type { LogEntry } from "../types/operations";

type LogLevel = LogEntry["level"];
type LogListener = (entry: LogEntry) => void;

const listeners: LogListener[] = [];

function emit(level: LogLevel, context: string, message: string) {
  const entry: LogEntry = {
    timestamp: new Date().toISOString(),
    level,
    target: context,
    message,
  };
  for (const listener of listeners) {
    listener(entry);
  }
}

export const logger = {
  debug(context: string, message: string) { emit("debug", context, message); },
  info(context: string, message: string) { emit("info", context, message); },
  warn(context: string, message: string) { emit("warn", context, message); },
  error(context: string, message: string) { emit("error", context, message); },
};

export function onLog(listener: LogListener): () => void {
  listeners.push(listener);
  return () => {
    const idx = listeners.indexOf(listener);
    if (idx !== -1) listeners.splice(idx, 1);
  };
}
