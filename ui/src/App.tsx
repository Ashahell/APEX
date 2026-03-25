import { useState, useEffect, useCallback, useMemo, lazy, Suspense } from 'react';
import { useAppStore } from './stores/appStore';
import { ThemeProvider, useTheme } from './hooks/useTheme';
import { Chat } from './components/chat/Chat';
import { Sidebar, AppTab } from './components/ui/Sidebar';
import { NotificationBell } from './components/ui/NotificationBell';
import { SkillQuickLaunch } from './components/skills/SkillQuickLaunch';
import { QuickCommandBar } from './components/ui/QuickCommandBar';
import { ToastContainer } from './components/ui/Toast';
import { wsClient } from './lib/websocket';

// Lazy-loaded components for code splitting (reduce initial bundle size)
// Keep Chat eager-loaded as it's the primary view
const DashboardLayout = lazy(() => import('./components/dashboard/DashboardLayout').then(m => ({ default: m.DashboardLayout })));
const Skills = lazy(() => import('./components/skills/Skills').then(m => ({ default: m.Skills })));
const SkillMarketplace = lazy(() => import('./components/skills/SkillMarketplace').then(m => ({ default: m.SkillMarketplace })));
const AutoCreatedSkills = lazy(() => import('./components/skills/AutoCreatedSkills').then(m => ({ default: m.AutoCreatedSkills })));
const ConsequenceViewer = lazy(() => import('./components/chat/ConsequenceViewer').then(m => ({ default: m.ConsequenceViewer })));
const Files = lazy(() => import('./components/files/Files').then(m => ({ default: m.Files })));
const Settings = lazy(() => import('./components/settings/Settings').then(m => ({ default: m.Settings })));
const KanbanBoard = lazy(() => import('./components/kanban/KanbanBoard').then(m => ({ default: m.KanbanBoard })));
const MemoryViewer = lazy(() => import('./components/memory/MemoryViewer').then(m => ({ default: m.MemoryViewer })));
const NarrativeMemoryViewer = lazy(() => import('./components/memory/NarrativeMemoryViewer').then(m => ({ default: m.NarrativeMemoryViewer })));
const MemoryStatsDashboard = lazy(() => import('./components/memory/MemoryStatsDashboard').then(m => ({ default: m.MemoryStatsDashboard })));
const Workflows = lazy(() => import('./components/workflows/Workflows').then(m => ({ default: m.Workflows })));
const AuditLog = lazy(() => import('./components/audit/AuditLog').then(m => ({ default: m.AuditLog })));
const ChannelManager = lazy(() => import('./components/channels/ChannelManager').then(m => ({ default: m.ChannelManager })));
const AdapterManager = lazy(() => import('./components/channels/AdapterManager').then(m => ({ default: m.AdapterManager })));
const WebhookManager = lazy(() => import('./components/integrations/WebhookManager').then(m => ({ default: m.WebhookManager })));
const DecisionJournal = lazy(() => import('./components/journal/DecisionJournal').then(m => ({ default: m.DecisionJournal })));
const SoulEditor = lazy(() => import('./components/soul/SoulEditor').then(m => ({ default: m.SoulEditor })));
const SocialDashboard = lazy(() => import('./components/social/SocialDashboard').then(m => ({ default: m.SocialDashboard })));
const AutonomyControls = lazy(() => import('./components/autonomy/AutonomyControls').then(m => ({ default: m.AutonomyControls })));
const GovernanceControls = lazy(() => import('./components/autonomy/GovernanceControls').then(m => ({ default: m.GovernanceControls })));
const DeepTaskPanel = lazy(() => import('./components/deep/DeepTaskPanel').then(m => ({ default: m.DeepTaskPanel })));
const VmPoolDashboard = lazy(() => import('./components/vm/VmPoolDashboard').then(m => ({ default: m.VmPoolDashboard })));
const MetricsPanel = lazy(() => import('./components/metrics/MetricsPanel').then(m => ({ default: m.MetricsPanel })));
const MonitoringDashboard = lazy(() => import('./components/metrics/MonitoringDashboard').then(m => ({ default: m.MonitoringDashboard })));
const SystemHealthPanel = lazy(() => import('./components/metrics/SystemHealthPanel').then(m => ({ default: m.SystemHealthPanel })));
const TotpSetup = lazy(() => import('./components/auth/TotpSetup').then(m => ({ default: m.TotpSetup })));
const ClientAuthManager = lazy(() => import('./components/auth/ClientAuthManager').then(m => ({ default: m.ClientAuthManager })));
const ThemeEditor = lazy(() => import('./components/settings/ThemeEditor').then(m => ({ default: m.ThemeEditor })));

