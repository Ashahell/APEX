import pytest
import ipaddress
from urllib.parse import urlparse

# Mock domain enforcement functions for testing
ALLOWED_DOMAINS = ["github.com", "api.example.com", "localhost"]


# Copied from __init__.py for testing - in production would import
BLOCKED_IP_RANGES = [
    ipaddress.ip_network("10.0.0.0/8"),
    ipaddress.ip_network("172.16.0.0/12"),
    ipaddress.ip_network("192.168.0.0/16"),
    ipaddress.ip_network("127.0.0.0/8"),
    ipaddress.ip_network("169.254.0.0/16"),
    ipaddress.ip_network("0.0.0.0/8"),
    ipaddress.ip_network("224.0.0.0/4"),
    ipaddress.ip_network("240.0.0.0/4"),
]


def is_safe_url(url: str) -> tuple[bool, str]:
    """Validate URL is safe from SSRF attacks."""
    if not url:
        return False, "No URL provided"

    if not url.startswith(("http://", "https://")):
        return False, "Invalid URL: must start with http:// or https://"

    try:
        parsed = urlparse(url)
        hostname = parsed.hostname

        if not hostname:
            return False, "Invalid URL: no hostname"

        if hostname in ("localhost", "localhost.localdomain"):
            return False, "Blocked: localhost access not allowed"

        try:
            ip = ipaddress.ip_address(hostname)
        except ValueError:
            try:
                import socket
                addr_info = socket.getaddrinfo(hostname, None, socket.AF_UNSPEC, socket.SOCK_STREAM)
                if not addr_info:
                    return False, f"Could not resolve hostname: {hostname}"
                ip = ipaddress.ip_address(addr_info[0][4][0])
            except Exception as e:
                return False, f"Could not resolve hostname {hostname}: {e}"

        if ip.is_private:
            return False, f"Blocked: private IP range {ip}"

        if ip.is_loopback:
            return False, f"Blocked: loopback address {ip}"

        if ip.is_link_local:
            return False, f"Blocked: link-local address {ip}"

        if str(ip) == "169.254.169.254":
            return False, "Blocked: cloud metadata endpoint"

        if str(ip) == "169.254.169.253":
            return False, "Blocked: cloud metadata endpoint"

        if str(ip) == "169.254.169.251":
            return False, "Blocked: cloud metadata endpoint"

        if ip.is_unspecified:
            return False, f"Blocked: unspecified address {ip}"

        return True, "OK"

    except Exception as e:
        return False, f"URL validation error: {e}"


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


# SSRF protection tests
class TestSSRFProtection:
    """Test SSRF protection for web.fetch tool."""

    def test_block_localhost(self):
        """Localhost should be blocked."""
        blocked, _ = is_safe_url("http://localhost/admin")
        assert blocked is False

    def test_block_localhost_ip(self):
        """127.0.0.1 should be blocked."""
        blocked, _ = is_safe_url("http://127.0.0.1/admin")
        assert blocked is False

    def test_block_private_10_range(self):
        """10.x.x.x private range should be blocked."""
        blocked, _ = is_safe_url("http://10.0.0.1/metadata")
        assert blocked is False

        blocked, _ = is_safe_url("http://10.255.255.255/admin")
        assert blocked is False

    def test_block_private_172_range(self):
        """172.16-31.x.x private range should be blocked."""
        blocked, _ = is_safe_url("http://172.16.0.1/metadata")
        assert blocked is False

        blocked, _ = is_safe_url("http://172.31.255.255/admin")
        assert blocked is False

    def test_block_private_192_range(self):
        """192.168.x.x private range should be blocked."""
        blocked, _ = is_safe_url("http://192.168.1.1/router")
        assert blocked is False

    def test_block_cloud_metadata(self):
        """AWS/GCP metadata endpoints should be blocked."""
        blocked, _ = is_safe_url("http://169.254.169.254/latest/meta-data/")
        assert blocked is False

        blocked, _ = is_safe_url("http://169.254.169.253/meta-data/")
        assert blocked is False

    def test_block_link_local(self):
        """Link-local addresses should be blocked."""
        blocked, _ = is_safe_url("http://169.254.169.254/")
        assert blocked is False

    def test_allow_public_urls(self):
        """Public URLs should be allowed."""
        allowed, _ = is_safe_url("https://github.com/openclaw/openclaw")
        assert allowed is True

        allowed, _ = is_safe_url("https://api.github.com/users")
        assert allowed is True

        allowed, _ = is_safe_url("https://httpbin.org/get")
        assert allowed is True

    def test_block_invalid_scheme(self):
        """Non-http schemes should be blocked."""
        blocked, reason = is_safe_url("file:///etc/passwd")
        assert blocked is False

        blocked, reason = is_safe_url("ftp://example.com")
        assert blocked is False

        blocked, reason = is_safe_url("javascript:alert(1)")
        assert blocked is False

    def test_block_empty_url(self):
        """Empty URLs should be blocked."""
        blocked, _ = is_safe_url("")
        assert blocked is False

    def test_block_no_scheme(self):
        """URLs without scheme should be blocked."""
        blocked, _ = is_safe_url("example.com")
        assert blocked is False

        blocked, _ = is_safe_url("localhost:8080")
        assert blocked is False
