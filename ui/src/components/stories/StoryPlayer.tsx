import { useState, useEffect, useRef } from 'react';

// Story types
interface StoryChoice {
  id: string;
  text: string;
  consequence?: string;
}

interface DiceRoll {
  dice: string;
  result: number;
  description: string;
}

interface StoryBeat {
  turn: number;
  narrative: string;
  choices: StoryChoice[];
  dice_rolls: DiceRoll[];
  timestamp: number;
}

interface Story {
  id: string;
  title: string;
  setting: string;
  characters: { name: string; description: string }[];
  location: string;
  inventory: string[];
  turn_count: number;
  available_choices: StoryChoice[];
  created_at: number;
  updated_at: number;
}

interface StoryPlayerProps {
  storyId?: string;
  onBack?: () => void;
}

// Dice types
const DICE_TYPES = [
  { notation: 'd4', sides: 4, icon: '4' },
  { notation: 'd6', sides: 6, icon: '6' },
  { notation: 'd8', sides: 8, icon: '8' },
  { notation: 'd10', sides: 10, icon: '10' },
  { notation: 'd12', sides: 12, icon: '12' },
  { notation: 'd20', sides: 20, icon: '20' },
  { notation: 'd100', sides: 100, icon: '00' },
];

// Setting icons
const SETTING_ICONS: Record<string, string> = {
  fantasy: '⚔️',
  scifi: '🚀',
  horror: '👻',
  mystery: '🔍',
  western: '🤠',
  modern: '🌆',
};

