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
        <div className="text-muted-foreground">Loading...</div>
      </div>
    );
  }

  if (!status) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-muted-foreground">Unable to load TOTP status</div>
      </div>
    );
  }

  return (
    <div className="h-full overflow-auto p-4">
      <div className="max-w-xl mx-auto space-y-6">
        <div>
          <h2 className="text-2xl font-bold">Two-Factor Authentication</h2>
          <p className="text-sm text-muted-foreground">
            Secure your account with TOTP-based 2FA
          </p>
        </div>

        {error && (
          <div className="p-3 bg-red-500/20 text-red-500 rounded-lg">
            {error}
          </div>
        )}

        {success && (
          <div className="p-3 bg-green-500/20 text-green-500 rounded-lg">
            {success}
          </div>
        )}

        <div className="border rounded-lg p-4">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="font-semibold">Status</h3>
              <p className="text-sm text-muted-foreground">
                {status.enabled 
                  ? status.verified 
                    ? 'TOTP is enabled and verified' 
                    : 'TOTP is set up but not verified'
                  : 'TOTP is not configured'
                }
              </p>
            </div>
            <span className={`px-3 py-1 rounded-full text-sm ${
              status.enabled && status.verified 
                ? 'bg-green-500/20 text-green-500' 
                : status.enabled 
                ? 'bg-yellow-500/20 text-yellow-500'
                : 'bg-muted text-muted-foreground'
            }`}>
              {status.enabled && status.verified ? 'Active' : status.enabled ? 'Pending' : 'Disabled'}
            </span>
          </div>
        </div>

        {!status.enabled && (
          <div className="border rounded-lg p-4">
            <h3 className="font-semibold mb-3">Setup TOTP</h3>
            <p className="text-sm text-muted-foreground mb-4">
              Generate a TOTP secret and scan the QR code with your authenticator app
              (Google Authenticator, Authy, 1Password, etc.)
            </p>
            <button
              onClick={setupTotp}
              disabled={settingUp}
              className="w-full px-4 py-2 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
            >
              {settingUp ? 'Generating...' : 'Generate Secret'}
            </button>
          </div>
        )}

        {status.enabled && !status.verified && status.secret && (
          <div className="border rounded-lg p-4 space-y-4">
            <h3 className="font-semibold">Verify Setup</h3>
            
            <div className="flex justify-center">
              {status.qr_code && (
                <img 
                  src={status.qr_code} 
                  alt="TOTP QR Code" 
                  className="border rounded-lg"
                />
              )}
            </div>

            <div className="text-center">
              <p className="text-sm text-muted-foreground mb-2">Or enter this secret manually:</p>
              <code className="text-xs bg-muted px-2 py-1 rounded">{status.secret}</code>
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
                className="w-full px-3 py-2 rounded-lg border bg-background font-mono text-center text-lg tracking-widest"
                maxLength={6}
              />
            </div>

            <button
              onClick={verifyTotp}
              disabled={verifying || code.length !== 6}
              className="w-full px-4 py-2 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
            >
              {verifying ? 'Verifying...' : 'Verify'}
            </button>
          </div>
        )}

        {status.enabled && status.verified && (
          <div className="border rounded-lg p-4">
            <div className="text-center">
              <div className="text-4xl mb-2">✓</div>
              <h3 className="font-semibold">TOTP is Active</h3>
              <p className="text-sm text-muted-foreground mt-2">
                Your account is protected with two-factor authentication
              </p>
            </div>
          </div>
        )}

        <div className="border rounded-lg p-4">
          <h3 className="font-semibold mb-2">What is TOTP?</h3>
          <p className="text-sm text-muted-foreground">
            Time-based One-Time Password (TOTP) is a standard algorithm that generates 
            a 6-digit code that changes every 30 seconds. You'll need to enter this 
            code when performing T3 (highly privileged) actions.
          </p>
        </div>
      </div>
    </div>
  );
}
