"""
pytest configuration for the ResilientPay simulator.

Disables the broken anchorpy pytest plugin that ships with the global Python
environment on this machine (it requires pytest_xprocess which is not installed).
This conftest.py is the correct place to suppress environment-specific plugin
conflicts without modifying pyproject.toml.
"""
collect_ignore_glob: list[str] = []


def pytest_configure(config):  # type: ignore[no-untyped-def]
    """Suppress known broken global plugins."""
    config.pluginmanager.set_blocked("anchorpy")
