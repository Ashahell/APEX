# HAND.toml Skeleton (OpenFang Adoption)

version = "0.1.0"
name = "computer-use-hand"
description = "MVP Computer Use Hand (openfang adoption)"
enabled = true

[[plans]]
name = "init"
description = "initial run plan"

[[tools]]
name = "computer-use"
type = "tool"
enabled = true

[[tools]]
name = "browser"
type = "tool"
enabled = true

[[system_prompt]]
text = "You are an autonomous Hand that can run Computer Use tasks, with safety gates."

[[guardrails]]
name = "prompt_injection_scan"
enabled = true

[[example_values]]
name = "computer-use"
enabled = true