export function StoryPlayer({ storyId, onBack }: StoryPlayerProps) {
  const [story, setStory] = useState<Story | null>(null);
  const [history, setHistory] = useState<StoryBeat[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isRolling, setIsRolling] = useState(false);
  const [lastRoll, setLastRoll] = useState<DiceRoll | null>(null);
  const [showInventory, setShowInventory] = useState(false);
  const [customDiceCount, setCustomDiceCount] = useState(1);
  const [customDiceSides, setCustomDiceSides] = useState(20);
  const [customDiceModifier, setCustomDiceModifier] = useState(0);
  const [inputText, setInputText] = useState('');
  const [showHistory, setShowHistory] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (storyId) {
      loadStory(storyId);
    } else {
      setIsLoading(false);
    }
  }, [storyId]);

  useEffect(() => {
    scrollToBottom();
  }, [history]);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  const loadStory = async (id: string) => {
    setIsLoading(true);
    try {
      const res = await fetch(`/api/v1/stories/${id}`, {
        headers: {
          'X-APEX-Signature': 'dev-signature',
          'X-APEX-Timestamp': Math.floor(Date.now() / 1000).toString(),
        },
      });
      
      if (res.ok) {
        const data = await res.json();
        setStory(data);
      } else {
        // Create sample story for demo
        setStory({
          id: 'demo',
          title: 'The Dragon\'s Lair',
          setting: 'fantasy',
          characters: [{ name: 'Hero', description: 'A brave adventurer' }],
          location: 'Forest Entrance',
          inventory: ['Sword', 'Shield'],
          turn_count: 0,
          available_choices: [
            { id: '1', text: 'Enter the dark forest' },
            { id: '2', text: 'Take the mountain path' },
            { id: '3', text: 'Rest by the river' },
          ],
          created_at: Date.now(),
          updated_at: Date.now(),
        });
      }
    } catch (err) {
      console.error('Failed to load story:', err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleRollDice = async (notation: string) => {
    setIsRolling(true);
    try {
      const res = await fetch(`/api/v1/stories/${story?.id}/roll`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-APEX-Signature': 'dev-signature',
          'X-APEX-Timestamp': Math.floor(Date.now() / 1000).toString(),
        },
        body: JSON.stringify({ dice: notation }),
      });
      
      if (res.ok) {
        const data = await res.json();
        setLastRoll(data);
      } else {
        // Simulate roll for demo
        const match = notation.match(/(\d*)d(\d+)([+-]\d+)?/);
        if (match) {
          const count = parseInt(match[1] || '1');
          const sides = parseInt(match[2]);
          const mod = parseInt(match[3] || '0');
          let total = mod;
          for (let i = 0; i < count; i++) {
            total += Math.floor(Math.random() * sides) + 1;
          }
          setLastRoll({
            dice: notation,
            result: total,
            description: `${count}d${sides}${mod >= 0 ? '+' : ''}${mod || ''} = ${total}`,
          });
        }
      }
    } catch (err) {
      console.error('Roll failed:', err);
    } finally {
      setIsRolling(false);
    }
  };

  const handleCustomRoll = () => {
    const notation = `${customDiceCount}d${customDiceSides}${customDiceModifier >= 0 ? '+' : ''}${customDiceModifier || ''}`;
    handleRollDice(notation);
  };

  const handleChoice = async (choice: StoryChoice) => {
    if (!story) return;
    
    // Add choice to history as a beat
    const beat: StoryBeat = {
      turn: history.length,
      narrative: `You chose: ${choice.text}`,
      choices: [],
      dice_rolls: [],
      timestamp: Date.now(),
    };
    setHistory([...history, beat]);
    
    // In real implementation, this would call the API to advance the story
    // For demo, we'll just show a response
    const responseBeat: StoryBeat = {
      turn: history.length + 1,
      narrative: `The path ahead reveals new possibilities. ${choice.consequence || 'Your adventure continues...'}`,
      choices: generateNewChoices(),
      dice_rolls: [],
      timestamp: Date.now(),
    };
    setHistory(prev => [...prev, responseBeat]);
  };

  const generateNewChoices = (): StoryChoice[] => {
    return [
      { id: crypto.randomUUID(), text: 'Continue forward' },
      { id: crypto.randomUUID(), text: 'Look around carefully' },
      { id: crypto.randomUUID(), text: 'Rest and recover' },
      { id: crypto.randomUUID(), text: 'Check your inventory' },
    ];
  };

  const handleSubmitNarrative = () => {
    if (!inputText.trim() || !story) return;
    
    const beat: StoryBeat = {
      turn: history.length,
      narrative: inputText,
      choices: [],
      dice_rolls: [],
      timestamp: Date.now(),
    };
    setHistory([...history, beat]);
    setInputText('');
    
    // Simulate response
    const responseBeat: StoryBeat = {
      turn: history.length + 1,
      narrative: 'The world responds to your action...',
      choices: generateNewChoices(),
      dice_rolls: [],
      timestamp: Date.now(),
    };
    setHistory(prev => [...prev, responseBeat]);
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="animate-spin w-8 h-8 border-2 border-indigo-500 border-t-transparent rounded-full" />
      </div>
    );
  }

  if (!story) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-center p-8">
        <div className="text-6xl mb-4">📖</div>
        <h2 className="text-xl font-bold text-gray-900 dark:text-gray-100 mb-2">
          No Story Selected
        </h2>
        <p className="text-gray-500 dark:text-gray-400 mb-4">
          Create or select a story to begin your adventure
        </p>
        <button
          onClick={onBack}
          className="px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700"
        >
          Browse Stories
        </button>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full bg-gray-50 dark:bg-gray-900">
      {/* Header */}
      <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-4 py-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <button
              onClick={onBack}
              className="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
            >
              <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
              </svg>
            </button>
            <div>
              <h2 className="text-lg font-bold text-gray-900 dark:text-gray-100">
                {SETTING_ICONS[story.setting] || '📖'} {story.title}
              </h2>
              <p className="text-xs text-gray-500 dark:text-gray-400">
                Turn {story.turn_count} • {story.location}
              </p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={() => setShowInventory(!showInventory)}
              className="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
              title="Inventory"
            >
              <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
              </svg>
            </button>
            <button
              onClick={() => setShowHistory(!showHistory)}
              className="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
              title="History"
            >
              <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </button>
          </div>
        </div>
      </div>

      {/* Inventory Sidebar */}
      {showInventory && (
        <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-4 py-3">
          <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Inventory</h3>
          <div className="flex flex-wrap gap-2">
            {story.inventory.length === 0 ? (
              <span className="text-xs text-gray-400">Empty</span>
            ) : (
              story.inventory.map((item, idx) => (
                <span key={idx} className="px-2 py-1 bg-gray-100 dark:bg-gray-700 rounded text-xs">
                  {item}
                </span>
              ))
            )}
          </div>
        </div>
      )}

      {/* Main Content */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {/* Story beats */}
        {history.map((beat, idx) => (
          <div key={idx} className="space-y-2">
            {beat.dice_rolls.length > 0 && (
              <div className="flex gap-2">
                {beat.dice_rolls.map((roll, rIdx) => (
                  <div key={rIdx} className="px-3 py-2 bg-purple-100 dark:bg-purple-900/30 rounded-lg">
                    <span className="font-mono text-purple-700 dark:text-purple-300">
                      🎲 {roll.description}
                    </span>
                  </div>
                ))}
              </div>
            )}
            <div className={`p-4 rounded-lg ${
              beat.turn === 0 
                ? 'bg-indigo-100 dark:bg-indigo-900/30 ml-8' 
                : 'bg-white dark:bg-gray-800 mr-8'
            }`}>
              <p className="text-gray-700 dark:text-gray-300 whitespace-pre-wrap">
                {beat.narrative}
              </p>
            </div>
            {beat.choices.length > 0 && idx === history.length - 1 && (
              <div className="ml-8 space-y-2">
                {beat.choices.map(choice => (
                  <button
                    key={choice.id}
                    onClick={() => handleChoice(choice)}
                    className="w-full text-left px-4 py-3 bg-indigo-50 dark:bg-indigo-900/20 hover:bg-indigo-100 dark:hover:bg-indigo-900/40 border border-indigo-200 dark:border-indigo-800 rounded-lg text-sm text-indigo-700 dark:text-indigo-300 transition-colors"
                  >
                    → {choice.text}
                  </button>
                ))}
              </div>
            )}
          </div>
        ))}
        <div ref={messagesEndRef} />
      </div>

      {/* Dice Roller */}
      <div className="bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 p-4 space-y-3">
        {/* Last Roll Result */}
        {lastRoll && (
          <div className="text-center py-2">
            <span className="text-3xl font-bold text-purple-600 dark:text-purple-400">
              {lastRoll.result}
            </span>
            <p className="text-xs text-gray-500 dark:text-gray-400">{lastRoll.description}</p>
          </div>
        )}

        {/* Dice Buttons */}
        <div className="flex flex-wrap gap-2 justify-center">
          {DICE_TYPES.map(dice => (
            <button
              key={dice.notation}
              onClick={() => handleRollDice(dice.notation)}
              disabled={isRolling}
              className="px-3 py-2 bg-purple-100 dark:bg-purple-900/50 hover:bg-purple-200 dark:hover:bg-purple-900/70 rounded-lg text-sm font-medium text-purple-700 dark:text-purple-300 disabled:opacity-50"
            >
              🎲 {dice.notation}
            </button>
          ))}
        </div>

        {/* Custom Dice */}
        <div className="flex items-center gap-2 justify-center">
          <input
            type="number"
            min="1"
            max="10"
            value={customDiceCount}
            onChange={(e) => setCustomDiceCount(parseInt(e.target.value) || 1)}
            className="w-16 px-2 py-1 text-center text-sm border rounded dark:bg-gray-900 dark:border-gray-700"
          />
          <span className="text-gray-500">d</span>
          <select
            value={customDiceSides}
            onChange={(e) => setCustomDiceSides(parseInt(e.target.value))}
            className="px-2 py-1 text-sm border rounded dark:bg-gray-900 dark:border-gray-700"
          >
            <option value="4">4</option>
            <option value="6">6</option>
            <option value="8">8</option>
            <option value="10">10</option>
            <option value="12">12</option>
            <option value="20">20</option>
            <option value="100">100</option>
          </select>
          <span className="text-gray-500">+</span>
          <input
            type="number"
            value={customDiceModifier}
            onChange={(e) => setCustomDiceModifier(parseInt(e.target.value) || 0)}
            className="w-16 px-2 py-1 text-center text-sm border rounded dark:bg-gray-900 dark:border-gray-700"
          />
          <button
            onClick={handleCustomRoll}
            disabled={isRolling}
            className="px-3 py-1 bg-purple-600 text-white text-sm rounded hover:bg-purple-700 disabled:opacity-50"
          >
            Roll
          </button>
        </div>

        {/* Text Input */}
        <div className="flex gap-2">
          <input
            type="text"
            value={inputText}
            onChange={(e) => setInputText(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleSubmitNarrative()}
            placeholder="Describe your action..."
            className="flex-1 px-4 py-2 border rounded-lg dark:bg-gray-900 dark:border-gray-700 focus:ring-indigo-500 focus:border-indigo-500"
          />
          <button
            onClick={handleSubmitNarrative}
            disabled={!inputText.trim()}
            className="px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 disabled:opacity-50"
          >
            Send
          </button>
        </div>
      </div>
    </div>
  );
}
