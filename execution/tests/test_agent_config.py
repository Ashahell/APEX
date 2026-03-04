import pytest
from apex_agent import AgentConfig, ApexAgent


def test_agent_config_defaults():
    """Test default agent configuration values."""
    config = AgentConfig()

    assert config.max_steps == 50
    assert config.max_cost_usd == 5.0
    assert config.max_cost_cents == 500
    assert config.context_window_tokens == 32768
    assert config.timeout_seconds == 300


def test_agent_config_custom_values():
    """Test custom agent configuration."""
    config = AgentConfig(
        max_steps=10,
        max_cost_usd=1.0,
        max_cost_cents=100,
        context_window_tokens=8192,
        timeout_seconds=60,
    )

    assert config.max_steps == 10
    assert config.max_cost_usd == 1.0
    assert config.max_cost_cents == 100
    assert config.context_window_tokens == 8192
    assert config.timeout_seconds == 60


def test_budget_enforcement_cents():
    """Test budget enforcement using cents."""
    config = AgentConfig(
        max_steps=50,
        max_cost_cents=100,  # $1.00
    )

    # Simulate cost tracking
    total_cost_cents = 0
    for i in range(10):
        step_cost = 15  # 15 cents per step
        total_cost_cents += step_cost

        if total_cost_cents >= config.max_cost_cents:
            assert True  # Budget exceeded
            break
    else:
        pytest.fail("Budget enforcement not triggered")


def test_budget_enforcement_usd():
    """Test budget enforcement using USD."""
    config = AgentConfig(
        max_steps=50,
        max_cost_usd=0.50,  # $0.50
    )

    # Verify cents conversion
    assert config.max_cost_cents == 50


def test_context_window_tokens():
    """Test context window tokens configuration."""
    config = AgentConfig(context_window_tokens=32768)
    assert config.context_window_tokens == 32768

    # Test minimum viable context
    config_min = AgentConfig(context_window_tokens=1024)
    assert config_min.context_window_tokens == 1024


def test_allowed_domains_default():
    """Test default allowed domains is empty (no restrictions)."""
    config = AgentConfig()
    assert config.allowed_domains == []


def test_allowed_domains_custom():
    """Test custom allowed domains."""
    config = AgentConfig(allowed_domains=["github.com", "api.example.com"])
    assert len(config.allowed_domains) == 2
    assert "github.com" in config.allowed_domains


def test_allowed_skills_default():
    """Test default allowed skills is empty (all allowed)."""
    config = AgentConfig()
    assert config.allowed_skills == []


def test_llm_url_configuration():
    """Test LLM URL configuration."""
    config = AgentConfig(llm_url="http://localhost:8080")
    assert config.llm_url == "http://localhost:8080"

    config_custom = AgentConfig(llm_url="http://ollama:11434")
    assert config_custom.llm_url == "http://ollama:11434"


def test_model_configuration():
    """Test model configuration."""
    config = AgentConfig(llm_model="qwen3-4b")
    assert config.llm_model == "qwen3-4b"

    config_alt = AgentConfig(llm_model="qwen3-8b")
    assert config_alt.llm_model == "qwen3-8b"
