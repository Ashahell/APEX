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
  T1: { label: 'Tap to Confirm', description: 'This action will write files to your workspace', color: 'blue', requiresTyping: false, requiresTotp: false },
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
      blue: { bg: 'bg-blue-500/10', border: 'border-blue-500', text: 'text-blue-400', button: 'bg-blue-500', buttonHover: 'hover:bg-blue-600' },
      orange: { bg: 'bg-orange-500/10', border: 'border-orange-500', text: 'text-orange-400', button: 'bg-orange-500', buttonHover: 'hover:bg-orange-600' },
      red: { bg: 'bg-red-500/10', border: 'border-red-500', text: 'text-red-400', button: 'bg-red-500', buttonHover: 'hover:bg-red-600' },
    };
    return colors[color] || colors.blue;
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
    <div className={`border-l-4 ${colors.border} ${colors.bg} rounded-lg p-4 my-4`}>
      <div className="flex items-start gap-3 mb-3">
        <div className={`w-10 h-10 rounded-full flex items-center justify-center ${colors.bg} ${colors.text}`}>
          {tier === 'T3' ? '🔒' : tier === 'T2' ? '⚠️' : tier === 'T1' ? '✏️' : '👁️'}
        </div>
        <div>
          <h4 className={`font-semibold ${colors.text}`}>{config.label} Required</h4>
          <p className="text-sm text-muted-foreground">{config.description}</p>
        </div>
      </div>

      <div className="bg-muted/50 rounded-lg p-3 mb-4">
        <p className="font-mono text-sm">{action}</p>
        {skillName && (
          <p className="text-xs text-muted-foreground mt-1">Skill: {skillName}</p>
        )}
      </div>

      {details && (
        <div className="text-sm mb-4 space-y-1">
          {details.target && <p><span className="text-muted-foreground">Target:</span> {details.target}</p>}
          {details.skill && <p><span className="text-muted-foreground">Skill:</span> {details.skill}</p>}
          {details.impact && <p><span className="text-muted-foreground">Impact:</span> {details.impact}</p>}
        </div>
      )}

      {consequences && (
        <div className="mb-4 border rounded-lg p-3 bg-muted/30">
          <div className="flex items-center justify-between mb-2">
            <h5 className="font-semibold text-sm">Consequence Preview</h5>
            <span className={`text-xs px-2 py-0.5 rounded ${
              consequences.blast_radius === 'minimal' ? 'bg-green-500/20 text-green-500' :
              consequences.blast_radius === 'limited' ? 'bg-yellow-500/20 text-yellow-500' :
              'bg-red-500/20 text-red-500'
            }`}>
              {consequences.blast_radius} blast radius
            </span>
          </div>
          <p className="text-xs text-muted-foreground mb-2">{consequences.summary}</p>
          
          {consequences.files_read.length > 0 && (
            <div className="text-xs mb-1">
              <span className="text-muted-foreground">Files read:</span>{' '}
              {consequences.files_read.map((f, i) => (
                <span key={i} className="font-mono bg-muted px-1 rounded ml-1">{f}</span>
              ))}
            </div>
          )}
          
          {consequences.files_written.length > 0 && (
            <div className="text-xs mb-1">
              <span className="text-muted-foreground">Files written:</span>{' '}
              {consequences.files_written.map((f, i) => (
                <span key={i} className="font-mono bg-muted px-1 rounded ml-1">{f}</span>
              ))}
            </div>
          )}
          
          {consequences.commands_executed.length > 0 && (
            <div className="text-xs">
              <span className="text-muted-foreground">Commands:</span>{' '}
              {consequences.commands_executed.map((c, i) => (
                <span key={i} className="font-mono bg-muted px-1 rounded ml-1">{c}</span>
              ))}
            </div>
          )}
        </div>
      )}

      {config.requiresTyping && (
        <div className="mb-4">
          <label className="block text-sm font-medium mb-2">
            Type to confirm: <span className="font-mono bg-muted px-1 rounded">{action}</span>
          </label>
          <input
            type="text"
            value={confirmationText}
            onChange={(e) => setConfirmationText(e.target.value)}
            placeholder="Type the action to confirm..."
            className="w-full px-3 py-2 rounded border bg-background focus:outline-none focus:ring-2 focus:ring-primary"
            autoFocus
          />
        </div>
      )}

      {config.requiresTotp && (
        <div className="mb-4">
          <label className="block text-sm font-medium mb-2">
            Enter TOTP code from your authenticator app:
          </label>
          <div className="flex gap-2">
            <input
              type="text"
              value={totpCode}
              onChange={(e) => setTotpCode(e.target.value.replace(/\D/g, '').slice(0, 6))}
              placeholder="000000"
              maxLength={6}
              className="w-32 px-3 py-2 rounded border bg-background focus:outline-none focus:ring-2 focus:ring-primary font-mono text-center text-lg tracking-widest"
              autoFocus
            />
            {verifying && <span className="flex items-center text-muted-foreground">Verifying...</span>}
            {totpVerified && <span className="flex items-center text-green-500">✓ Verified</span>}
            {totpError && <span className="flex items-center text-red-500">{totpError}</span>}
          </div>
          <p className="text-xs text-muted-foreground mt-1">
            Use your authenticator app (Google Authenticator, Authy, etc.) to get a 6-digit code.
          </p>
        </div>
      )}

      {config.requiresTotp && countdown > 0 && !canConfirm && (
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
          className="flex-1 px-4 py-2 rounded border hover:bg-muted transition-colors"
        >
          Cancel
        </button>
        <button
          onClick={handleConfirm}
          disabled={!canSubmit()}
          className={`flex-1 px-4 py-2 rounded text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed ${colors.button} ${colors.buttonHover}`}
        >
          {tier === 'T0' ? 'Continue' : 'Confirm'}
        </button>
      </div>
    </div>
  );
}
