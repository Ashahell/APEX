import { useState } from 'react';

export type ConfirmationTier = 't0' | 't1' | 't2' | 't3';

interface ConfirmationModalProps {
  isOpen: boolean;
  tier: ConfirmationTier;
  action: string;
  onConfirm: (confirmationText?: string) => void;
  onCancel: () => void;
}

const TIER_INFO: Record<ConfirmationTier, { label: string; description: string; requiresTyping: boolean; requiresDelay: boolean; color: string; bgColor: string }> = {
  t0: { label: 'Read-only', description: 'This action will only read data', requiresTyping: false, requiresDelay: false, color: 'text-green-500', bgColor: 'bg-green-500/10' },
  t1: { label: 'Tap to Confirm', description: 'This action will write files to your workspace', requiresTyping: false, requiresDelay: false, color: 'text-[#4248f1]', bgColor: 'bg-[#4248f1]/10' },
  t2: { label: 'Type to Confirm', description: 'This action will send messages or make external calls', requiresTyping: true, requiresDelay: false, color: 'text-orange-500', bgColor: 'bg-orange-500/10' },
  t3: { label: 'TOTP + Delay', description: 'This is a destructive operation that requires extra verification', requiresTyping: true, requiresDelay: true, color: 'text-red-500', bgColor: 'bg-red-500/10' },
};

export function ConfirmationModal({ isOpen, tier, action, onConfirm, onCancel }: ConfirmationModalProps) {
  const [confirmationText, setConfirmationText] = useState('');
  const [countdown, setCountdown] = useState(0);
  const [canConfirm, setCanConfirm] = useState(false);

  const tierData = TIER_INFO[tier] || TIER_INFO.t0;

  const handleConfirm = () => {
    if (tierData.requiresDelay && countdown > 0) return;
    if (tierData.requiresTyping && confirmationText.toLowerCase() !== action.toLowerCase()) return;
    onConfirm(confirmationText);
  };

  const startCountdown = () => {
    setCountdown(5);
    const interval = setInterval(() => {
      setCountdown((prev) => {
        if (prev <= 1) {
          clearInterval(interval);
          setCanConfirm(true);
          return 0;
        }
        return prev - 1;
      });
    }, 1000);
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 backdrop-blur-sm" onClick={onCancel}>
      <div className="bg-[var(--color-panel)] rounded-xl p-6 max-w-lg w-full mx-4 shadow-2xl border border-[var(--color-border)]" onClick={(e) => e.stopPropagation()}>
        {/* Header with icon */}
        <div className="flex items-center gap-4 mb-6">
          <div className={`w-12 h-12 rounded-full flex items-center justify-center ${tierData.bgColor}`}>
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={tierData.color}>
              {tier === 't3' ? (
                <>
                  <rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
                  <path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
                </>
              ) : tier === 't2' ? (
                <>
                  <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"></path>
                  <line x1="12" y1="9" x2="12" y2="13"></line>
                  <line x1="12" y1="17" x2="12.01" y2="17"></line>
                </>
              ) : tier === 't1' ? (
                <>
                  <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                  <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
                </>
              ) : (
                <>
                  <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path>
                  <circle cx="12" cy="12" r="3"></circle>
                </>
              )}
            </svg>
          </div>
          <div>
            <h3 className="text-lg font-semibold text-[var(--color-text)]">{tierData.label} Required</h3>
            <p className="text-sm text-[var(--color-text-muted)]">{tierData.description}</p>
          </div>
        </div>

        {/* Action preview */}
        <div className="border border-[var(--color-border)] rounded-lg p-4 mb-6 bg-[var(--color-input)]">
          <p className="text-sm font-mono text-[var(--color-text)]">{action}</p>
        </div>

        {/* Typing confirmation */}
        {tierData.requiresTyping && (
          <div className="mb-6">
            <label className="block text-sm font-medium mb-2 text-[var(--color-text)]">
              Type to confirm: <span className="font-mono bg-[var(--color-muted)] px-2 py-0.5 rounded ml-1">{action}</span>
            </label>
            <input
              type="text"
              value={confirmationText}
              onChange={(e) => setConfirmationText(e.target.value)}
              placeholder="Type the action to confirm..."
              className="w-full px-4 py-3 rounded-lg border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[#4248f1]/50"
              autoFocus
            />
          </div>
        )}

        {/* Delay countdown */}
        {tierData.requiresDelay && !canConfirm && (
          <div className="mb-6">
            <button
              onClick={startCountdown}
              className="w-full px-4 py-3 rounded-lg border border-[var(--color-border)] bg-[var(--color-muted)] hover:bg-[var(--color-muted)]/80 flex items-center justify-center gap-2 transition-colors"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <circle cx="12" cy="12" r="10"></circle>
                <polyline points="12 6 12 12 16 14"></polyline>
              </svg>
              <span>{countdown > 0 ? `Waiting (${countdown}s)` : 'Start 5-second delay'}</span>
            </button>
          </div>
        )}

        {/* Action buttons */}
        <div className="flex gap-3">
          <button
            onClick={onCancel}
            className="flex-1 px-4 py-3 rounded-lg border border-[var(--color-border)] hover:bg-[var(--color-muted)] text-[var(--color-text)] transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleConfirm}
            disabled={
              (tierData.requiresTyping && confirmationText.toLowerCase() !== action.toLowerCase()) ||
              (tierData.requiresDelay && !canConfirm)
            }
            className="flex-1 px-4 py-3 rounded-lg bg-[#4248f1] text-white hover:bg-[#353bc5] disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-medium"
          >
            {tier === 't0' ? 'Continue' : 'Confirm'}
          </button>
        </div>
      </div>
    </div>
  );
}
