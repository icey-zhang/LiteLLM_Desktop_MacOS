import { useEffect, useMemo, useRef, useState } from "react";
import type { LogEntry } from "../types/config";

interface LogsPanelProps {
  logs: LogEntry[];
  onClear: () => void;
}

export function LogsPanel({ logs, onClear }: LogsPanelProps) {
  const [keyword, setKeyword] = useState("");
  const scrollRef = useRef<HTMLDivElement>(null);

  const filteredLogs = useMemo(() => {
    if (!keyword.trim()) {
      return logs;
    }

    return logs.filter((entry) =>
      entry.line.toLowerCase().includes(keyword.trim().toLowerCase()),
    );
  }, [keyword, logs]);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [filteredLogs]);

  function formatTimestamp(timestamp: string) {
    const value = Number(timestamp);
    if (Number.isFinite(value)) {
      return new Date(value).toLocaleTimeString();
    }

    return new Date(timestamp).toLocaleTimeString();
  }

  return (
    <section className="panel-stack">
      <header className="panel-header">
        <div>
          <p className="eyebrow">日志</p>
          <h2>实时输出</h2>
        </div>
        <div className="action-row compact">
          <input
            aria-label="日志过滤"
            className="search-input"
            placeholder="过滤关键字"
            value={keyword}
            onChange={(event) => setKeyword(event.target.value)}
          />
          <button className="btn btn-ghost" onClick={onClear}>
            清空日志
          </button>
        </div>
      </header>

      <div className="log-console" ref={scrollRef}>
        {filteredLogs.length === 0 ? (
          <p className="empty-copy">还没有日志输出。</p>
        ) : (
          filteredLogs.map((entry, index) => (
            <div
              key={`${entry.timestamp}-${index}`}
              className={
                entry.stream === "stderr"
                  ? "log-line stderr"
                  : entry.stream === "system"
                    ? "log-line system"
                    : "log-line"
              }
            >
              <span className="log-time">{formatTimestamp(entry.timestamp)}</span>
              <code className="log-content">{entry.line}</code>
            </div>
          ))
        )}
      </div>
    </section>
  );
}
