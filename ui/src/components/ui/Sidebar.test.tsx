import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render } from '@testing-library/react';
import { Sidebar } from './Sidebar';

describe('Sidebar', () => {
  const mockOnTabChange = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders sidebar component', () => {
    render(<Sidebar activeTab="chat" onTabChange={mockOnTabChange} />);
    // Sidebar should render without crashing
    const aside = document.querySelector('aside');
    expect(aside).toBeInTheDocument();
  });

  it('accepts activeTab and onTabChange props', () => {
    const { container } = render(<Sidebar activeTab="settings" onTabChange={mockOnTabChange} />);
    expect(container).toBeInTheDocument();
  });

  it('renders with collapsed state', () => {
    render(<Sidebar activeTab="chat" onTabChange={mockOnTabChange} collapsed={true} />);
    const aside = document.querySelector('aside');
    expect(aside).toBeInTheDocument();
  });

  it('has navigation buttons', () => {
    render(<Sidebar activeTab="chat" onTabChange={mockOnTabChange} />);
    const buttons = document.querySelectorAll('button');
    expect(buttons.length).toBeGreaterThan(0);
  });
});
