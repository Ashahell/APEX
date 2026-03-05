import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { ConfirmationModal } from './ConfirmationModal';

describe('ConfirmationModal', () => {
  const mockOnConfirm = vi.fn();
  const mockOnCancel = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders nothing when isOpen is false', () => {
    const { container } = render(
      <ConfirmationModal isOpen={false} tier="t0" action="test" onConfirm={mockOnConfirm} onCancel={mockOnCancel} />
    );
    expect(container.firstChild).toBeNull();
  });

  it('renders modal when isOpen is true', () => {
    render(
      <ConfirmationModal isOpen={true} tier="t0" action="test" onConfirm={mockOnConfirm} onCancel={mockOnCancel} />
    );
    expect(screen.getByText('Read-only Required')).toBeInTheDocument();
  });

  it('displays action text', () => {
    render(
      <ConfirmationModal isOpen={true} tier="t0" action="Delete file" onConfirm={mockOnConfirm} onCancel={mockOnCancel} />
    );
    expect(screen.getByText('Delete file')).toBeInTheDocument();
  });

  it('shows T1 label for tier t1', () => {
    render(
      <ConfirmationModal isOpen={true} tier="t1" action="Write file" onConfirm={mockOnConfirm} onCancel={mockOnCancel} />
    );
    expect(screen.getByText('Tap to Confirm Required')).toBeInTheDocument();
  });

  it('shows T2 label for tier t2', () => {
    render(
      <ConfirmationModal isOpen={true} tier="t2" action="Send message" onConfirm={mockOnConfirm} onCancel={mockOnCancel} />
    );
    expect(screen.getByText('Type to Confirm Required')).toBeInTheDocument();
  });

  it('shows T3 label for tier t3', () => {
    render(
      <ConfirmationModal isOpen={true} tier="t3" action="Delete all" onConfirm={mockOnConfirm} onCancel={mockOnCancel} />
    );
    expect(screen.getByText('TOTP + Delay Required')).toBeInTheDocument();
  });

  it('calls onCancel when Cancel button is clicked', () => {
    render(
      <ConfirmationModal isOpen={true} tier="t0" action="test" onConfirm={mockOnConfirm} onCancel={mockOnCancel} />
    );
    
    fireEvent.click(screen.getByText('Cancel'));
    expect(mockOnCancel).toHaveBeenCalled();
  });

  it('calls onConfirm when Continue button is clicked for t0', () => {
    render(
      <ConfirmationModal isOpen={true} tier="t0" action="Read data" onConfirm={mockOnConfirm} onCancel={mockOnCancel} />
    );
    
    fireEvent.click(screen.getByText('Continue'));
    expect(mockOnConfirm).toHaveBeenCalledWith('');
  });

  it('shows input for t2 tier', () => {
    render(
      <ConfirmationModal isOpen={true} tier="t2" action="Send message" onConfirm={mockOnConfirm} onCancel={mockOnCancel} />
    );
    expect(screen.getByPlaceholderText('Type the action to confirm...')).toBeInTheDocument();
  });
});
