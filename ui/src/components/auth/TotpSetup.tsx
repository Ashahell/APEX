import { useState, useEffect } from 'react';
import { apiGet, apiPost } from '../../lib/api';

interface TotpStatus {
  enabled: boolean;
  verified: boolean;
  secret?: string;
  qr_code?: string;
}

export function TotpSetup() {
  const [status, setStatus] = useState<TotpStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [settingUp, setSettingUp] = useState(false);
  const [verifying, setVerifying] = useState(false);
  const [code, setCode] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  useEffect(() => {
    loadStatus();
  }, []);

  const loadStatus = async () => {
    try {
      const res = await apiGet('/api/v1/totp/status');
      if (res.ok) {
        const data = await res.json();
        setStatus(data);
      }
    } catch (err) {
      console.error('Failed to load TOTP status:', err);
    } finally {
      setLoading(false);
    }
  };

  const setupTotp = async () => {
    setSettingUp(true);
    setError(null);
    try {
      const res = await apiPost('/api/v1/totp/setup', {});
      if (res.ok) {
        const data = await res.json();
        setStatus(data);
        setSuccess('TOTP secret generated');
      } else {
        setError('Failed to setup TOTP');
      }
    } catch (err) {
      setError('Failed to setup TOTP');
    } finally {
      setSettingUp(false);
    }
  };

  const verifyTotp = async () => {
    if (!code || code.length !== 6) {
      setError('Please enter a 6-digit code');
      return;
    }
    setVerifying(true);
    setError(null);
    try {
      const res = await apiPost('/api/v1/totp/verify', { code });
      if (res.ok) {
        setSuccess('TOTP verified successfully');
        setCode('');
        await loadStatus();
      } else {
        setError('Invalid code - please try again');
      }
    } catch (err) {
      setError('Verification failed');
    } finally {
      setVerifying(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-[var(--color-text-muted)] flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="animate-spin">
            <line x1="12" y1="2" x2="12" y2="6"></line>
            <line x1="12" y1="18" x2="12" y2="22"></line>
            <line x1="4.93" y1="4.93" x2="7.76" y2="7.76"></line>
            <line x1="16.24" y1="16.24" x2="19.07" y2="19.07"></line>
            <line x1="2" y1="12" x2="6" y2="12"></line>
            <line x1="18" y1="12" x2="22" y2="12"></line>
            <line x1="4.93" y1="19.07" x2="7.76" y2="16.24"></line>
            <line x1="16.24" y1="7.76" x2="19.07" y2="4.93"></line>
          </svg>
          Loading...
        </div>
      </div>
    );
  }

  if (!status) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-[var(--color-text-muted)]">Unable to load TOTP status</div>
      </div>
    );
  }

  return (
    <div className="h-full overflow-auto p-6">
      <div className="max-w-xl mx-auto space-y-6">
        {/* Header */}
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-[#4248f1]/10 flex items-center justify-center">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#4248f1" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
              <path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
            </svg>
          </div>
          <div>
            <h2 className="text-xl font-semibold">Two-Factor Authentication</h2>
            <p className="text-sm text-[var(--color-text-muted)]">
              Secure your account with TOTP-based 2FA
            </p>
          </div>
        </div>

        {/* Notifications */}
        {error && (
          <div className="p-3 bg-red-500/10 text-red-500 rounded-lg border border-red-500/20 flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="15" y1="9" x2="9" y2="15"></line>
              <line x1="9" y1="9" x2="15" y2="15"></line>
            </svg>
            {error}
          </div>
        )}

        {success && (
          <div className="p-3 bg-green-500/10 text-green-500 rounded-lg border border-green-500/20 flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
              <polyline points="22 4 12 14.01 9 11.01"></polyline>
            </svg>
            {success}
          </div>
        )}

        {/* Status Card */}
        <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="font-semibold">Status</h3>
              <p className="text-sm text-[var(--color-text-muted)]">
                {status.enabled 
                  ? status.verified 
                    ? 'TOTP is enabled and verified' 
                    : 'TOTP is set up but not verified'
                  : 'TOTP is not configured'
                }
              </p>
            </div>
            <span className={`px-3 py-1 rounded-full text-sm font-medium ${
              status.enabled && status.verified 
                ? 'bg-green-500/10 text-green-500 border border-green-500/20' 
                : status.enabled 
                ? 'bg-yellow-500/10 text-yellow-500 border border-yellow-500/20'
                : 'bg-[var(--color-muted)] text-[var(--color-text-muted)]'
            }`}>
              {status.enabled && status.verified ? 'Active' : status.enabled ? 'Pending' : 'Disabled'}
            </span>
          </div>
        </div>

        {/* Setup Card */}
        {!status.enabled && (
          <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
            <h3 className="font-semibold mb-3 flex items-center gap-2">
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
                <path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
              </svg>
              Setup TOTP
            </h3>
            <p className="text-sm text-[var(--color-text-muted)] mb-4">
              Generate a TOTP secret and scan the QR code with your authenticator app
              (Google Authenticator, Authy, 1Password, etc.)
            </p>
            <button
              onClick={setupTotp}
              disabled={settingUp}
              className="w-full px-4 py-2.5 rounded-lg bg-[#4248f1] text-white hover:bg-[#353bc5] disabled:opacity-50 transition-colors flex items-center justify-center gap-2"
            >
              {settingUp ? (
                <>
                  <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="animate-spin">
                    <line x1="12" y1="2" x2="12" y2="6"></line>
                    <line x1="12" y1="18" x2="12" y2="22"></line>
                    <line x1="4.93" y1="4.93" x2="7.76" y2="7.76"></line>
                    <line x1="16.24" y1="16.24" x2="19.07" y2="19.07"></line>
                  </svg>
                  Generating...
                </>
              ) : (
                'Generate Secret'
              )}
            </button>
          </div>
        )}

        {/* Verification Card */}
        {status.enabled && !status.verified && status.secret && (
          <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)] space-y-4">
            <h3 className="font-semibold flex items-center gap-2">
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
                <polyline points="22 4 12 14.01 9 11.01"></polyline>
              </svg>
              Verify Setup
            </h3>
            
            <div className="flex justify-center">
              {status.qr_code && (
                <img 
                  src={status.qr_code} 
                  alt="TOTP QR Code" 
                  className="border border-[var(--color-border)] rounded-lg"
                />
              )}
            </div>

            <div className="text-center">
              <p className="text-sm text-[var(--color-text-muted)] mb-2">Or enter this secret manually:</p>
              <code className="text-xs bg-[var(--color-muted)] px-3 py-1.5 rounded-lg font-mono">{status.secret}</code>
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">
                Enter 6-digit code from your authenticator
              </label>
              <input
                type="text"
                value={code}
                onChange={(e) => setCode(e.target.value.replace(/\D/g, '').slice(0, 6))}
                placeholder="000000"
                className="w-full px-3 py-3 rounded-lg border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] font-mono text-center text-lg tracking-widest"
                maxLength={6}
              />
            </div>

            <button
              onClick={verifyTotp}
              disabled={verifying || code.length !== 6}
              className="w-full px-4 py-2.5 rounded-lg bg-[#4248f1] text-white hover:bg-[#353bc5] disabled:opacity-50 transition-colors flex items-center justify-center gap-2"
            >
              {verifying ? 'Verifying...' : 'Verify'}
            </button>
          </div>
        )}

        {/* Success Card */}
        {status.enabled && status.verified && (
          <div className="border border-green-500/20 rounded-xl p-6 bg-green-500/5">
            <div className="text-center">
              <div className="w-16 h-16 mx-auto mb-3 rounded-full bg-green-500/10 flex items-center justify-center">
                <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="green" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
                  <polyline points="22 4 12 14.01 9 11.01"></polyline>
                </svg>
              </div>
              <h3 className="font-semibold text-lg">TOTP is Active</h3>
              <p className="text-sm text-[var(--color-text-muted)] mt-2">
                Your account is protected with two-factor authentication
              </p>
            </div>
          </div>
        )}

        {/* Info Card */}
        <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
          <h3 className="font-semibold mb-2 flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="12" y1="16" x2="12" y2="12"></line>
              <line x1="12" y1="8" x2="12.01" y2="8"></line>
            </svg>
            What is TOTP?
          </h3>
          <p className="text-sm text-[var(--color-text-muted)]">
            Time-based One-Time Password (TOTP) is a standard algorithm that generates 
            a 6-digit code that changes every 30 seconds. You'll need to enter this 
            code when performing T3 (highly privileged) actions.
          </p>
        </div>
      </div>
    </div>
  );
}
