# Phase 6 Runbook: UI Parity and Theming

## Overview
- Phase 6 aligns UI visuals with AgentZero parity; ensures accessibility and UX polish.
- This runbook provides operational steps for incident response, debugging, and escalation.

## Phase 6 Features

| Feature | Description |
|---------|-------------|
| High Contrast Theme | WCAG AAA compliant theme for accessibility |
| Consolidated Theming | 4 built-in themes with seamless switching |
| Accessibility | ARIA labels, keyboard navigation, focus management |
| UX Polish | Smooth transitions, loading states, hover effects |

---

## Incident Response Procedures

### 1. Theme Application Issues

**Symptoms:**
- Theme not applying after selection
- CSS variables not updating
- Flash of unstyled content

**Immediate Steps:**
```bash
# Check theme is stored in localStorage
# Open browser console:
localStorage.getItem('apex-theme-id')

# Check theme styles are applied
document.getElementById('apex-theme-styles')?.textContent
```

**Debug Commands:**
```bash
# Check theme files
# Review: ui/src/themes/
# Review: ui/src/hooks/useTheme.tsx
```

**Common Causes:**
- localStorage not available (private browsing)
- CSS specificity conflicts
- Theme file syntax errors

**Rollback:** Reset theme to default: `localStorage.setItem('apex-theme-id', 'modern-2026')`

---

### 2. Accessibility Failures

**Symptoms:**
- axe-core audit failures
- Keyboard navigation broken
- Screen reader not announcing elements

**Immediate Steps:**
```bash
# Run axe-core audit
# In browser console:
axe.run().then(results => console.log(results.violations))

# Check focus indicators
# Tab through UI, verify focus-visible outlines
```

**Debug Commands:**
```bash
# Check ARIA labels
# Review: ui/src/components/ for aria-label attributes

# Check keyboard handlers
# Review: onKeyDown, onKeyPress handlers
```

**Common Causes:**
- Missing ARIA labels
- Focus trap issues
- Custom components without keyboard support

**Rollback:** Revert to previous component version, add missing ARIA attributes.

---

### 3. Performance Regressions

**Symptoms:**
- UI jank or stuttering
- Slow theme switching
- High CPU usage during animations

**Immediate Steps:**
```bash
# Check React DevTools Profiler
# Look for components with high render times

# Check Lighthouse performance score
# Run: npx lighthouse http://localhost:5173 --view
```

**Debug Commands:**
```bash
# Check for unnecessary re-renders
# Review: React.memo, useMemo, useCallback usage

# Check animation performance
# Use browser DevTools Performance tab
```

**Common Causes:**
- Missing React.memo on heavy components
- Unnecessary state updates
- Heavy animations on main thread

**Rollback:** Disable animations, optimize component rendering.

---

## Debug Commands Quick Reference

| Command | Purpose |
|---------|---------|
| `localStorage.getItem('apex-theme-id')` | Check current theme |
| `document.getElementById('apex-theme-styles')` | Check applied CSS |
| `axe.run()` | Run accessibility audit |
| Lighthouse | Performance audit |

---

## Test Commands

```bash
# Build UI
cd ui && npm run build

# Run UI tests
cd ui && npm test
```

---

## Escalation Paths

| Issue | First Contact | Escalation |
|-------|--------------|------------|
| Theme issues | @frontend-team | @engineering-ops |
| Accessibility issues | @frontend-team | @ux-team |
| Performance issues | @frontend-team | @backend-team |

---

## Rollback Procedure

If Phase 6 changes cause critical issues:

1. **Reset theme to default:**
   ```javascript
   localStorage.setItem('apex-theme-id', 'modern-2026');
   location.reload();
   ```

2. **Restart UI dev server:**
   ```bash
   cd ui && npm run dev
   ```

3. **Verify recovery:**
   ```bash
   # Check theme applies correctly
   # Verify accessibility basics
   ```

---

## Verification Checklist

After any incident, verify:

- [ ] All 4 themes apply correctly
- [ ] Theme switcher works
- [ ] Keyboard navigation functional
- [ ] Focus indicators visible
- [ ] No performance regressions
- [ ] UI builds successfully

---

## Contacts

- On-call: @engineering-ops
- Frontend UI: @frontend-team
- UX/Accessibility: @ux-team

---

## Last Updated

- Phase 6: UI Parity and Theming
- Version: 1.0
- Date: 2026-03-31
