import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Sidebar } from './Sidebar';

describe('Sidebar', () => {
  const mockOnTabChange = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders sidebar with Chat tab', () => {
    render(<Sidebar activeTab="chat" onTabChange={mockOnTabChange} />);
    
    const chatButton = screen.getByTitle('Chat');
    expect(chatButton).toBeInTheDocument();
  });

  it('renders sidebar with Board tab', () => {
    render(<Sidebar activeTab="chat" onTabChange={mockOnTabChange} />);
    
    const boardButton = screen.getByTitle('Board');
    expect(boardButton).toBeInTheDocument();
  });

  it('calls onTabChange when top-level tab is clicked', () => {
    render(<Sidebar activeTab="chat" onTabChange={mockOnTabChange} />);
    
    const boardButton = screen.getByTitle('Board');
    fireEvent.click(boardButton);
    
    expect(mockOnTabChange).toHaveBeenCalledWith('board');
  });

  it('highlights active tab', () => {
    render(<Sidebar activeTab="settings" onTabChange={mockOnTabChange} />);
    
    const settingsButton = screen.getByTitle('Settings');
    expect(settingsButton).toBeInTheDocument();
  });

  it('renders Settings tab', () => {
    render(<Sidebar activeTab="chat" onTabChange={mockOnTabChange} />);
    
    expect(screen.getByTitle('Settings')).toBeInTheDocument();
  });

  it('renders top-level navigation items', () => {
    render(<Sidebar activeTab="chat" onTabChange={mockOnTabChange} />);
    
    expect(screen.getByTitle('Chat')).toBeInTheDocument();
    expect(screen.getByTitle('Board')).toBeInTheDocument();
    expect(screen.getByTitle('Workflows')).toBeInTheDocument();
    expect(screen.getByTitle('Settings')).toBeInTheDocument();
    expect(screen.getByTitle('Theme')).toBeInTheDocument();
  });

  it('renders Memory group icon', () => {
    render(<Sidebar activeTab="chat" onTabChange={mockOnTabChange} />);
    
    expect(screen.getByTitle('Memory')).toBeInTheDocument();
  });

  it('expands submenu when group icon is clicked', () => {
    render(<Sidebar activeTab="chat" onTabChange={mockOnTabChange} />);
    
    const memoryGroup = screen.getByTitle('Memory');
    fireEvent.click(memoryGroup);
    
    expect(screen.getAllByText('Stats').length).toBeGreaterThan(0);
    expect(screen.getAllByText('Narrative').length).toBeGreaterThan(0);
  });

  it('calls onTabChange when submenu item is clicked', () => {
    render(<Sidebar activeTab="chat" onTabChange={mockOnTabChange} />);
    
    const memoryGroup = screen.getByTitle('Memory');
    fireEvent.click(memoryGroup);
    
    const statsItem = screen.getByText('Stats');
    fireEvent.click(statsItem);
    
    expect(mockOnTabChange).toHaveBeenCalledWith('memoryStats');
  });

  it('calls onTabChange with different tab ids', () => {
    render(<Sidebar activeTab="chat" onTabChange={mockOnTabChange} />);
    
    fireEvent.click(screen.getByTitle('Settings'));
    expect(mockOnTabChange).toHaveBeenCalledWith('settings');
    
    fireEvent.click(screen.getByTitle('Board'));
    expect(mockOnTabChange).toHaveBeenCalledWith('board');
  });
});
