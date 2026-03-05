import { useState, useEffect, useCallback } from 'react';
import { useAppStore } from './stores/appStore';
import { useTheme } from './hooks/useTheme';
import { Chat } from './components/chat/Chat';
import { Skills } from './components/skills/Skills';
import { SkillMarketplace } from './components/skills/SkillMarketplace';
import { ConsequenceViewer } from './components/chat/ConsequenceViewer';
import { Files } from './components/files/Files';
import { Settings } from './components/settings/Settings';
import { KanbanBoard } from './components/kanban/KanbanBoard';
import { MemoryViewer } from './components/memory/MemoryViewer';
import { NarrativeMemoryViewer } from './components/memory/NarrativeMemoryViewer';
import { MemoryStatsDashboard } from './components/memory/MemoryStatsDashboard';
import { Workflows } from './components/workflows/Workflows';
import { AuditLog } from './components/audit/AuditLog';
import { ChannelManager } from './components/channels/ChannelManager';
import { AdapterManager } from './components/channels/AdapterManager';
import { WebhookManager } from './components/integrations/WebhookManager';
import { DecisionJournal } from './components/journal/DecisionJournal';
import { SoulEditor } from './components/soul/SoulEditor';
import { SocialDashboard } from './components/social/SocialDashboard';
import { AutonomyControls } from './components/autonomy/AutonomyControls';
import { GovernanceControls } from './components/autonomy/GovernanceControls';
import { DeepTaskPanel } from './components/deep/DeepTaskPanel';
import { VmPoolDashboard } from './components/vm/VmPoolDashboard';
import { MetricsPanel } from './components/metrics/MetricsPanel';
import { MonitoringDashboard } from './components/metrics/MonitoringDashboard';
import { SystemHealthPanel } from './components/metrics/SystemHealthPanel';
import { TotpSetup } from './components/auth/TotpSetup';
import { ClientAuthManager } from './components/auth/ClientAuthManager';
import { Sidebar } from './components/ui/Sidebar';
import { NotificationBell } from './components/ui/NotificationBell';
import { SkillQuickLaunch } from './components/skills/SkillQuickLaunch';
import { QuickCommandBar } from './components/ui/QuickCommandBar';
import { wsClient } from './lib/websocket';

type AppTab = 'chat' | 'skills' | 'marketplace' | 'consequences' | 'files' | 'kanban' | 'memory' | 'memoryStats' | 'narrative' | 'workflows' | 'audit' | 'channels' | 'journal' | 'soul' | 'social' | 'autonomy' | 'governance' | 'vm' | 'metrics' | 'monitoring' | 'health' | 'totp' | 'clients' | 'deep' | 'settings' | 'adapters' | 'webhooks';

const TAB_ORDER: AppTab[] = ['chat', 'skills', 'marketplace', 'consequences', 'memory', 'memoryStats', 'narrative', 'files', 'kanban', 'workflows', 'audit', 'channels', 'adapters', 'webhooks', 'journal', 'deep', 'vm', 'metrics', 'monitoring', 'health', 'soul', 'social', 'autonomy', 'governance', 'totp', 'clients', 'settings'];

