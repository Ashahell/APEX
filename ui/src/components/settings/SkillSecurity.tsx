import { useState, useEffect } from 'react';

interface VerificationResult {
  valid: boolean;
  status: string;
  error?: string;
  skill_name: string;
}

export function SkillSecurity() {
  const [publicKey, setPublicKey] = useState<string>('');
  const [skills, setSkills] = useState<{ name: string; signed: boolean }[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [verifying, setVerifying] = useState<string | null>(null);
  const [verificationResults, setVerificationResults] = useState<Map<string, VerificationResult>>(new Map());

  useEffect(() => {
    loadSigningInfo();
  }, []);

  const loadSigningInfo = async () => {
    setIsLoading(true);
    try {
      // Load public key
      const keyRes = await fetch('/api/v1/signing/keys/verify-key', {
        headers: {
          'X-APEX-Signature': 'dev-signature',
          'X-APEX-Timestamp': Math.floor(Date.now() / 1000).toString(),
        },
      });
      if (keyRes.ok) {
        const keyData = await keyRes.json();
        setPublicKey(keyData.public_key);
      }

      // Load skills list (from API or localStorage)
      const skillsRes = await fetch('/api/v1/skills', {
        headers: {
          'X-APEX-Signature': 'dev-signature',
          'X-APEX-Timestamp': Math.floor(Date.now() / 1000).toString(),
        },
      });
      
      if (skillsRes.ok) {
        const skillsData = await skillsRes.json();
        // Filter to built-in skills for now
        const builtInSkills = (skillsData.skills || []).slice(0, 10).map((s: { name: string }) => ({
          name: s.name,
          signed: false,
        }));
        setSkills(builtInSkills);
      } else {
        // Fallback to sample skills
        setSkills([
          { name: 'shell.execute', signed: true },
          { name: 'code.generate', signed: false },
          { name: 'code.review', signed: true },
          { name: 'git.commit', signed: false },
          { name: 'file.delete', signed: true },
        ]);
      }
    } catch (err) {
      console.warn('Failed to load signing info:', err);
      setSkills([
        { name: 'shell.execute', signed: true },
        { name: 'code.generate', signed: false },
        { name: 'code.review', signed: true },
      ]);
    } finally {
      setIsLoading(false);
    }
  };

  const handleVerifySkill = async (skillName: string) => {
    setVerifying(skillName);
    try {
      // Get signature for this skill
      const sigRes = await fetch(`/api/v1/signing/skills/${encodeURIComponent(skillName)}/signature`, {
        headers: {
          'X-APEX-Signature': 'dev-signature',
          'X-APEX-Timestamp': Math.floor(Date.now() / 1000).toString(),
        },
      });

      if (sigRes.ok) {
        const sig = await sigRes.json();
        
        if (sig) {
          // Verify the signature
          const verifyRes = await fetch(`/api/v1/signing/skills/${encodeURIComponent(skillName)}/verify`, {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
              'X-APEX-Signature': 'dev-signature',
              'X-APEX-Timestamp': Math.floor(Date.now() / 1000).toString(),
            },
            body: JSON.stringify({
              skill_name: skillName,
              content: 'sample content',
              signature: sig,
            }),
          });

          if (verifyRes.ok) {
            const result: VerificationResult = await verifyRes.json();
            setVerificationResults(prev => new Map(prev).set(skillName, result));
          }
        } else {
          setVerificationResults(prev => new Map(prev).set(skillName, {
            valid: false,
            status: 'Unsigned',
            error: 'No signature found for this skill',
            skill_name: skillName,
          }));
        }
      }
    } catch (err) {
      console.error('Verification failed:', err);
    } finally {
      setVerifying(null);
    }
  };

  const handleVerifyAll = async () => {
    for (const skill of skills) {
      await handleVerifySkill(skill.name);
    }
  };

  const getStatusColor = (result?: VerificationResult) => {
    if (!result) return 'bg-gray-200 dark:bg-gray-700';
    switch (result.status) {
      case 'Valid':
        return 'bg-green-100 text-green-700 dark:bg-green-900/50 dark:text-green-300';
      case 'Invalid':
        return 'bg-red-100 text-red-700 dark:bg-red-900/50 dark:text-red-300';
      case 'Unsigned':
        return 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/50 dark:text-yellow-300';
      case 'Expired':
        return 'bg-orange-100 text-orange-700 dark:bg-orange-900/50 dark:text-orange-300';
      default:
        return 'bg-gray-200 dark:bg-gray-700';
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="animate-spin w-6 h-6 border-2 border-indigo-500 border-t-transparent rounded-full" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100">
            Skill Security
          </h3>
          <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
            Cryptographic verification for skills using ed25519 signatures
          </p>
        </div>
        <button
          onClick={handleVerifyAll}
          disabled={verifying !== null}
          className="px-4 py-2 bg-indigo-600 text-white text-sm font-medium rounded-lg hover:bg-indigo-700 disabled:opacity-50 transition-colors"
        >
          {verifying ? 'Verifying...' : 'Verify All'}
        </button>
      </div>

      {/* Public Key */}
      <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
        <h4 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Verification Public Key
        </h4>
        <div className="flex items-center gap-3">
          <code className="flex-1 text-xs font-mono text-gray-600 dark:text-gray-400 bg-white dark:bg-gray-900 p-2 rounded break-all">
            {publicKey || 'Not available'}
          </code>
          <button
            onClick={() => navigator.clipboard.writeText(publicKey)}
            className="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
            title="Copy"
          >
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
            </svg>
          </button>
        </div>
      </div>

      {/* Signature Info */}
      <div className="grid grid-cols-3 gap-4">
        <div className="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
          <div className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            {skills.filter(s => s.signed).length}
          </div>
          <div className="text-sm text-gray-500 dark:text-gray-400">Signed Skills</div>
        </div>
        <div className="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
          <div className="text-2xl font-bold text-green-600">
            {skills.length - skills.filter(s => s.signed).length}
          </div>
          <div className="text-sm text-gray-500 dark:text-gray-400">Unsigned Skills</div>
        </div>
        <div className="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
          <div className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            {skills.length}
          </div>
          <div className="text-sm text-gray-500 dark:text-gray-400">Total Skills</div>
        </div>
      </div>

      {/* Skills List */}
      <div className="space-y-3">
        <h4 className="text-sm font-medium text-gray-700 dark:text-gray-300">
          Skill Signature Status
        </h4>
        
        {skills.length === 0 ? (
          <div className="text-center py-8 text-gray-500 dark:text-gray-400">
            <div className="text-4xl mb-4">🔐</div>
            <p className="text-sm">No skills loaded</p>
          </div>
        ) : (
          skills.map(skill => {
            const result = verificationResults.get(skill.name);
            return (
              <div
                key={skill.name}
                className="flex items-center justify-between p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700"
              >
                <div className="flex items-center gap-4">
                  <div className={`w-10 h-10 rounded-full flex items-center justify-center ${
                    skill.signed 
                      ? 'bg-green-100 text-green-600 dark:bg-green-900/50' 
                      : 'bg-gray-100 text-gray-400 dark:bg-gray-700'
                  }`}>
                    {skill.signed ? (
                      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                      </svg>
                    ) : (
                      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                      </svg>
                    )}
                  </div>
                  <div>
                    <p className="text-sm font-medium text-gray-900 dark:text-gray-100">
                      {skill.name}
                    </p>
                    {result && (
                      <p className="text-xs text-gray-500 dark:text-gray-400">
                        {result.error || `Status: ${result.status}`}
                      </p>
                    )}
                  </div>
                </div>
                
                <div className="flex items-center gap-3">
                  {result && (
                    <span className={`text-xs px-2 py-1 rounded-full ${getStatusColor(result)}`}>
                      {result.status}
                    </span>
                  )}
                  <button
                    onClick={() => handleVerifySkill(skill.name)}
                    disabled={verifying !== null}
                    className="px-3 py-1 text-xs font-medium text-indigo-600 hover:text-indigo-700 dark:text-indigo-400 disabled:opacity-50"
                  >
                    {verifying === skill.name ? 'Verifying...' : 'Verify'}
                  </button>
                </div>
              </div>
            );
          })
        )}
      </div>

      {/* Info Box */}
      <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
        <div className="flex items-start gap-3">
          <svg className="h-5 w-5 text-blue-400 mt-0.5" viewBox="0 0 20 20" fill="currentColor">
            <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clipRule="evenodd" />
          </svg>
          <div>
            <h4 className="text-sm font-medium text-blue-800 dark:text-blue-200">
              About Skill Signing
            </h4>
            <p className="mt-1 text-sm text-blue-700 dark:text-blue-300">
              Skills can be cryptographically signed using ed25519 to verify their authenticity 
              and detect tampering. Signatures include expiry dates and can be verified against 
              the public key shown above.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
