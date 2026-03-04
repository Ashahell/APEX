import { useState } from 'react';

export type ConfirmationTier = 't0' | 't1' | 't2' | 't3';

interface ConfirmationModalProps {
  isOpen: boolean;
  tier: ConfirmationTier;
  action: string;
  onConfirm: (confirmationText?: string) => void;
  onCancel: () => void;
}

const TIER_INFO: Record<ConfirmationTier, { label: string; description: string; requiresTyping: boolean; requiresDelay: boolean }> = {
  t0: { label: 'Read-only', description: 'This action will only read data', requiresTyping: false, requiresDelay: false },
  t1: { label: 'Tap to Confirm', description: 'This action will write files to your workspace', requiresTyping: false, requiresDelay: false },
  t2: { label: 'Type to Confirm', description: 'This action will send messages or make external calls', requiresTyping: true, requiresDelay: false },
  t3: { label: 'TOTP + Delay', description: 'This is a destructive operation that requires extra verification', requiresTyping: true, requiresDelay: true },
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
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={onCancel}>
      <div className="bg-background rounded-lg p-6 max-w-md w-full mx-4 shadow-xl" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center gap-3 mb-4">
          <div className={`w-10 h-10 rounded-full flex items-center justify-center ${
            tier === 't3' ? 'bg-red-100 text-red-600' :
            tier === 't2' ? 'bg-orange-100 text-orange-600' :
            tier === 't1' ? 'bg-blue-100 text-blue-600' :
            'bg-green-100 text-green-600'
          }`}>
            {tier === 't3' ? '🔒' : tier === 't2' ? '⚠️' : tier === 't1' ? '✏️' : '👁️'}
          </div>
          <div>
            <h3 className="text-lg font-semibold">{tierData.label} Required</h3>
            <p className="text-sm text-muted-foreground">{tierData.description}</p>
          </div>
        </div>

        <div className="border rounded-lg p-3 mb-4 bg-muted/50">
          <p className="text-sm font-mono">{action}</p>
        </div>

        {tierData.requiresTyping && (
          <div className="mb-4">
            <label className="block text-sm font-medium mb-2">
              Type to confirm: <span className="font-mono bg-muted px-1 rounded">{action}</span>
            </label>
            <input
              type="text"
              value={confirmationText}
              onChange={(e) => setConfirmationText(e.target.value)}
              placeholder="Type the action to confirm..."
              className="w-full px-3 py-2 rounded border bg-background"
              autoFocus
            />
          </div>
        )}

        {tierData.requiresDelay && !canConfirm && (
          <div className="mb-4">
            <button
              onClick={startCountdown}
              className="w-full px-4 py-2 rounded border bg-muted hover:bg-muted/80 flex items-center justify-center gap-2"
            >
              <span>⏱️</span>
              <span>Start {countdown > 0 ? `(${countdown}s)` : '5-second delay'}</span>
            </button>
          </div>
        )}

        <div className="flex gap-3">
          <button
            onClick={onCancel}
            className="flex-1 px-4 py-2 rounded border hover:bg-muted"
          >
            Cancel
          </button>
          <button
            onClick={handleConfirm}
            disabled={
              (tierData.requiresTyping && confirmationText.toLowerCase() !== action.toLowerCase()) ||
              (tierData.requiresDelay && !canConfirm)
            }
            className="flex-1 px-4 py-2 rounded bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {tier === 't0' ? 'Continue' : 'Confirm'}
          </button>
        </div>
      </div>
    </div>
  );
}
