HAND.toml - MVP sample for the Computer Use Hand

version = "0.1.0"
name = "computer-use-hand"
description = "MVP Computer Use Hand for APEX/OpenFang adoption"
enabled = true

[[plans]]
name = "init"
description = "Initial plan for the Hand Runner lifecycle"

[[tools]]
name = "computer-use"
type = "tool"
enabled = true

[[tools]]
name = "browser"
type = "tool"
enabled = true

[[system_prompt]]
text = "You are an autonomous Hand that can execute Computer Use tasks with safety and auditability."

[[guardrails]]
name = "prompt_injection_scan"
enabled = true
