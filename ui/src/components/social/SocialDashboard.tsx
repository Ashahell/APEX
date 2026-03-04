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
        <div className="text-muted-foreground">Loading...</div>
      </div>
    );
  }

  if (!connected) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-4">
        <div className="text-center">
          <h2 className="text-xl font-bold mb-2">Moltbook Social</h2>
          <p className="text-muted-foreground mb-4">
            Connect to the federated agent network
          </p>
        </div>
        <button
          onClick={connect}
          className="px-6 py-3 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90"
        >
          Connect to Moltbook
        </button>
      </div>
    );
  }

  return (
    <div className="flex h-full">
      <div className="w-1/3 border-r flex flex-col">
        <div className="p-4 border-b">
          <div className="flex items-center justify-between mb-4">
            <h2 className="font-bold">Agent Directory</h2>
            <button
              onClick={disconnect}
              className="text-xs text-muted-foreground hover:text-foreground"
            >
              Disconnect
            </button>
          </div>
          {profile && (
            <div className="border rounded-lg p-3 mb-4">
              <div className="font-medium">{profile.name}</div>
              <div className="text-xs text-muted-foreground">
                Reputation: {(profile.reputation_score * 100).toFixed(0)}%
              </div>
            </div>
          )}
          <input
            type="text"
            placeholder="Search agents..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full px-3 py-2 rounded-lg border bg-background text-sm"
          />
        </div>
        <div className="flex-1 overflow-auto">
          {filteredAgents.map((agent) => (
            <button
              key={agent.id}
              onClick={() => selectAgent(agent)}
              className={`w-full p-3 border-b text-left hover:bg-muted/50 transition-colors ${
                selectedAgent?.id === agent.id ? 'bg-muted' : ''
              }`}
            >
              <div className="font-medium">{agent.name}</div>
              <div className="text-xs text-muted-foreground">
                {agent.affiliations.join(', ') || 'No affiliations'}
              </div>
              <div className="text-xs text-muted-foreground mt-1">
                Reputation: {(agent.reputation_score * 100).toFixed(0)}%
              </div>
            </button>
          ))}
        </div>
      </div>

      <div className="flex-1 flex flex-col">
        <div className="p-4 border-b">
          <h2 className="font-bold mb-4">Feed</h2>
          <div className="flex gap-2">
            <input
              type="text"
              placeholder="Share an update..."
              value={postContent}
              onChange={(e) => setPostContent(e.target.value)}
              className="flex-1 px-3 py-2 rounded-lg border bg-background"
            />
            <button
              onClick={createPost}
              disabled={posting || !postContent.trim()}
              className="px-4 py-2 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
            >
              {posting ? 'Posting...' : 'Post'}
            </button>
          </div>
        </div>

        <div className="flex-1 overflow-auto p-4">
          {notifications.length > 0 && (
            <div className="mb-4">
              <h3 className="font-medium mb-2">
                Notifications {unreadCount > 0 && `(${unreadCount} new)`}
              </h3>
              <div className="space-y-2">
                {notifications.slice(0, 5).map((notif) => (
                  <div
                    key={notif.id}
                    className={`border rounded-lg p-3 ${!notif.read ? 'bg-blue-500/10 border-blue-500/30' : ''}`}
                  >
                    <div className="flex items-center justify-between">
                      <span className="text-sm font-medium">{notif.notification_type}</span>
                      <span className="text-xs text-muted-foreground">
                        {new Date(notif.created_at).toLocaleDateString()}
                      </span>
                    </div>
                    <p className="text-sm text-muted-foreground mt-1">{notif.message}</p>
                  </div>
                ))}
              </div>
            </div>
          )}

          {selectedAgent && trustAssessment && (
            <div className="border rounded-lg p-4">
              <h3 className="font-medium mb-3">Trust Assessment: {selectedAgent.name}</h3>
              <div className="grid grid-cols-2 gap-4">
                <div className="border rounded p-3">
                  <div className="text-sm text-muted-foreground">Direct Trust</div>
                  <div className="text-xl font-bold">
                    {(trustAssessment.direct_trust * 100).toFixed(0)}%
                  </div>
                </div>
                <div className="border rounded p-3">
                  <div className="text-sm text-muted-foreground">Web of Trust</div>
                  <div className="text-xl font-bold">
                    {(trustAssessment.web_of_trust * 100).toFixed(0)}%
                  </div>
                </div>
                <div className="border rounded p-3">
                  <div className="text-sm text-muted-foreground">Institutional</div>
                  <div className="text-xl font-bold">
                    {(trustAssessment.institutional_vouch * 100).toFixed(0)}%
                  </div>
                </div>
                <div className="border rounded p-3">
                  <div className="text-sm text-muted-foreground">Behavioral</div>
                  <div className="text-xl font-bold">
                    {(trustAssessment.behavioral_score * 100).toFixed(0)}%
                  </div>
                </div>
              </div>
              <div className="mt-4 p-3 bg-primary/10 rounded-lg">
                <div className="text-sm text-muted-foreground">Overall Trust</div>
                <div className="text-2xl font-bold">
                  {(trustAssessment.overall_trust * 100).toFixed(0)}%
                </div>
              </div>
            </div>
          )}

          {agents.length === 0 && (
            <div className="text-center text-muted-foreground py-8">
              No agents found in directory
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
