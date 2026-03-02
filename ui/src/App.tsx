import { useState } from 'react';
import { useAppStore } from './stores/appStore';
import { Chat } from './components/chat/Chat';
import { Skills } from './components/skills/Skills';
import { Files } from './components/files/Files';
import { Settings } from './components/settings/Settings';
import { KanbanBoard } from './components/kanban/KanbanBoard';
import { Sidebar } from './components/ui/Sidebar';

function App() {
  const [activeTab, setActiveTab] = useState<'chat' | 'skills' | 'files' | 'kanban' | 'settings'>('chat');
  const isConnected = useAppStore((s) => s.isConnected);

  return (
    <div className="flex h-screen bg-background">
      <Sidebar activeTab={activeTab} onTabChange={setActiveTab} />
      
      <main className="flex-1 flex flex-col">
        <header className="border-b p-4 flex items-center justify-between">
          <h1 className="text-xl font-semibold">APEX</h1>
          <div className="flex items-center gap-2">
            <span
              className={`w-2 h-2 rounded-full ${
                isConnected ? 'bg-green-500' : 'bg-gray-500'
              }`}
            />
            <span className="text-sm text-muted-foreground">
              {isConnected ? 'Connected' : 'Local Mode'}
            </span>
          </div>
        </header>

        <div className="flex-1 overflow-hidden">
          {activeTab === 'chat' && <Chat />}
          {activeTab === 'skills' && <Skills />}
          {activeTab === 'files' && <Files />}
          {activeTab === 'kanban' && <KanbanBoard />}
          {activeTab === 'settings' && <Settings />}
        </div>
      </main>
    </div>
  );
}

export default App;
