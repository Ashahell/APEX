import clsx from 'clsx';

type AppTab = 'chat' | 'skills' | 'marketplace' | 'consequences' | 'files' | 'kanban' | 'memory' | 'narrative' | 'workflows' | 'audit' | 'channels' | 'journal' | 'soul' | 'social' | 'autonomy' | 'governance' | 'vm' | 'metrics' | 'monitoring' | 'health' | 'totp' | 'clients' | 'deep' | 'settings';

interface SidebarProps {
  activeTab: AppTab;
  onTabChange: (tab: AppTab) => void;
  collapsed?: boolean;
}

const tabs: { id: AppTab; label: string; icon: string }[] = [
  { id: 'chat', label: 'Chat', icon: '💬' },
  { id: 'skills', label: 'Skills', icon: '⚡' },
  { id: 'marketplace', label: 'Market', icon: '🛒' },
  { id: 'consequences', label: 'Preview', icon: '👁️' },
  { id: 'memory', label: 'Memory', icon: '🧠' },
  { id: 'narrative', label: 'Narrative', icon: '📖' },
  { id: 'files', label: 'Files', icon: '📁' },
  { id: 'kanban', label: 'Board', icon: '📋' },
  { id: 'workflows', label: 'Workflows', icon: '🔄' },
  { id: 'deep', label: 'Deep', icon: '🧩' },
  { id: 'audit', label: 'Audit', icon: '📊' },
  { id: 'channels', label: 'Channels', icon: '📢' },
  { id: 'journal', label: 'Journal', icon: '📝' },
  { id: 'vm', label: 'VMs', icon: '🖥️' },
  { id: 'metrics', label: 'Metrics', icon: '📈' },
  { id: 'monitoring', label: 'Monitor', icon: '📊' },
  { id: 'health', label: 'Health', icon: '❤️' },
  { id: 'soul', label: 'SOUL', icon: '🎭' },
  { id: 'social', label: 'Social', icon: '🌐' },
  { id: 'autonomy', label: 'Autonomy', icon: '🤖' },
  { id: 'governance', label: 'Governance', icon: '⚖️' },
  { id: 'totp', label: '2FA', icon: '🔐' },
  { id: 'clients', label: 'Clients', icon: '🔑' },
  { id: 'settings', label: 'Settings', icon: '⚙️' },
];

export function Sidebar({ activeTab, onTabChange, collapsed = false }: SidebarProps) {
  return (
    <>
      <aside className={clsx(
        "hidden md:flex border-r flex-col py-4 gap-2 transition-all duration-200 shrink-0",
        collapsed ? "w-12 items-center" : "w-16 lg:w-20 items-center"
      )}>
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => onTabChange(tab.id)}
            className={clsx(
              "rounded-lg flex items-center justify-center transition-colors",
              collapsed ? "w-10 h-10 text-lg" : "w-12 lg:w-14 h-10 lg:h-12 text-lg",
              activeTab === tab.id
                ? "bg-primary text-primary-foreground"
                : "hover:bg-muted"
            )}
            title={tab.label}
          >
            {tab.icon}
          </button>
        ))}
      </aside>
      
      <nav className="md:hidden fixed bottom-0 left-0 right-0 border-t bg-background flex items-center justify-around py-2 z-50">
        {tabs.slice(0, 5).map((tab) => (
          <button
            key={tab.id}
            onClick={() => onTabChange(tab.id)}
            className={clsx(
              "flex flex-col items-center justify-center p-2 rounded-lg transition-colors",
              activeTab === tab.id
                ? "text-primary"
                : "text-muted-foreground hover:text-foreground"
            )}
          >
            <span className="text-xl">{tab.icon}</span>
            <span className="text-xs mt-1">{tab.label}</span>
          </button>
        ))}
        <button
          onClick={() => onTabChange('settings')}
          className={clsx(
            "flex flex-col items-center justify-center p-2 rounded-lg transition-colors",
            activeTab === 'settings'
              ? "text-primary"
              : "text-muted-foreground hover:text-foreground"
          )}
        >
          <span className="text-xl">⚙️</span>
          <span className="text-xs mt-1">More</span>
        </button>
      </nav>
    </>
  );
}
