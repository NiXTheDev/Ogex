#!/usr/bin/env python3
"""
Test file for Ogex Python bindings

First install the package:
  cd ogex-python
  maturin develop

Then run:
  python test_ogex.py
"""

import ogex

def test_basic_match():
    """Test basic pattern matching"""
    print("Test 1: Basic match")
    regex = ogex.compile("hello")
    print(f"  'hello world' matches: {regex.is_match('hello world')}")
    print(f"  'goodbye' matches: {regex.is_match('goodbye')}")

def test_named_groups():
    """Test named group capture"""
    print("\nTest 2: Named groups")
    regex = ogex.compile("(name:\\w+)")
    m = regex.search("hello name:John Smith")
    if m:
        print(f"  Match: '{m.text}'")
        print(f"  Named group 'name': '{m.group(1)}'")

def test_relative_backref():
    r"""Test relative backreference \g{-1}"""
    print("\nTest 3: Relative backreference \\g{-1}")
    regex = ogex.compile("(a)(b)\\g{-1}")
    print(f"  'abb' matches: {regex.is_match('abb')}")
    print(f"  'aba' matches: {regex.is_match('aba')}")

def test_g_literal():
    r"""Test \G as literal in pattern"""
    print("\nTest 4: \\G literal in pattern")
    regex = ogex.compile("\\G")
    print(f"  'G' matches: {regex.is_match('G')}")
    print(f"  'g' matches: {regex.is_match('g')}")

def test_findall():
    """Test findall functionality"""
    print("\nTest 5: Find all")
    regex = ogex.compile("\\d+")
    matches = regex.findall("abc 123 def 456 ghi 789")
    print(f"  Found {len(matches)} matches:")
    for i, m in enumerate(matches, 1):
        print(f"    {i}: '{m.text}' at {m.start}-{m.end}")

def test_sub():
    """Test substitution"""
    print("\nTest 6: Substitution")
    regex = ogex.compile("\\d+")
    result = regex.sub("[\\G]", "abc 123 def")
    print(f"  Replace digits with [match]: '{result}'")

if __name__ == "__main__":
    print("=== Ogex Python Tests ===\n")
    
    test_basic_match()
    test_named_groups()
    test_relative_backref()
    test_g_literal()
    test_findall()
    test_sub()
    
    print("\n✅ All tests completed!")
