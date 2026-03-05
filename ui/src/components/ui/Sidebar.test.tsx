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

  it('renders sidebar with Skills tab', () => {
    render(<Sidebar activeTab="chat" onTabChange={mockOnTabChange} />);
    
    const skillsButton = screen.getByTitle('Skills');
    expect(skillsButton).toBeInTheDocument();
  });

  it('calls onTabChange when tab is clicked', () => {
    render(<Sidebar activeTab="chat" onTabChange={mockOnTabChange} />);
    
    const skillsButton = screen.getByTitle('Skills');
    fireEvent.click(skillsButton);
    
    expect(mockOnTabChange).toHaveBeenCalledWith('skills');
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

  it('renders multiple navigation items by title', () => {
    render(<Sidebar activeTab="chat" onTabChange={mockOnTabChange} />);
    
    expect(screen.getByTitle('Memory')).toBeInTheDocument();
    expect(screen.getByTitle('Files')).toBeInTheDocument();
    expect(screen.getByTitle('Board')).toBeInTheDocument();
    expect(screen.getByTitle('Workflows')).toBeInTheDocument();
  });

  it('calls onTabChange with different tab ids', () => {
    render(<Sidebar activeTab="chat" onTabChange={mockOnTabChange} />);
    
    fireEvent.click(screen.getByTitle('Settings'));
    expect(mockOnTabChange).toHaveBeenCalledWith('settings');
    
    fireEvent.click(screen.getByTitle('Files'));
    expect(mockOnTabChange).toHaveBeenCalledWith('files');
    
    fireEvent.click(screen.getByTitle('Board'));
    expect(mockOnTabChange).toHaveBeenCalledWith('kanban');
  });
});