function App() {
  const [activeTab, setActiveTab] = useState<AppTab>('chat');
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const { theme, toggleTheme } = useTheme();
  
  const { connectionState, sessionCost, tasks } = useAppStore((s) => ({
    connectionState: s.connectionState,
    sessionCost: s.sessionCost,
    tasks: s.tasks,
  }));

  const runningTasks = tasks.filter(t => t.status === 'running').length;

  useEffect(() => {
    wsClient.connect();
    return () => wsClient.disconnect();
  }, []);

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
      return;
    }
    
    if (e.key === 'Escape') {
      return;
    }
    
    if (e.ctrlKey || e.metaKey) {
      const num = parseInt(e.key);
      if (num >= 1 && num <= 10) {
        setActiveTab(TAB_ORDER[num - 1]);
        return;
      }
      
      switch (e.key.toLowerCase()) {
        case 'b':
          setSidebarCollapsed(prev => !prev);
          break;
      }
    }
  }, []);

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  const getConnectionDisplay = (): { color: string; text: string } => {
    switch (connectionState) {
      case 'connected':
        return { color: 'bg-green-500', text: 'Connected' };
      case 'degraded':
        return { color: 'bg-amber-500', text: 'Degraded' };
      case 'disconnected':
        return { color: 'bg-red-500', text: 'Disconnected' };
    }
  };

  const conn = getConnectionDisplay();

  return (
    <div className="flex h-screen bg-background">
      <Sidebar 
        activeTab={activeTab} 
        onTabChange={setActiveTab}
        collapsed={sidebarCollapsed}
      />
      
      <main className="flex-1 flex flex-col min-w-0 pb-16 md:pb-0">
        <header className="border-b p-4 flex items-center justify-between bg-card shrink-0">
          <div className="flex items-center gap-4">
            <button
              onClick={() => setSidebarCollapsed(prev => !prev)}
              className="lg:hidden p-2 hover:bg-muted rounded-lg"
            >
              ☰
            </button>
            <h1 className="text-xl font-semibold hidden sm:block">APEX</h1>
            {runningTasks > 0 && (
              <span className="px-2 py-1 bg-primary/10 text-primary text-sm rounded-full">
                {runningTasks} task{runningTasks !== 1 ? 's' : ''} running
              </span>
            )}
          </div>
          
          <div className="flex items-center gap-2 sm:gap-6">
            <SkillQuickLaunch />
            <QuickCommandBar onNavigate={(tab) => setActiveTab(tab as AppTab)} onOpenSettings={() => setActiveTab('settings')} />
            <button 
              className="flex items-center gap-2 hover:bg-muted px-2 sm:px-3 py-1.5 rounded-lg transition-colors"
              onClick={() => setActiveTab('kanban')}
            >
              <span className="text-sm font-medium hidden sm:inline">Budget:</span>
              <span className="text-sm font-mono">${sessionCost.toFixed(2)}</span>
            </button>
            
            <button
              onClick={toggleTheme}
              className="p-2 hover:bg-muted rounded-lg transition-colors"
              title={theme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
            >
              {theme === 'dark' ? '☀️' : '🌙'}
            </button>
            
            <NotificationBell />
            
            <div className="flex items-center gap-2">
              <span className={`w-2 h-2 rounded-full ${conn.color} animate-pulse`} />
              <span className="text-sm text-muted-foreground hidden sm:inline">
                {conn.text}
              </span>
            </div>
          </div>
        </header>

        <div className="flex-1 overflow-hidden">
          {activeTab === 'chat' && <Chat />}
          {activeTab === 'skills' && <Skills />}
          {activeTab === 'marketplace' && <SkillMarketplace />}
          {activeTab === 'consequences' && <ConsequenceViewer />}
          {activeTab === 'files' && <Files />}
          {activeTab === 'kanban' && <KanbanBoard />}
          {activeTab === 'memory' && <MemoryViewer />}
          {activeTab === 'memoryStats' && <MemoryStatsDashboard />}
          {activeTab === 'narrative' && <NarrativeMemoryViewer />}
          {activeTab === 'workflows' && <Workflows />}
          {activeTab === 'audit' && <AuditLog />}
          {activeTab === 'channels' && <ChannelManager />}
          {activeTab === 'adapters' && <AdapterManager />}
          {activeTab === 'webhooks' && <WebhookManager />}
          {activeTab === 'journal' && <DecisionJournal />}
          {activeTab === 'deep' && <DeepTaskPanel />}
          {activeTab === 'vm' && <VmPoolDashboard />}
          {activeTab === 'metrics' && <MetricsPanel />}
          {activeTab === 'monitoring' && <MonitoringDashboard />}
          {activeTab === 'health' && <SystemHealthPanel />}
          {activeTab === 'soul' && <SoulEditor />}
          {activeTab === 'social' && <SocialDashboard />}
          {activeTab === 'autonomy' && <AutonomyControls />}
          {activeTab === 'governance' && <GovernanceControls />}
          {activeTab === 'totp' && <TotpSetup />}
          {activeTab === 'clients' && <ClientAuthManager />}
          {activeTab === 'settings' && <Settings />}
        </div>
      </main>
    </div>
  );
}

export default App;
