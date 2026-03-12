import { useState, useEffect } from 'react';
import { apiPost } from '../../lib/api';

type TierLevel = 'T0' | 'T1' | 'T2' | 'T3';

interface ConfirmationGateProps {
  tier: TierLevel;
  action: string;
  skillName?: string;
  details?: {
    target?: string;
    skill?: string;
    impact?: string;
  };
  consequences?: {
    files_read: string[];
    files_written: string[];
    commands_executed: string[];
    blast_radius: 'minimal' | 'limited' | 'extensive';
    summary: string;
  };
  onConfirm: (confirmationText?: string, totpCode?: string) => void;
  onCancel: () => void;
}

const TIER_CONFIG: Record<TierLevel, { label: string; description: string; color: string; requiresTyping: boolean; requiresTotp: boolean }> = {
  T0: { label: 'Read-only', description: 'This action only reads data', color: 'green', requiresTyping: false, requiresTotp: false },
  T1: { label: 'Tap to Confirm', description: 'This action will write files to your workspace', color: 'indigo', requiresTyping: false, requiresTotp: false },
  T2: { label: 'Type to Confirm', description: 'This action will send messages or make external calls', color: 'orange', requiresTyping: true, requiresTotp: false },
  T3: { label: 'Elevated Action', description: 'This is a destructive operation requiring full verification', color: 'red', requiresTyping: true, requiresTotp: true },
};

