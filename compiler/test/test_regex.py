import pytest

from compiler import compiler, runner

# Regexes taken from https://github.com/mariomka/regex-benchmark
email_regex = r"[\w.+-]+@[\w.-]+\.[\w.-]+"
uri_regex = r"[\w]+://[^/\s?#]+[^\s?#]+(\?[^\s#]*)?(#[^\s]*)?"
ipv4_regex = r"((25[0-5]|2[0-4][0-9]|[01]?[0-9]?[0-9])\.){3}(25[0-5]|2[0-4][0-9]|[01]?[0-9]?[0-9])"

@pytest.mark.parametrize("test_input,expected", [
    ("joe@example.com", "joe@example.com"),
    ("My email is foo@example.com", "foo@example.com"),
    ("bar@example.com is my email", "bar@example.com"),
    ("example.com", None),
    ("foo@example", None)])
def test_email_regex(test_input: str, expected: str | None):
    program = compiler.compile_regex(email_regex)
    match = runner.search(test_input, program)
    assert match == expected

@pytest.mark.parametrize("test_input,expected", [
    ("https://www.example.com", "https://www.example.com"),
    ("https://github.com/search?q=regex&type=repositories", "https://github.com/search?q=regex&type=repositories"),
    ("www.example.com", None),
    ("foo@example.com", None)])
def test_uri_regex(test_input: str, expected: str | None):
    program = compiler.compile_regex(uri_regex)
    match = runner.search(test_input, program)
    assert match == expected

@pytest.mark.parametrize("test_input,expected", [
    ("1.2.3.4", "1.2.3.4"),
    ("255.255.255.255", "255.255.255.255"),
    ("An IP Address: 127.0.0.1", "127.0.0.1"),
    ("0.1.0.1 is an IP address", "0.1.0.1"),
    ("I think [4.3.2.1] is an IP Address", "4.3.2.1"),
    ("256.255.255.255", "56.255.255.255"),
    ("256.255.255.255.255", "255.255.255.255"),
    ("25.321.2.2", None),
    ("25.32..2", None),
    ("a.b.c.d", None)])
def test_ip_regex(test_input: str, expected: str | None):
    program = compiler.compile_regex(ipv4_regex)
    match = runner.search(test_input, program)
    assert match == expected
