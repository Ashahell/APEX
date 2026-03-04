import { useState, useRef, useEffect } from 'react';
import { useAppStore } from '../../stores/appStore';
import { apiPost, apiGet } from '../../lib/api';
import { TaskSidebar } from './TaskSidebar';
import { ConfirmationGate } from './ConfirmationGate';
import ReactMarkdown from 'react-markdown';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { oneDark } from 'react-syntax-highlighter/dist/esm/styles/prism';

export function Chat() {
  const [input, setInput] = useState('');
  const [sending, setSending] = useState(false);
  const [lastStats, setLastStats] = useState<{steps: number, cost: number} | null>(null);
  const [error, setError] = useState<string | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const { messages, addMessage, pendingConfirmation, setPendingConfirmation } = useAppStore();

  useEffect(() => {
    const saved = localStorage.getItem('apex-last-stats');
    if (saved) {
      try {
        setLastStats(JSON.parse(saved));
      } catch (e) {
        console.warn('Failed to load saved stats:', e);
      }
    }
  }, []);

  useEffect(() => {
    if (lastStats) {
      localStorage.setItem('apex-last-stats', JSON.stringify(lastStats));
    }
  }, [lastStats]);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const pollTaskResult = async (taskId: string, maxSteps: number = 10): Promise<{observation: string, cost?: number, steps?: number}> => {
    const timeoutSeconds = Math.max(maxSteps * 15, 60);
    for (let i = 0; i < timeoutSeconds; i++) {
      await new Promise(r => setTimeout(r, 1000));
      try {
        const res = await apiGet(`/api/v1/tasks/${taskId}`);
        if (res.ok) {
          const data = await res.json();
          if (data.status === 'completed' || data.status === 'failed') {
            if (data.output) {
              try {
                const output = JSON.parse(data.output);
                let observation = '';
                // Show only the last observation (most recent response)
                if (output.history && output.history.length > 0) {
                  const lastObservation = output.history[output.history.length - 1]?.observation;
                  if (lastObservation) observation = lastObservation;
                  else observation = output.output || data.output;
                } else {
                  observation = output.output || data.output;
                }
                return {
                  observation,
                  cost: output.total_cost_usd,
                  steps: output.steps_executed,
                };
              } catch (e) {
                console.warn('Failed to parse task output:', e);
                return { observation: data.output };
              }
            }
            if (data.status === 'failed' && data.error) {
              setError(`Task failed: ${data.error}`);
            }
            return { observation: data.error || `Task ${data.status}` };
          }
        }
      } catch (e) {
        console.warn('Poll error:', e);
        // continue polling
      }
    }
    setError('Task timed out');
    return { observation: 'Task timed out - please check Settings for task status' };
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim() || sending) return;

    const userMessage = input.trim();
    setInput('');
    setSending(true);
    setLastStats(null);

    addMessage({ role: 'user', content: userMessage });
    addMessage({ role: 'assistant', content: '⏳ Processing...' });

    // Load config from localStorage
    let taskConfig: { max_steps: number; budget_usd: number; time_limit_secs?: number } = { max_steps: 3, budget_usd: 1.0 };
    try {
      const saved = localStorage.getItem('apex-task-config');
      if (saved) {
        const parsed = JSON.parse(saved);
        taskConfig = {
          max_steps: parsed.maxSteps || 3,
          budget_usd: parsed.budgetUsd || 1.0,
        };
        if (parsed.timeLimitSecs) {
          taskConfig.time_limit_secs = parsed.timeLimitSecs;
        }
      }
    } catch (e) {
      console.warn('Failed to load task config:', e);
    }

    setError(null);

    try {
      const response = await apiPost('/api/v1/tasks', { content: userMessage, ...taskConfig });

      if (response.ok) {
        const data = await response.json();
        
        // Check for instant response (greetings, simple queries)
        if (data.instant_response) {
          messages.pop();
          addMessage({ role: 'assistant', content: data.instant_response });
          setSending(false);
          return;
        }
        
        // Check if task was auto-routed to deep (LLM)
        if (data.status === 'running' || data.tier === 'deep') {
          const result = await pollTaskResult(data.task_id, taskConfig.max_steps);
          if (result.cost !== undefined) {
            setLastStats({ steps: result.steps || 0, cost: result.cost });
          }
          addMessage({
            role: 'assistant',
            content: result.observation,
          });
        } else {
          messages.pop();
          addMessage({
            role: 'assistant',
            content: `✅ Task created\n**ID:** ${data.task_id}\n**Tier:** ${data.tier}\n**Status:** ${data.status}`,
          });
        }
      } else {
        messages.pop();
        addMessage({
          role: 'assistant',
          content: `❌ Error: ${response.statusText}`,
        });
        setError(`Server error: ${response.statusText}`);
      }
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : 'Failed to create task';
      messages.pop();
      addMessage({
        role: 'assistant',
        content: `❌ Error: ${errorMsg}`,
      });
      setError(errorMsg);
    } finally {
      setSending(false);
    }
  };

  return (
    <div className="flex h-full">
      <div className="flex-1 flex flex-col">
        <div className="border-b p-3 flex items-center gap-4 flex-wrap">
          {error && (
            <span className="text-sm bg-red-100 text-red-800 px-2 py-1 rounded flex items-center gap-2">
              ⚠️ {error}
              <button onClick={() => setError(null)} className="hover:text-red-600">✕</button>
            </span>
          )}
          {lastStats && (
            <span className="text-sm bg-muted px-2 py-1 rounded">
              Stats: {lastStats.steps} steps, ${lastStats.cost.toFixed(4)}
            </span>
          )}
        </div>

        <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {messages.length === 0 && (
          <div className="text-center text-muted-foreground mt-8">
            <p className="text-lg">Welcome to APEX</p>
            <p className="text-sm">Start a conversation to begin</p>
          </div>
        )}
        
        {messages.map((message) => (
          <div
            key={message.id}
            className={`flex ${
              message.role === 'user' ? 'justify-end' : 'justify-start'
            }`}
          >
            <div
              className={`max-w-[80%] rounded-lg p-3 ${
                message.role === 'user'
                  ? 'bg-primary text-primary-foreground'
                  : 'bg-muted'
              }`}
            >
              <div className="prose prose-sm dark:prose-invert">
                <ReactMarkdown
                  components={{
                    code(props) {
                      const { children, className, node, ...rest } = props;
                      const match = /language-(\w+)/.exec(className || '');
                      const inline = !match;
                      return !inline ? (
                        <SyntaxHighlighter
                          style={oneDark as { [key: string]: React.CSSProperties }}
                          language={match[1]}
                          PreTag="div"
                        >
                          {String(children).replace(/\n$/, '')}
                        </SyntaxHighlighter>
                      ) : (
                        <code className={className} {...rest}>
                          {children}
                        </code>
                      );
                    },
                  }}
                >
          {message.content}
                </ReactMarkdown>
              </div>
            </div>
          </div>
        ))}
        
        {pendingConfirmation && (
          <ConfirmationGate
            tier={pendingConfirmation.tier}
            action={pendingConfirmation.action}
            skillName={pendingConfirmation.skillName}
            details={pendingConfirmation.consequences ? {
              impact: pendingConfirmation.consequences.summary,
            } : undefined}
            onConfirm={(confirmationText, totpCode) => {
              console.log('Confirmed:', confirmationText, totpCode);
              setPendingConfirmation(null);
            }}
            onCancel={() => {
              setPendingConfirmation(null);
            }}
          />
        )}
        
        <div ref={messagesEndRef} />
      </div>

      <form onSubmit={handleSubmit} className="border-t p-4">
        <div className="flex gap-2">
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="Type your message..."
            disabled={sending}
            className="flex-1 px-4 py-2 rounded-lg border bg-background focus:outline-none focus:ring-2 focus:ring-primary disabled:opacity-50"
          />
          <button
            type="submit"
            disabled={sending || !input.trim()}
            className="px-4 py-2 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
          >
            {sending ? '...' : 'Send'}
          </button>
        </div>
      </form>
      </div>
      
      <TaskSidebar onTaskClick={(taskId) => console.log('Task clicked:', taskId)} />
    </div>
  );
}
