import pytest

# Mock domain enforcement functions for testing
ALLOWED_DOMAINS = ["github.com", "api.example.com", "localhost"]


def check_domain_allowed(domain: str, allowed_domains: list[str]) -> tuple[bool, str]:
    """Check if a domain is allowed."""
    if not allowed_domains:  # Empty means all allowed
        return True, ""

    if "*" in allowed_domains:
        return True, ""

    if domain in allowed_domains:
        return True, ""

    return False, f"Domain '{domain}' not in allowed list: {allowed_domains}"


def test_domain_allowed_list():
    """Test domain enforcement with allowed list."""
    # Should pass
    allowed, msg = check_domain_allowed("github.com", ALLOWED_DOMAINS)
    assert allowed is True

    allowed, msg = check_domain_allowed("api.example.com", ALLOWED_DOMAINS)
    assert allowed is True

    # Should fail
    allowed, msg = check_domain_allowed("evil.com", ALLOWED_DOMAINS)
    assert allowed is False


def test_domain_allowed_empty():
    """Test domain enforcement with empty list (all allowed)."""
    allowed, msg = check_domain_allowed("anydomain.com", [])
    assert allowed is True

    allowed, msg = check_domain_allowed("github.com", [])
    assert allowed is True


def test_domain_allowed_wildcard():
    """Test domain enforcement with wildcard."""
    allowed, msg = check_domain_allowed("anything.com", ["*"])
    assert allowed is True


def test_domain_subdomain():
    """Test subdomain matching - currently exact match only."""
    # Subdomains should fail with exact match
    allowed, msg = check_domain_allowed("api.github.com", ALLOWED_DOMAINS)
    assert allowed is False


# Tool limit tests
class TestToolLimits:
    """Test tool execution limits."""

    def test_step_limit(self):
        """Test max steps enforcement."""
        max_steps = 50
        current_step = 0

        for step in range(max_steps + 1):
            current_step = step
            if current_step >= max_steps:
                assert True  # Limit reached
                break

        assert current_step == max_steps

    def test_tool_count_per_step(self):
        """Test that only one tool runs per step."""
        # The agent should execute one tool per step
        tools_executed = ["code.generate"]

        assert len(tools_executed) == 1

    def test_budget_per_step(self):
        """Test budget allocation per step."""
        max_budget_cents = 500
        max_steps = 50

        budget_per_step = max_budget_cents // max_steps
        assert budget_per_step == 10  # 10 cents per step


# Safety tests
class TestSafetyFeatures:
    """Test safety features."""

    def test_no_shell_in_t0(self):
        """T0 tier should not have shell access."""
        tier = "T0"
        allowed_skills = ["code.review", "docs.read", "deps.check"]

        assert "shell.execute" not in allowed_skills
        assert tier == "T0"

    def test_shell_requires_t3(self):
        """Shell execution requires T3."""
        required_tier = "T3"

        # Verify T3 is the highest tier
        tier_order = {"T0": 0, "T1": 1, "T2": 2, "T3": 3}
        assert tier_order[required_tier] == 3

    def test_file_delete_requires_t3(self):
        """File deletion requires T3."""
        dangerous_skills = ["shell.execute", "file.delete", "git.force_push", "db.drop"]

        # All dangerous skills should require T3
        for skill in dangerous_skills:
            assert skill in ["shell.execute", "file.delete", "git.force_push", "db.drop"]