const TAB_ORDER: AppTab[] = [
  'chat', 'dashboard', 'board', 'workflows', 'settings', 'theme',
  'memory', 'memoryStats', 'narrative',
  'skills', 'marketplace', 'autoCreatedSkills', 'deep',
  'files', 'channels', 'journal', 'audit', 'consequences',
  'metrics', 'monitoring', 'health', 'vm',
  'totp', 'clients',
  'adapters', 'webhooks', 'social',
  'soul', 'autonomy', 'governance'
];

function AppContent() {
  const [activeTab, setActiveTab] = useState<AppTab>('chat');
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const { themeId, toggleTheme } = useTheme();
  
  // Use individual selectors to prevent unnecessary re-renders
  const connectionState = useAppStore(s => s.connectionState);
  const sessionCost = useAppStore(s => s.sessionCost);
  const tasks = useAppStore(s => s.tasks);

  // Memoize expensive computations
  const runningTasks = useMemo(() => 
    tasks.filter(t => t.status === 'running').length, 
    [tasks]
  );

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

  const isAmigaTheme = themeId === 'amiga';
  const conn = getConnectionDisplay();

  const renderContent = () => {
    const fallback = <div className="flex items-center justify-center h-full"><div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div></div>;
    
    switch (activeTab) {
      case 'chat': return <Chat />;
      case 'dashboard': return <Suspense fallback={fallback}><DashboardLayout /></Suspense>;
      case 'board': return <Suspense fallback={fallback}><KanbanBoard /></Suspense>;
      case 'workflows': return <Suspense fallback={fallback}><Workflows /></Suspense>;
      case 'settings': return <Suspense fallback={fallback}><Settings /></Suspense>;
      case 'theme': return <Suspense fallback={fallback}><ThemeEditor /></Suspense>;
      case 'memory': return <Suspense fallback={fallback}><MemoryViewer /></Suspense>;
      case 'memoryStats': return <Suspense fallback={fallback}><MemoryStatsDashboard /></Suspense>;
      case 'narrative': return <Suspense fallback={fallback}><NarrativeMemoryViewer /></Suspense>;
      case 'skills': return <Suspense fallback={fallback}><Skills /></Suspense>;
      case 'marketplace': return <Suspense fallback={fallback}><SkillMarketplace /></Suspense>;
      case 'autoCreatedSkills': return <Suspense fallback={fallback}><AutoCreatedSkills /></Suspense>;
      case 'deep': return <Suspense fallback={fallback}><DeepTaskPanel /></Suspense>;
      case 'files': return <Suspense fallback={fallback}><Files /></Suspense>;
      case 'channels': return <Suspense fallback={fallback}><ChannelManager /></Suspense>;
      case 'journal': return <Suspense fallback={fallback}><DecisionJournal /></Suspense>;
      case 'audit': return <Suspense fallback={fallback}><AuditLog /></Suspense>;
      case 'consequences': return <Suspense fallback={fallback}><ConsequenceViewer /></Suspense>;
      case 'metrics': return <Suspense fallback={fallback}><MetricsPanel /></Suspense>;
      case 'monitoring': return <Suspense fallback={fallback}><MonitoringDashboard /></Suspense>;
      case 'health': return <Suspense fallback={fallback}><SystemHealthPanel /></Suspense>;
      case 'vm': return <Suspense fallback={fallback}><VmPoolDashboard /></Suspense>;
      case 'totp': return <Suspense fallback={fallback}><TotpSetup /></Suspense>;
      case 'clients': return <Suspense fallback={fallback}><ClientAuthManager /></Suspense>;
      case 'adapters': return <Suspense fallback={fallback}><AdapterManager /></Suspense>;
      case 'webhooks': return <Suspense fallback={fallback}><WebhookManager /></Suspense>;
      case 'social': return <Suspense fallback={fallback}><SocialDashboard /></Suspense>;
      case 'soul': return <Suspense fallback={fallback}><SoulEditor /></Suspense>;
      case 'autonomy': return <Suspense fallback={fallback}><AutonomyControls /></Suspense>;
      case 'governance': return <Suspense fallback={fallback}><GovernanceControls /></Suspense>;
      default:
        return <div className="p-6">Select an option from the sidebar</div>;
    }
  };

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
              onClick={() => setActiveTab('board')}
            >
              <span className="text-sm font-medium hidden sm:inline">Budget:</span>
              <span className="text-sm font-mono">${sessionCost.toFixed(2)}</span>
            </button>
            
            <button
              onClick={toggleTheme}
              className="p-2 hover:bg-muted rounded-lg transition-colors"
              title={isAmigaTheme ? 'Switch to Modern theme' : 'Switch to Amiga theme'}
            >
              {isAmigaTheme ? '🖥️' : '🎨'}
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
          {renderContent()}
        </div>
        
        <ToastContainer />
      </main>
    </div>
  );
}

function App() {
  return (
    <ThemeProvider>
      <AppContent />
    </ThemeProvider>
  );
}

export default App;
