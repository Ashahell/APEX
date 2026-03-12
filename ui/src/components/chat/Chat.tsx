import { useState, useRef, useEffect } from 'react';
import { useAppStore } from '../../stores/appStore';
import { apiPost, apiGet } from '../../lib/api';
import { TaskSidebar } from './TaskSidebar';
import { ConfirmationGate } from './ConfirmationGate';
import ReactMarkdown from 'react-markdown';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { oneDark } from 'react-syntax-highlighter/dist/esm/styles/prism';

interface Attachment {
  id: string;
  name: string;
  size: number;
  type: string;
  file?: File;
}

// TypeScript declarations for Web Speech API
interface SpeechRecognitionEvent extends Event {
  results: SpeechRecognitionResultList;
}

interface SpeechRecognitionResultList {
  length: number;
  item(index: number): SpeechRecognitionResult;
  [index: number]: SpeechRecognitionResult;
}

interface SpeechRecognitionResult {
  length: number;
  item(index: number): SpeechRecognitionAlternative;
  [index: number]: SpeechRecognitionAlternative;
  isFinal: boolean;
}

interface SpeechRecognitionAlternative {
  transcript: string;
  confidence: number;
}

interface SpeechRecognition extends EventTarget {
  continuous: boolean;
  interimResults: boolean;
  lang: string;
  onresult: ((event: SpeechRecognitionEvent) => void) | null;
  onerror: ((event: Event) => void) | null;
  onend: (() => void) | null;
  start: () => void;
  stop: () => void;
  abort: () => void;
}

declare global {
  interface Window {
    SpeechRecognition: new () => SpeechRecognition;
    webkitSpeechRecognition: new () => SpeechRecognition;
  }
}

