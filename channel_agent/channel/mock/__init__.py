# encoding:utf-8
from channel.registry import register_channel


def _factory():
    from channel.mock.mock_channel import MockChannel

    return MockChannel()


register_channel("mock", _factory)
