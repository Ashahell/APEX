# Prompt Injection Scanner Integration (MVP)

- Purpose: provide an integration point to scan prompts and screen content for push/pull prompt-injection patterns.
- MVP: a simple function that flags a few obvious patterns and can be wired into Hand prompts and UI logs.
- Integration points: Hands prompt generation path, UI display of scan results, and audit trail integration.