export function Chat() {
  const [input, setInput] = useState('');
  const [sending, setSending] = useState(false);
  const [lastStats, setLastStats] = useState<{steps: number, cost: number} | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [attachments, setAttachments] = useState<Attachment[]>([]);
  const [isRecording, setIsRecording] = useState(false);
  const [recognition, setRecognition] = useState<SpeechRecognition | null>(null);
  const [hoveredMessageId, setHoveredMessageId] = useState<string | null>(null);
  const [copiedMessageId, setCopiedMessageId] = useState<string | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const { 
    messages, 
    addMessage, 
    pendingConfirmation, 
    setPendingConfirmation,
    messageQueue,
    addToMessageQueue,
    removeFromMessageQueue,
    clearMessageQueue,
    isProcessingQueue,
    setIsProcessingQueue,
  } = useAppStore();

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

  // Setup Speech Recognition
  useEffect(() => {
    // Check for Web Speech API support
    const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
    if (SpeechRecognition) {
      const recognitionInstance = new SpeechRecognition();
      recognitionInstance.continuous = false;
      recognitionInstance.interimResults = true;
      recognitionInstance.lang = 'en-US';

      recognitionInstance.onresult = (event) => {
        const transcript = Array.from(event.results)
          .map((result) => result[0].transcript)
          .join('');
        
        if (event.results[0].isFinal) {
          setInput((prev) => prev + (prev ? ' ' : '') + transcript);
        }
      };

      recognitionInstance.onerror = () => {
        console.error('Speech recognition error');
        setIsRecording(false);
      };

      recognitionInstance.onend = () => {
        setIsRecording(false);
      };

      setRecognition(recognitionInstance);
    }

    return () => {
      if (recognition) {
        recognition.abort();
      }
    };
  }, []);

  // Toggle speech recording
  const toggleRecording = () => {
    if (!recognition) {
      alert('Speech recognition is not supported in your browser.');
      return;
    }

    if (isRecording) {
      recognition.stop();
    } else {
      recognition.start();
      setIsRecording(true);
    }
  };

  // Handle file attachment
  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (!files) return;

    const newAttachments: Attachment[] = Array.from(files).map((file) => ({
      id: Math.random().toString(36).substr(2, 9),
      name: file.name,
      size: file.size,
      type: file.type,
      file,
    }));

    setAttachments((prev) => [...prev, ...newAttachments]);
    
    // Reset file input
    if (fileInputRef.current) {
      fileInputRef.current.value = '';
    }
  };

  // Remove attachment
  const removeAttachment = (id: string) => {
    setAttachments((prev) => prev.filter((a) => a.id !== id));
  };

  // Format file size
  const formatFileSize = (bytes: number) => {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
  };

  // Process message queue when not busy
  useEffect(() => {
    const processQueue = async () => {
      if (messageQueue.length > 0 && !sending && !isProcessingQueue) {
        setIsProcessingQueue(true);
        const nextMessage = messageQueue[0];
        
        // Send the queued message
        setSending(true);
        addMessage({ role: 'user', content: nextMessage });
        addMessage({ role: 'assistant', content: '⏳ Processing...' });

        try {
          let taskConfig: { max_steps: number; budget_usd: number; time_limit_secs?: number } = { max_steps: 3, budget_usd: 1.0 };
          const saved = localStorage.getItem('apex-task-config');
          if (saved) {
            try { taskConfig = { ...taskConfig, ...JSON.parse(saved) }; } catch {}
          }

          const res = await apiPost('/api/v1/tasks', {
            content: nextMessage,
            max_steps: taskConfig.max_steps,
            budget_usd: taskConfig.budget_usd,
            ...(taskConfig.time_limit_secs ? { time_limit_secs: taskConfig.time_limit_secs } : {}),
          });

          if (res.ok) {
            const data = await res.json();
            const result = await pollTaskResult(data.task_id, taskConfig.max_steps);
            
            // Update the assistant message
            const msgs = useAppStore.getState().messages;
            const lastMsgId = msgs[msgs.length - 1]?.id;
            if (lastMsgId) {
              // Replace the "processing" message with result
              useAppStore.setState({
                messages: msgs.map((m, i) => 
                  i === msgs.length - 1 
                    ? { ...m, content: result.observation || 'Done' }
                    : m
                ),
              });
            }
            
            if (result.cost !== undefined) {
              const newStats = { steps: result.steps || 0, cost: result.cost };
              setLastStats(newStats);
              const currentCost = useAppStore.getState().sessionCost;
              useAppStore.setState({ sessionCost: currentCost + result.cost });
            }
          }
        } catch (err) {
          console.error('Queue processing error:', err);
        } finally {
          removeFromMessageQueue(0);
          setSending(false);
          setIsProcessingQueue(false);
        }
      }
    };

    processQueue();
  }, [messageQueue, sending, isProcessingQueue]);

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

  // Auto-resize textarea based on content
  const adjustTextareaHeight = () => {
    const textarea = textareaRef.current;
    if (textarea) {
      textarea.style.height = 'auto';
      const newHeight = Math.min(Math.max(textarea.scrollHeight, 48), 112); // min 48px, max 112px (7rem)
      textarea.style.height = `${newHeight}px`;
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey && e.keyCode !== 229) {
      e.preventDefault();
      if (input.trim() && !sending && !isProcessingQueue) {
        handleSubmit(e as unknown as React.FormEvent);
      } else if (input.trim() && (sending || isProcessingQueue)) {
        addToMessageQueue(input.trim());
        setInput('');
      }
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim()) return;

    const userMessage = input.trim();
    setInput('');
    setLastStats(null);

    // If already sending, add to queue instead
    if (sending || isProcessingQueue) {
      addToMessageQueue(userMessage);
      return;
    }

    setSending(true);

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
        <div className="border-b border-border p-3 flex items-center gap-4 flex-wrap bg-[var(--color-panel)]">
          {error && (
            <span className="text-sm bg-red-500/10 text-red-600 dark:text-red-400 px-3 py-1.5 rounded-full flex items-center gap-2 border border-red-500/20">
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <circle cx="12" cy="12" r="10"></circle>
                <line x1="12" y1="8" x2="12" y2="12"></line>
                <line x1="12" y1="16" x2="12.01" y2="16"></line>
              </svg>
              {error}
              <button onClick={() => setError(null)} className="hover:text-red-800 ml-1">
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <line x1="18" y1="6" x2="6" y2="18"></line>
                  <line x1="6" y1="6" x2="18" y2="18"></line>
                </svg>
              </button>
            </span>
          )}
          {lastStats && (
            <span className="text-sm bg-muted px-3 py-1.5 rounded-full flex items-center gap-2">
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon>
              </svg>
              {lastStats.steps} steps • ${lastStats.cost.toFixed(4)}
            </span>
          )}
        </div>

        <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {messages.length === 0 && (
          <div className="flex flex-col items-center justify-center h-full min-h-[400px]">
            {/* AgentZero-style welcome */}
            <div className="text-center mb-8">
              <div className="w-20 h-20 mx-auto mb-4 rounded-2xl bg-gradient-to-br from-[#4248f1] to-[#6b5fff] flex items-center justify-center shadow-lg shadow-[#4248f1]/30">
                <svg xmlns="http://www.w3.org/2000/svg" width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="white" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M12 2a10 10 0 1 0 10 10H12V2z"></path>
                  <path d="M12 2a10 10 0 0 1 10 10"></path>
                  <circle cx="12" cy="12" r="4"></circle>
                </svg>
              </div>
              <h2 className="text-2xl font-semibold text-[var(--color-text)] mb-2">Welcome to APEX</h2>
              <p className="text-[var(--color-text-muted)]">Your autonomous agent platform</p>
            </div>

            {/* Quick action suggestions */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3 max-w-lg w-full">
              <button
                onClick={() => setInput('Help me write a function to process data')}
                className="p-4 rounded-xl border border-[var(--color-border)] bg-[var(--color-panel)] hover:border-[#4248f1]/50 hover:bg-[#4248f1]/5 transition-all text-left group"
              >
                <div className="flex items-center gap-3 mb-2">
                  <div className="w-8 h-8 rounded-lg bg-[#4248f1]/10 flex items-center justify-center">
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#4248f1" strokeWidth="2">
                      <polyline points="16 18 22 12 16 6"></polyline>
                      <polyline points="8 6 2 12 8 18"></polyline>
                    </svg>
                  </div>
                  <span className="font-medium text-[var(--color-text)]">Write Code</span>
                </div>
                <p className="text-xs text-[var(--color-text-muted)]">Generate code from description</p>
              </button>

              <button
                onClick={() => setInput('Review my latest code changes')}
                className="p-4 rounded-xl border border-[var(--color-border)] bg-[var(--color-panel)] hover:border-[#4248f1]/50 hover:bg-[#4248f1]/5 transition-all text-left group"
              >
                <div className="flex items-center gap-3 mb-2">
                  <div className="w-8 h-8 rounded-lg bg-teal-500/10 flex items-center justify-center">
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="teal-500" strokeWidth="2">
                      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                      <polyline points="14 2 14 8 20 8"></polyline>
                      <line x1="16" y1="13" x2="8" y2="13"></line>
                      <line x1="16" y1="17" x2="8" y2="17"></line>
                    </svg>
                  </div>
                  <span className="font-medium text-[var(--color-text)]">Code Review</span>
                </div>
                <p className="text-xs text-[var(--color-text-muted)]">Analyze code for issues</p>
              </button>

              <button
                onClick={() => setInput('Search for information about')}
                className="p-4 rounded-xl border border-[var(--color-border)] bg-[var(--color-panel)] hover:border-[#4248f1]/50 hover:bg-[#4248f1]/5 transition-all text-left group"
              >
                <div className="flex items-center gap-3 mb-2">
                  <div className="w-8 h-8 rounded-lg bg-purple-500/10 flex items-center justify-center">
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="purple-500" strokeWidth="2">
                      <circle cx="11" cy="11" r="8"></circle>
                      <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
                    </svg>
                  </div>
                  <span className="font-medium text-[var(--color-text)]">Web Search</span>
                </div>
                <p className="text-xs text-[var(--color-text-muted)]">Find information online</p>
              </button>

              <button
                onClick={() => setInput('Execute shell command:')}
                className="p-4 rounded-xl border border-[var(--color-border)] bg-[var(--color-panel)] hover:border-[#4248f1]/50 hover:bg-[#4248f1]/5 transition-all text-left group"
              >
                <div className="flex items-center gap-3 mb-2">
                  <div className="w-8 h-8 rounded-lg bg-amber-500/10 flex items-center justify-center">
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="amber-500" strokeWidth="2">
                      <polyline points="4 17 10 11 4 5"></polyline>
                      <line x1="12" y1="19" x2="20" y2="19"></line>
                    </svg>
                  </div>
                  <span className="font-medium text-[var(--color-text)]">Run Command</span>
                </div>
                <p className="text-xs text-[var(--color-text-muted)]">Execute shell commands</p>
              </button>
            </div>

            {/* Keyboard shortcuts hint */}
            <p className="text-xs text-[var(--color-text-muted)] mt-6">
              Press <kbd className="px-1.5 py-0.5 rounded bg-muted text-xs">Enter</kbd> to send • <kbd className="px-1.5 py-0.5 rounded bg-muted text-xs">Ctrl+Enter</kbd> for new line
            </p>
          </div>
        )}
        
        {messages.map((message) => (
          <div
            key={message.id}
            className={`flex group ${
              message.role === 'user' ? 'justify-end' : 'justify-start'
            }`}
            onMouseEnter={() => setHoveredMessageId(message.id)}
            onMouseLeave={() => setHoveredMessageId(null)}
          >
            <div
              className={`relative max-w-[80%] rounded-2xl px-4 py-3 ${
                message.role === 'user'
                  ? 'bg-[#4248f1] text-white'
                  : 'bg-muted'
              }`}
            >
              {/* Message action buttons - show on hover */}
              {hoveredMessageId === message.id && message.role === 'user' && (
                <div className="absolute -top-8 right-0 flex gap-1 bg-[var(--color-panel)] border border-border rounded-lg p-1 shadow-md">
                  <button
                    onClick={() => {
                      navigator.clipboard.writeText(message.content);
                      setCopiedMessageId(message.id);
                      setTimeout(() => setCopiedMessageId(null), 1500);
                    }}
                    className="p-1.5 rounded hover:bg-muted transition-colors"
                    title="Copy"
                  >
                    {copiedMessageId === message.id ? (
                      <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="text-green-500">
                        <polyline points="20 6 9 17 4 12"></polyline>
                      </svg>
                    ) : (
                      <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                        <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                        <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                      </svg>
                    )}
                  </button>
                  <button
                    onClick={() => setInput(message.content)}
                    className="p-1.5 rounded hover:bg-muted transition-colors"
                    title="Edit"
                  >
                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                      <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                      <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
                    </svg>
                  </button>
                  <button
                    onClick={() => {
                      setInput(message.content);
                      // Trigger submit after setting input
                      const form = document.querySelector('form');
                      if (form) form.dispatchEvent(new Event('submit', { bubbles: true }));
                    }}
                    className="p-1.5 rounded hover:bg-muted transition-colors"
                    title="Regenerate"
                  >
                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                      <polyline points="23 4 23 10 17 10"></polyline>
                      <polyline points="1 20 1 14 7 14"></polyline>
                      <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
                    </svg>
                  </button>
                </div>
              )}
              
              {/* Assistant message actions */}
              {hoveredMessageId === message.id && message.role === 'assistant' && (
                <div className="absolute -top-8 right-0 flex gap-1 bg-[var(--color-panel)] border border-border rounded-lg p-1 shadow-md">
                  <button
                    onClick={() => {
                      navigator.clipboard.writeText(message.content);
                      setCopiedMessageId(message.id);
                      setTimeout(() => setCopiedMessageId(null), 1500);
                    }}
                    className="p-1.5 rounded hover:bg-muted transition-colors"
                    title="Copy"
                  >
                    {copiedMessageId === message.id ? (
                      <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="text-green-500">
                        <polyline points="20 6 9 17 4 12"></polyline>
                      </svg>
                    ) : (
                      <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                        <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                        <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                      </svg>
                    )}
                  </button>
                  <button
                    onClick={() => setInput(message.content)}
                    className="p-1.5 rounded hover:bg-muted transition-colors"
                    title="Use as input"
                  >
                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                      <polyline points="9 10 4 15 9 20"></polyline>
                      <path d="M20 4v7a4 4 0 0 1-4 4H4"></path>
                    </svg>
                  </button>
                </div>
              )}
              
              <div className="prose prose-sm dark:prose-invert max-w-none">
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
                          customStyle={{
                            margin: '0.5rem 0',
                            borderRadius: '0.5rem',
                            fontSize: '0.85rem',
                          }}
                        >
                          {String(children).replace(/\n$/, '')}
                        </SyntaxHighlighter>
                      ) : (
                        <code className={`${className || ''} px-1.5 py-0.5 rounded bg-black/20`} {...rest}>
                          {children}
                        </code>
                      );
                    },
                    p: ({ children }) => (
                      <p className="mb-2 last:mb-0">{children}</p>
                    ),
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
            onConfirm={(_confirmationText, _totpCode) => {
              setPendingConfirmation(null);
            }}
            onCancel={() => {
              setPendingConfirmation(null);
            }}
          />
        )}
        
        <div ref={messagesEndRef} />
      </div>

      {/* Message Queue Indicator - AgentZero style */}
      {messageQueue.length > 0 && (
        <div className="border-t border-b p-2 bg-orange-500/10 flex items-center justify-between">
          <div className="flex items-center gap-2 text-sm">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-orange-500">
              <line x1="8" y1="6" x2="21" y2="6"></line>
              <line x1="8" y1="12" x2="21" y2="12"></line>
              <line x1="8" y1="18" x2="21" y2="18"></line>
              <line x1="3" y1="6" x2="3.01" y2="6"></line>
              <line x1="3" y1="12" x2="3.01" y2="12"></line>
              <line x1="3" y1="18" x2="3.01" y2="18"></line>
            </svg>
            <span className="text-orange-600 dark:text-orange-400">
              {messageQueue.length} message{messageQueue.length > 1 ? 's' : ''} queued
            </span>
          </div>
          <button
            onClick={() => clearMessageQueue()}
            className="text-xs px-3 py-1 text-orange-600 hover:bg-orange-500/20 rounded-full transition-colors"
          >
            Clear all
          </button>
        </div>
      )}

      {/* Attachments preview */}
      {attachments.length > 0 && (
        <div className="border-t border-b p-2 bg-muted/30 flex items-center gap-2 overflow-x-auto">
          {attachments.map((attachment) => (
            <div
              key={attachment.id}
              className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-background border border-border text-sm flex-shrink-0"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="text-[#4248f1]">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                <polyline points="14 2 14 8 20 8"></polyline>
              </svg>
              <span className="max-w-[150px] truncate">{attachment.name}</span>
              <span className="text-xs text-muted-foreground">({formatFileSize(attachment.size)})</span>
              <button
                type="button"
                onClick={() => removeAttachment(attachment.id)}
                className="ml-1 hover:text-red-500 transition-colors"
              >
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <line x1="18" y1="6" x2="6" y2="18"></line>
                  <line x1="6" y1="6" x2="18" y2="18"></line>
                </svg>
              </button>
            </div>
          ))}
        </div>
      )}

      <form onSubmit={handleSubmit} className="border-t border-border p-3 flex items-center gap-2 bg-[var(--color-panel)]">
        {/* Attachment button */}
        <div className="flex-shrink-0">
          <label htmlFor="file-input" className={`cursor-pointer transition-opacity flex items-center ${attachments.length > 0 ? 'text-[#4248f1]' : 'opacity-70 hover:opacity-100'}`} title="Add attachments">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="currentColor" className="w-6 h-6">
              <path d="M16.5 6v11.5c0 2.21-1.79 4-4 4s-4-1.79-4-4V5c0-1.38 1.12-2.5 2.5-2.5s2.5 1.12 2.5 2.5v10.5c0 .55-.45 1-1 1s-1-.45-1-1V6H10v9.5c0 1.38 1.12 2.5 2.5 2.5s2.5-1.12 2.5-2.5V5c0-2.21-1.79-4-4-4S7 2.79 7 5v12.5c0 3.04 2.46 5.5 5.5 5.5s5.5-2.46 5.5-5.5V6h-1.5z" />
            </svg>
          </label>
          <input 
            type="file" 
            id="file-input" 
            accept="*" 
            multiple 
            className="hidden" 
            ref={fileInputRef}
            onChange={handleFileChange}
          />
        </div>

        {/* Textarea container */}
        <div className="relative flex-1">
          <textarea
            ref={textareaRef}
            value={input}
            onChange={(e) => {
              setInput(e.target.value);
              adjustTextareaHeight();
            }}
            onKeyDown={handleKeyDown}
            placeholder={sending || isProcessingQueue ? "Queue mode - Enter to add to queue" : "Type your message..."}
            disabled={sending}
            rows={1}
            className="w-full px-3 py-2 pr-10 rounded-xl border border-border bg-[var(--color-background)] focus:outline-none focus:ring-2 focus:ring-[#4248f1]/50 focus:border-[#4248f1] resize-none overflow-hidden disabled:opacity-50 transition-colors"
            style={{ minHeight: '3.05rem', maxHeight: '7rem' }}
          />
          {/* Expand button */}
          <button
            type="button"
            className="absolute right-2 top-3 opacity-40 hover:opacity-70 transition-opacity bg-transparent border-none cursor-pointer"
            title="Expand input"
          >
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" className="w-5 h-5">
              <path d="M7 14H5v5h5v-2H7v-3zm-2-4h2V7h3V5H5v5zm12 7h-3v2h5v-5h-2v3zM14 5v2h3v3h2V5h-5z"/>
            </svg>
          </button>
        </div>

        {/* Buttons wrapper */}
        <div className="flex items-center gap-1 pl-1">
          {/* Send button - AgentZero style: round, indigo */}
          <button
            type="submit"
            disabled={!input.trim()}
            className="w-10 h-10 rounded-full flex items-center justify-center transition-all hover:scale-105 disabled:opacity-50 disabled:hover:scale-100"
            style={{ 
              backgroundColor: (sending || isProcessingQueue) ? '#e67e22' : '#4248f1',
              color: 'white'
            }}
            title={(sending || isProcessingQueue) ? "Add to queue" : "Send message"}
          >
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" className="w-5 h-5">
              <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/>
            </svg>
          </button>

          {/* Microphone button - AgentZero style: round */}
          <button
            type="button"
            onClick={toggleRecording}
            className={`w-10 h-10 rounded-full flex items-center justify-center transition-all hover:scale-105 ${
              isRecording 
                ? 'bg-red-500 text-white animate-pulse' 
                : 'bg-muted text-muted-foreground hover:bg-muted/80'
            }`}
            title={isRecording ? "Stop recording" : "Start voice input"}
          >
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 18" fill="currentColor" className="w-5 h-5">
              <path d="m8,12c1.66,0,3-1.34,3-3V3c0-1.66-1.34-3-3-3s-3,1.34-3,3v6c0,1.66,1.34,3,3,3Zm-1,1.9c-2.7-.4-4.8-2.6-5-5.4H0c.2,3.8,3.1,6.9,7,7.5v2h2v-2c3.9-.6,6.8-3.7,7-7.5h-2c-.2,2.8-2.3,5-5,5.4h-2Z" />
            </svg>
          </button>
        </div>

        {(sending || isProcessingQueue) && messageQueue.length > 0 && (
          <div className="text-xs text-muted-foreground mt-1 text-center">
            Current task running • {messageQueue.length} queued
          </div>
        )}
      </form>
      </div>
      
      <TaskSidebar onTaskClick={(_taskId) => {}} />
    </div>
  );
}
