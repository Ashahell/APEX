import { useState } from 'react';
import clsx from 'clsx';

export type AppTab = 
  // Top-level
  | 'chat' | 'board' | 'workflows' | 'settings' | 'theme'
  // Memory
  | 'memory' | 'memoryStats' | 'narrative'
  // Skills
  | 'skills' | 'marketplace' | 'deep'
  // Work
  | 'files' | 'channels' | 'journal' | 'audit' | 'consequences'
  // System
  | 'metrics' | 'monitoring' | 'health' | 'vm'
  // Security
  | 'totp' | 'clients'
  // Integrations
  | 'adapters' | 'webhooks' | 'social'
  // Agent
  | 'soul' | 'autonomy' | 'governance';

interface SidebarItem {
  id: AppTab;
  label: string;
}

interface SidebarGroup {
  id: string;
  label: string;
  icon: string;
  items: SidebarItem[];
  isTopLevel?: boolean;
}

const GROUPS: SidebarGroup[] = [
  // Top-level (always visible)
  {
    id: 'toplevel',
    label: '',
    icon: '',
    isTopLevel: true,
    items: [
      { id: 'chat', label: 'Chat' },
      { id: 'board', label: 'Board' },
      { id: 'workflows', label: 'Workflows' },
      { id: 'settings', label: 'Settings' },
      { id: 'theme', label: 'Theme' },
    ],
  },
  {
    id: 'memory',
    label: 'Memory',
    icon: '░',
    items: [
      { id: 'memory', label: 'Memory' },
      { id: 'memoryStats', label: 'Stats' },
      { id: 'narrative', label: 'Narrative' },
    ],
  },
  {
    id: 'skills',
    label: 'Skills',
    icon: '⚡',
    items: [
      { id: 'skills', label: 'Registry' },
      { id: 'marketplace', label: 'Marketplace' },
      { id: 'deep', label: 'Deep Tasks' },
    ],
  },
  {
    id: 'work',
    label: 'Work',
    icon: '▤',
    items: [
      { id: 'files', label: 'Files' },
      { id: 'channels', label: 'Channels' },
      { id: 'journal', label: 'Journal' },
      { id: 'audit', label: 'Audit' },
      { id: 'consequences', label: 'Preview' },
    ],
  },
  {
    id: 'system',
    label: 'System',
    icon: '▣',
    items: [
      { id: 'metrics', label: 'Metrics' },
      { id: 'monitoring', label: 'Monitor' },
      { id: 'health', label: 'Health' },
      { id: 'vm', label: 'VMs' },
    ],
  },
  {
    id: 'security',
    label: 'Security',
    icon: '§',
    items: [
      { id: 'totp', label: '2FA' },
      { id: 'clients', label: 'Clients' },
    ],
  },
  {
    id: 'integrations',
    label: 'Integrations',
    icon: '◈',
    items: [
      { id: 'adapters', label: 'Adapters' },
      { id: 'webhooks', label: 'Webhooks' },
      { id: 'social', label: 'Social' },
    ],
  },
  {
    id: 'agent',
    label: 'Agent',
    icon: '◆',
    items: [
      { id: 'soul', label: 'Identity' },
      { id: 'autonomy', label: 'Autonomy' },
      { id: 'governance', label: 'Governance' },
    ],
  },
];

const TOP_LEVEL_ICONS: Record<string, string> = {
  chat: '💬',  // Speech bubble
  board: '📋',  // Clipboard
  workflows: '⚙', // Gear
  settings: '⌘',  // Command/Settings
  theme: '◉',   // Circle (prefs)
};

interface SidebarProps {
  activeTab: AppTab;
  onTabChange: (tab: AppTab) => void;
  collapsed?: boolean;
}

export function Sidebar({ activeTab, onTabChange, collapsed = false }: SidebarProps) {
  const [expandedGroup, setExpandedGroup] = useState<string | null>(null);

  const toggleGroup = (groupId: string) => {
    setExpandedGroup(prev => prev === groupId ? null : groupId);
  };

  const topLevelGroup = GROUPS.find(g => g.isTopLevel);
  const submenuGroups = GROUPS.filter(g => !g.isTopLevel);

  const getTopLevelIcon = (tab: AppTab): string => {
    return TOP_LEVEL_ICONS[tab] ?? '•';
  };

  return (
    <>
      <aside className={clsx(
        "hidden md:flex border-r flex-col py-3 gap-1 transition-all duration-200 shrink-0",
        collapsed ? "w-12 items-center" : "w-16 lg:w-20 items-center"
      )}>
        {/* Top-level items */}
        {topLevelGroup?.items.map((item) => (
          <button
            key={item.id}
            onClick={() => onTabChange(item.id)}
            className={clsx(
              "rounded-lg flex items-center justify-center transition-colors",
              collapsed ? "w-10 h-10 text-lg" : "w-12 lg:w-14 h-10 lg:h-12 text-lg",
              activeTab === item.id
                ? "bg-primary text-primary-foreground"
                : "hover:bg-muted"
            )}
            title={item.label}
          >
            {getTopLevelIcon(item.id)}
          </button>
        ))}
        
        <div className="w-8 border-t my-2" />
        
        {/* Grouped submenu items */}
        {submenuGroups.map((group) => (
          <div key={group.id} className="relative">
            <button
              onClick={() => toggleGroup(group.id)}
              className={clsx(
                "rounded-lg flex items-center justify-center transition-colors w-full",
                collapsed ? "w-10 h-10 text-lg" : "w-12 lg:w-14 h-10 lg:h-12 text-lg",
                expandedGroup === group.id
                  ? "bg-primary/20 text-primary"
                  : "hover:bg-muted"
              )}
              title={group.label}
            >
              {group.icon}
            </button>
            
            {/* Expanded submenu */}
            {expandedGroup === group.id && !collapsed && (
              <div className="absolute left-full top-0 ml-1 bg-popover border rounded-lg shadow-lg py-1 min-w-[140px] z-50">
                <div className="px-3 py-1 text-xs font-semibold text-muted-foreground border-b">
                  {group.label}
                </div>
                {group.items.map((item) => (
                  <button
                    key={item.id}
                    onClick={() => {
                      onTabChange(item.id);
                      setExpandedGroup(null);
                    }}
                    className={clsx(
                      "w-full px-3 py-2 text-left text-sm hover:bg-muted flex items-center justify-between",
                      activeTab === item.id && "bg-primary/10 text-primary"
                    )}
                  >
                    <span>{item.label}</span>
                  </button>
                ))}
              </div>
            )}
          </div>
        ))}
      </aside>
      
      {/* Mobile bottom nav - only top level */}
      <nav className="md:hidden fixed bottom-0 left-0 right-0 border-t bg-background flex items-center justify-around py-2 z-50">
        {topLevelGroup?.items.slice(0, 4).map((item) => (
          <button
            key={item.id}
            onClick={() => onTabChange(item.id)}
            className={clsx(
              "flex flex-col items-center justify-center p-2 rounded-lg transition-colors",
              activeTab === item.id
                ? "text-primary"
                : "text-muted-foreground hover:text-foreground"
            )}
          >
            <span className="text-xl">{getTopLevelIcon(item.id)}</span>
            <span className="text-xs mt-1">{item.label}</span>
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
