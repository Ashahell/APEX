import { useState, useEffect } from 'react';
import { apiGet, apiPost } from '../../lib/api';

interface AgentProfile {
  id: string;
  name: string;
  description: string | null;
  affiliations: string[];
  reputation_score: number;
  last_seen: string;
}

interface Notification {
  id: string;
  notification_type: string;
  from_agent_id: string;
  message: string;
  created_at: string;
  read: boolean;
}

interface TrustAssessment {
  agent_id: string;
  direct_trust: number;
  web_of_trust: number;
  institutional_vouch: number;
  behavioral_score: number;
  overall_trust: number;
}

export function SocialDashboard() {
  const [connected, setConnected] = useState(false);
  const [profile, setProfile] = useState<AgentProfile | null>(null);
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [agents, setAgents] = useState<AgentProfile[]>([]);
  const [selectedAgent, setSelectedAgent] = useState<AgentProfile | null>(null);
  const [trustAssessment, setTrustAssessment] = useState<TrustAssessment | null>(null);
  const [loading, setLoading] = useState(true);
  const [posting, setPosting] = useState(false);
  const [postContent, setPostContent] = useState('');
  const [searchQuery, setSearchQuery] = useState('');

  useEffect(() => {
    loadDashboard();
  }, []);

  const loadDashboard = async () => {
    setLoading(true);
    try {
      const res = await apiGet('/api/v1/moltbook/status');
      if (res.ok) {
        const data = await res.json();
        setConnected(data.connected);
        if (data.profile) setProfile(data.profile);
        if (data.notifications) setNotifications(data.notifications);
      }
      
      const agentsRes = await apiGet('/api/v1/moltbook/agents');
      if (agentsRes.ok) {
        const agentsData = await agentsRes.json();
        setAgents(agentsData);
      }
    } catch (err) {
      console.error('Failed to load dashboard:', err);
    } finally {
      setLoading(false);
    }
  };

  const connect = async () => {
    try {
      const res = await apiPost('/api/v1/moltbook/connect', {});
      if (res.ok) {
        setConnected(true);
        await loadDashboard();
      }
    } catch (err) {
      console.error('Failed to connect:', err);
    }
  };

  const disconnect = async () => {
    try {
      const res = await apiPost('/api/v1/moltbook/disconnect', {});
      if (res.ok) {
        setConnected(false);
        setProfile(null);
        setNotifications([]);
      }
    } catch (err) {
      console.error('Failed to disconnect:', err);
    }
  };

  const assessTrust = async (agentId: string) => {
    try {
      const res = await apiGet(`/api/v1/moltbook/trust/${agentId}`);
      if (res.ok) {
        const data = await res.json();
        setTrustAssessment(data);
      }
    } catch (err) {
      console.error('Failed to assess trust:', err);
    }
  };

  const selectAgent = async (agent: AgentProfile) => {
    setSelectedAgent(agent);
    await assessTrust(agent.id);
  };

  const createPost = async () => {
    if (!postContent.trim()) return;
    setPosting(true);
    try {
      const res = await apiPost('/api/v1/moltbook/posts', { content: postContent });
      if (res.ok) {
        setPostContent('');
        await loadDashboard();
      }
    } catch (err) {
      console.error('Failed to post:', err);
    } finally {
      setPosting(false);
    }
  };

  const filteredAgents = agents.filter(agent =>
    agent.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    agent.id.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const unreadCount = notifications.filter(n => !n.read).length;

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

  if (!connected) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-6">
        <div className="text-center">
          <div className="w-16 h-16 rounded-2xl bg-[#4248f1]/10 flex items-center justify-center mx-auto mb-4">
            <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="#4248f1" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
              <circle cx="9" cy="7" r="4"></circle>
              <path d="M23 21v-2a4 4 0 0 0-3-3.87"></path>
              <path d="M16 3.13a4 4 0 0 1 0 7.75"></path>
            </svg>
          </div>
          <h2 className="text-2xl font-bold mb-2">Moltbook Social</h2>
          <p className="text-[var(--color-text-muted)] mb-6">
            Connect to the federated agent network
          </p>
        </div>
        <button
          onClick={connect}
          className="px-6 py-3 rounded-xl bg-[#4248f1] text-white hover:bg-[#353bc5] transition-colors flex items-center gap-2"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4"></path>
            <polyline points="10 17 15 12 10 7"></polyline>
            <line x1="15" y1="12" x2="3" y2="12"></line>
          </svg>
          Connect to Moltbook
        </button>
      </div>
    );
  }

  return (
    <div className="flex h-full">
      {/* Left Panel - Agent Directory */}
      <div className="w-80 border-r border-[var(--color-border)] flex flex-col bg-[var(--color-panel)]">
        <div className="p-4 border-b border-[var(--color-border)]">
          <div className="flex items-center justify-between mb-4">
            <h2 className="font-bold">Agent Directory</h2>
            <button
              onClick={disconnect}
              className="text-xs text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
            >
              Disconnect
            </button>
          </div>
          {profile && (
            <div className="border border-[var(--color-border)] rounded-lg p-3 mb-4 bg-[var(--color-muted)]/30">
              <div className="font-medium">{profile.name}</div>
              <div className="text-xs text-[var(--color-text-muted)]">
                Reputation: {(profile.reputation_score * 100).toFixed(0)}%
              </div>
            </div>
          )}
          <input
            type="text"
            placeholder="Search agents..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full px-3 py-2.5 rounded-lg border border-[var(--color-border)] bg-[var(--color-background)] text-sm text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[#4248f1]/50"
          />
        </div>
        <div className="flex-1 overflow-auto">
          {filteredAgents.map((agent) => (
            <button
              key={agent.id}
              onClick={() => selectAgent(agent)}
              className={`w-full p-3 border-b border-[var(--color-border)] text-left hover:bg-[var(--color-muted)]/50 transition-colors ${
                selectedAgent?.id === agent.id ? 'bg-[var(--color-muted)]' : ''
              }`}
            >
              <div className="font-medium">{agent.name}</div>
              <div className="text-xs text-[var(--color-text-muted)]">
                {agent.affiliations.join(', ') || 'No affiliations'}
              </div>
              <div className="text-xs text-[var(--color-text-muted)] mt-1">
                Reputation: {(agent.reputation_score * 100).toFixed(0)}%
              </div>
            </button>
          ))}
        </div>
      </div>

      {/* Right Panel - Feed */}
      <div className="flex-1 flex flex-col bg-[var(--color-background)]">
        <div className="p-4 border-b border-[var(--color-border)]">
          <h2 className="font-bold mb-4 flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M4 11a9 9 0 0 1 9 9"></path>
              <path d="M4 4a16 16 0 0 1 16 16"></path>
              <circle cx="5" cy="19" r="1"></circle>
            </svg>
            Feed
          </h2>
          <div className="flex gap-2">
            <input
              type="text"
              placeholder="Share an update..."
              value={postContent}
              onChange={(e) => setPostContent(e.target.value)}
              className="flex-1 px-3 py-2.5 rounded-lg border border-[var(--color-border)] bg-[var(--color-panel)] text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[#4248f1]/50"
            />
            <button
              onClick={createPost}
              disabled={posting || !postContent.trim()}
              className="px-4 py-2.5 rounded-lg bg-[#4248f1] text-white hover:bg-[#353bc5] transition-colors disabled:opacity-50 flex items-center gap-2"
            >
              {posting ? 'Posting...' : 'Post'}
            </button>
          </div>
        </div>

        <div className="flex-1 overflow-auto p-4 space-y-4">
          {/* Notifications */}
          {notifications.length > 0 && (
            <div>
              <h3 className="font-medium mb-3 flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"></path>
                  <path d="M13.73 21a2 2 0 0 1-3.46 0"></path>
                </svg>
                Notifications {unreadCount > 0 && `(${unreadCount} new)`}
              </h3>
              <div className="space-y-2">
                {notifications.slice(0, 5).map((notif) => (
                  <div
                    key={notif.id}
                    className={`border rounded-lg p-3 ${
                      !notif.read 
                        ? 'bg-[#4248f1]/10 border-[#4248f1]/30' 
                        : 'border-[var(--color-border)] bg-[var(--color-panel)]'
                    }`}
                  >
                    <div className="flex items-center justify-between">
                      <span className="text-sm font-medium">{notif.notification_type}</span>
                      <span className="text-xs text-[var(--color-text-muted)]">
                        {new Date(notif.created_at).toLocaleDateString()}
                      </span>
                    </div>
                    <p className="text-sm text-[var(--color-text-muted)] mt-1">{notif.message}</p>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Trust Assessment */}
          {selectedAgent && trustAssessment && (
            <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
              <h3 className="font-medium mb-3 flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path>
                </svg>
                Trust Assessment: {selectedAgent.name}
              </h3>
              <div className="grid grid-cols-2 gap-3">
                <div className="border border-[var(--color-border)] rounded-lg p-3">
                  <div className="text-xs text-[var(--color-text-muted)]">Direct Trust</div>
                  <div className="text-xl font-bold text-[#4248f1]">
                    {(trustAssessment.direct_trust * 100).toFixed(0)}%
                  </div>
                </div>
                <div className="border border-[var(--color-border)] rounded-lg p-3">
                  <div className="text-xs text-[var(--color-text-muted)]">Web of Trust</div>
                  <div className="text-xl font-bold">
                    {(trustAssessment.web_of_trust * 100).toFixed(0)}%
                  </div>
                </div>
                <div className="border border-[var(--color-border)] rounded-lg p-3">
                  <div className="text-xs text-[var(--color-text-muted)]">Institutional</div>
                  <div className="text-xl font-bold">
                    {(trustAssessment.institutional_vouch * 100).toFixed(0)}%
                  </div>
                </div>
                <div className="border border-[var(--color-border)] rounded-lg p-3">
                  <div className="text-xs text-[var(--color-text-muted)]">Behavioral</div>
                  <div className="text-xl font-bold">
                    {(trustAssessment.behavioral_score * 100).toFixed(0)}%
                  </div>
                </div>
              </div>
              <div className="mt-4 p-3 bg-[#4248f1]/10 rounded-lg border border-[#4248f1]/20">
                <div className="text-sm text-[var(--color-text-muted)]">Overall Trust</div>
                <div className="text-2xl font-bold text-[#4248f1]">
                  {(trustAssessment.overall_trust * 100).toFixed(0)}%
                </div>
              </div>
            </div>
          )}

          {agents.length === 0 && (
            <div className="text-center text-[var(--color-text-muted)] py-8">
              No agents found in directory
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
