// Motion utilities for APEX UI
// Provides smooth animations with reduced-motion accessibility support

import { useEffect, useState } from 'react';

// Transition durations (matching Tailwind defaults)
export const MOTION = {
  // Fast: 150ms - for micro-interactions like button hover
  fast: '150ms',
  // Normal: 200ms - for panel open/close
  normal: '200ms',
  // Slow: 300ms - for page transitions
  slow: '300ms',
  // Slower: 500ms - for dramatic reveals
  slower: '500ms',
} as const;

// Easing curves
export const EASE = {
  // Standard ease
  standard: 'cubic-bezier(0.4, 0, 0.2, 1)',
  // Ease in
  in: 'cubic-bezier(0.4, 0, 1, 1)',
  // Ease out
  out: 'cubic-bezier(0, 0, 0.2, 1)',
  // Bounce
  bounce: 'cubic-bezier(0.68, -0.55, 0.265, 1.55)',
} as const;

// Hook to detect reduced motion preference
export function usePrefersReducedMotion(): boolean {
  const [prefersReducedMotion, setPrefersReducedMotion] = useState(false);

  useEffect(() => {
    // Check for reduced motion preference
    const mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
    setPrefersReducedMotion(mediaQuery.matches);

    // Listen for changes
    const handler = (e: MediaQueryListEvent) => {
      setPrefersReducedMotion(e.matches);
    };

    mediaQuery.addEventListener('change', handler);
    return () => mediaQuery.removeEventListener('change', handler);
  }, []);

  return prefersReducedMotion;
}

// Animation class names for common transitions
export const ANIMATION_CLASSES = {
  // Fade in
  fadeIn: 'transition-opacity duration-200 ease-out',
  fadeInFast: 'transition-opacity duration-150 ease-out',
  fadeInSlow: 'transition-opacity duration-300 ease-out',
  
  // Scale in (for modals, cards)
  scaleIn: 'transition-transform duration-200 ease-out',
  scaleInBounce: 'transition-transform duration-300 cubic-bezier(0.68, -0.55, 0.265, 1.55)',
  
  // Slide in
  slideInRight: 'transition-transform duration-200 ease-out',
  slideInLeft: 'transition-transform duration-200 ease-out',
  slideInUp: 'transition-transform duration-200 ease-out',
  slideInDown: 'transition-transform duration-200 ease-out',
  
  // Hover effects
  hoverScale: 'transition-transform duration-150 ease-out',
  hoverBrightness: 'transition-all duration-150 ease-out',
  
  // Loading spinner
  spin: 'animate-spin',
  
  // Reduced motion alternatives (instant transitions)
  instant: 'transition-none',
} as const;

// Helper to conditionally apply animation based on reduced motion preference
export function getMotionClasses(
  prefersReducedMotion: boolean,
  animatedClasses: string,
  staticClasses: string = ''
): string {
  return prefersReducedMotion ? staticClasses : `${animatedClasses} ${staticClasses}`;
}

// Base classes for animated elements
export const BASE_ANIMATED = 'transition-all duration-200 ease-out';

// Classes that work well with Tailwind for common patterns
export const patterns = {
  // Card hover effect
  cardHover: 'hover:scale-[1.02] hover:shadow-lg',
  
  // Button press effect
  buttonPress: 'active:scale-[0.98]',
  
  // List item stagger (for use with animation delay)
  listItem: 'opacity-0 animate-fade-in',
  
  // Modal entrance
  modalEntrance: 'opacity-0 scale-95 animate-modal-in',
  
  // Panel slide
  panelSlide: 'transform -translate-x-full animate-panel-in',
  
  // Skeleton loading
  skeleton: 'animate-pulse bg-muted',
} as const;