export function ConfirmationGate({ tier, action, skillName, details, consequences, onConfirm, onCancel }: ConfirmationGateProps) {
  const [confirmationText, setConfirmationText] = useState('');
  const [totpCode, setTotpCode] = useState('');
  const [totpVerified, setTotpVerified] = useState(false);
  const [totpError, setTotpError] = useState<string | null>(null);
  const [verifying, setVerifying] = useState(false);
  const [countdown, setCountdown] = useState(0);
  const [canConfirm, setCanConfirm] = useState(false);

  const config = TIER_CONFIG[tier];

  useEffect(() => {
    if (tier === 'T3' && totpCode.length === 6 && !totpVerified) {
      verifyTotp();
    }
  }, [totpCode]);

  useEffect(() => {
    if (countdown > 0) {
      const timer = setTimeout(() => setCountdown(countdown - 1), 1000);
      return () => clearTimeout(timer);
    } else if (countdown === 0 && !canConfirm) {
      setCanConfirm(true);
    }
  }, [countdown]);

  const verifyTotp = async () => {
    setVerifying(true);
    setTotpError(null);
    try {
      const response = await apiPost('/api/v1/totp/verify', { token: totpCode });
      if (response.ok) {
        const data = await response.json();
        if (data.valid) {
          setTotpVerified(true);
        }
      } else {
        setTotpError('Invalid code');
        setTotpVerified(false);
      }
    } catch {
      setTotpError('Verification failed');
      setTotpVerified(false);
    } finally {
      setVerifying(false);
    }
  };

  const getColorClasses = (color: string) => {
    const colors: Record<string, { bg: string; border: string; text: string; button: string; buttonHover: string }> = {
      green: { bg: 'bg-green-500/10', border: 'border-green-500', text: 'text-green-400', button: 'bg-green-500', buttonHover: 'hover:bg-green-600' },
      indigo: { bg: 'bg-[#4248f1]/10', border: 'border-[#4248f1]', text: 'text-[#4248f1]', button: 'bg-[#4248f1]', buttonHover: 'hover:bg-[#353bc5]' },
      orange: { bg: 'bg-orange-500/10', border: 'border-orange-500', text: 'text-orange-400', button: 'bg-orange-500', buttonHover: 'hover:bg-orange-600' },
      red: { bg: 'bg-red-500/10', border: 'border-red-500', text: 'text-red-400', button: 'bg-red-500', buttonHover: 'hover:bg-red-600' },
    };
    return colors[color] || colors.indigo;
  };

  const colors = getColorClasses(config.color);

  const canSubmit = () => {
    if (!canConfirm) return false;
    if (config.requiresTyping && confirmationText.toLowerCase() !== action.toLowerCase()) return false;
    if (config.requiresTotp && !totpVerified) return false;
    return true;
  };

  const handleConfirm = () => {
    onConfirm(confirmationText, totpVerified ? totpCode : undefined);
  };

  const startCountdown = () => {
    setCountdown(5);
    setCanConfirm(false);
  };

  return (
    <div className={`border-l-4 ${colors.border} ${colors.bg} rounded-xl p-4 my-4 border border-[var(--color-border)]`}>
      <div className="flex items-start gap-3 mb-3">
        <div className={`w-10 h-10 rounded-full flex items-center justify-center ${colors.bg} ${colors.text}`}>
          {tier === 'T3' ? (
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect><path d="M7 11V7a5 5 0 0 1 10 0v4"></path></svg>
          ) : tier === 'T2' ? (
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"></path><line x1="12" y1="9" x2="12" y2="13"></line><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>
          ) : tier === 'T1' ? (
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path></svg>
          ) : (
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path><circle cx="12" cy="12" r="3"></circle></svg>
          )}
        </div>
        <div>
          <h4 className={`font-semibold ${colors.text}`}>{config.label} Required</h4>
          <p className="text-sm text-[var(--color-text-muted)]">{config.description}</p>
        </div>
      </div>

      <div className="bg-[var(--color-muted)]/30 rounded-lg p-3 mb-4 border border-[var(--color-border)]">
        <p className="font-mono text-sm text-[var(--color-text)]">{action}</p>
        {skillName && (
          <p className="text-xs text-[var(--color-text-muted)] mt-1">Skill: {skillName}</p>
        )}
      </div>

      {details && (
        <div className="text-sm mb-4 space-y-1">
          {details.target && <p><span className="text-[var(--color-text-muted)]">Target:</span> {details.target}</p>}
          {details.skill && <p><span className="text-[var(--color-text-muted)]">Skill:</span> {details.skill}</p>}
          {details.impact && <p><span className="text-[var(--color-text-muted)]">Impact:</span> {details.impact}</p>}
        </div>
      )}

      {consequences && (
        <div className="mb-4 border rounded-xl p-3 bg-[var(--color-muted)]/20 border-[var(--color-border)]">
          <div className="flex items-center justify-between mb-2">
            <h5 className="font-semibold text-sm text-[var(--color-text)]">Consequence Preview</h5>
            <span className={`text-xs px-2 py-0.5 rounded-md ${
              consequences.blast_radius === 'minimal' ? 'bg-green-500/20 text-green-500' :
              consequences.blast_radius === 'limited' ? 'bg-yellow-500/20 text-yellow-500' :
              'bg-red-500/20 text-red-500'
            }`}>
              {consequences.blast_radius} blast radius
            </span>
          </div>
          <p className="text-xs text-[var(--color-text-muted)] mb-2">{consequences.summary}</p>
          
          {consequences.files_read.length > 0 && (
            <div className="text-xs mb-1">
              <span className="text-[var(--color-text-muted)]">Files read:</span>{' '}
              {consequences.files_read.map((f, i) => (
                <span key={i} className="font-mono bg-[var(--color-muted)] px-1 rounded ml-1 text-[var(--color-text)]">{f}</span>
              ))}
            </div>
          )}
          
          {consequences.files_written.length > 0 && (
            <div className="text-xs mb-1">
              <span className="text-[var(--color-text-muted)]">Files written:</span>{' '}
              {consequences.files_written.map((f, i) => (
                <span key={i} className="font-mono bg-[var(--color-muted)] px-1 rounded ml-1 text-[var(--color-text)]">{f}</span>
              ))}
            </div>
          )}
          
          {consequences.commands_executed.length > 0 && (
            <div className="text-xs">
              <span className="text-[var(--color-text-muted)]">Commands:</span>{' '}
              {consequences.commands_executed.map((c, i) => (
                <span key={i} className="font-mono bg-[var(--color-muted)] px-1 rounded ml-1 text-[var(--color-text)]">{c}</span>
              ))}
            </div>
          )}
        </div>
      )}

      {config.requiresTyping && (
        <div className="mb-4">
          <label className="block text-sm font-medium mb-2 text-[var(--color-text)]">
            Type to confirm: <span className="font-mono bg-[var(--color-muted)] px-1 rounded">{action}</span>
          </label>
          <input
            type="text"
            value={confirmationText}
            onChange={(e) => setConfirmationText(e.target.value)}
            placeholder="Type the action to confirm..."
            className="w-full px-3 py-2 rounded-xl border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[#4248f1]/50"
            autoFocus
          />
        </div>
      )}

      {config.requiresTotp && (
        <div className="mb-4">
          <label className="block text-sm font-medium mb-2 text-[var(--color-text)]">
            Enter TOTP code from your authenticator app:
          </label>
          <div className="flex gap-2">
            <input
              type="text"
              value={totpCode}
              onChange={(e) => setTotpCode(e.target.value.replace(/\D/g, '').slice(0, 6))}
              placeholder="000000"
              maxLength={6}
              className="w-32 px-3 py-2 rounded-xl border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[#4248f1]/50 font-mono text-center text-lg tracking-widest"
              autoFocus
            />
            {verifying && <span className="flex items-center text-[var(--color-text-muted)]">Verifying...</span>}
            {totpVerified && <span className="flex items-center text-green-500">✓ Verified</span>}
            {totpError && <span className="flex items-center text-red-500">{totpError}</span>}
          </div>
          <p className="text-xs text-[var(--color-text-muted)] mt-1">
            Use your authenticator app (Google Authenticator, Authy, etc.) to get a 6-digit code.
          </p>
        </div>
      )}

      {config.requiresTotp && countdown > 0 && !canConfirm && (
        <div className="mb-4">
          <button
            onClick={startCountdown}
            className="w-full px-4 py-2 rounded-xl border border-[var(--color-border)] bg-[var(--color-muted)] hover:bg-[var(--color-muted)]/80 flex items-center justify-center gap-2 text-[var(--color-text)]"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10"></circle><polyline points="12 6 12 12 16 14"></polyline></svg>
            <span>Start {countdown > 0 ? `(${countdown}s)` : '5-second delay'}</span>
          </button>
        </div>
      )}

      <div className="flex gap-3">
        <button
          onClick={onCancel}
          className="flex-1 px-4 py-2 rounded-xl border border-[var(--color-border)] hover:bg-[var(--color-muted)] transition-colors text-[var(--color-text)]"
        >
          Cancel
        </button>
        <button
          onClick={handleConfirm}
          disabled={!canSubmit()}
          className={`flex-1 px-4 py-2 rounded-xl text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed ${colors.button} ${colors.buttonHover}`}
        >
          {tier === 'T0' ? 'Continue' : 'Confirm'}
        </button>
      </div>
    </div>
  );
}
