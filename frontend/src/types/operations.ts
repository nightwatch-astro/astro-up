export interface Operation {
  id: string;
  label: string;
  progress: number;
  status: "running" | "complete" | "failed" | "cancelled";
  steps: OperationStep[];
}

export interface OperationStep {
  timestamp: string;
  message: string;
  level: "info" | "warn" | "error";
}

export interface LogEntry {
  timestamp: string;
  level: "error" | "warn" | "info" | "debug" | "trace";
  target: string;
  message: string;
}

export interface ConfigSnapshot {
  id: string;
  timestamp: string;
  config: Record<string, unknown>;
}
