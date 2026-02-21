"""
Test suite for Ogex Python bindings

These tests verify the Python bindings work correctly with the `re`-compatible API.
"""

import pytest


class TestCompile:
    """Test regex compilation"""
    
    def test_compile_simple(self):
        """Test compiling a simple pattern"""
        import ogex
        r = ogex.compile("abc")
        assert r is not None
    
    def test_compile_named_group(self):
        """Test compiling a named group"""
        import ogex
        r = ogex.compile("(name:hello)")
        assert r is not None
    
    def test_compile_invalid(self):
        """Test compiling an invalid pattern raises error"""
        import ogex
        with pytest.raises(ValueError):
            ogex.compile("(unclosed")


class TestMatch:
    """Test match functionality"""
    
    def test_match_at_start(self):
        """Test match at the beginning of string"""
        import ogex
        r = ogex.compile("hello")
        m = r.match_("hello world")
        assert m is not None
        assert m.text() == "hello"
    
    def test_match_not_at_start(self):
        """Test match fails if pattern not at start"""
        import ogex
        r = ogex.compile("world")
        m = r.match_("hello world")
        assert m is None
    
    def test_match_function(self):
        """Test module-level match function"""
        import ogex
        m = ogex.match__("hello", "hello world")
        assert m is not None
        assert m.text() == "hello"


class TestSearch:
    """Test search functionality"""
    
    def test_search_anywhere(self):
        """Test search finds match anywhere"""
        import ogex
        r = ogex.compile("world")
        m = r.search("hello world")
        assert m is not None
        assert m.text() == "world"
    
    def test_search_no_match(self):
        """Test search returns None if no match"""
        import ogex
        r = ogex.compile("xyz")
        m = r.search("hello world")
        assert m is None
    
    def test_search_function(self):
        """Test module-level search function"""
        import ogex
        m = ogex.search("world", "hello world")
        assert m is not None
        assert m.text() == "world"


class TestFindAll:
    """Test findall functionality"""
    
    def test_findall_multiple(self):
        """Test findall returns all matches"""
        import ogex
        r = ogex.compile("a+")
        matches = r.findall("banana")
        assert len(matches) == 3  # a, a, a
    
    def test_findall_no_match(self):
        """Test findall returns empty list"""
        import ogex
        r = ogex.compile("z+")
        matches = r.findall("banana")
        assert len(matches) == 0
    
    def test_findall_function(self):
        """Test module-level findall function"""
        import ogex
        matches = ogex.findall("a+", "banana")
        assert len(matches) == 3


class TestSub:
    """Test substitution functionality"""
    
    def test_sub_simple(self):
        """Test simple substitution"""
        import ogex
        r = ogex.compile("a")
        result = r.sub("X", "banana")
        assert result == "bXnXnX"
    
    def test_sub_with_groups(self):
        """Test substitution with backreferences"""
        import ogex
        r = ogex.compile("(a)(b)")
        result = r.sub(r"\2\1", "ab ab")
        assert result == "ba ba"
    
    def test_sub_entire_match(self):
        """Test substitution with \G (entire match)"""
        import ogex
        r = ogex.compile("hello")
        result = r.sub(r"[\G]", "hello world")
        assert result == "[hello] world"
    
    def test_sub_count(self):
        """Test substitution with count limit"""
        import ogex
        r = ogex.compile("a")
        result = r.sub("X", "banana", count=1)
        assert result == "bXnana"
    
    def test_sub_function(self):
        """Test module-level sub function"""
        import ogex
        result = ogex.sub("a", "X", "banana")
        assert result == "bXnXnX"


class TestNamedGroups:
    """Test named group functionality"""
    
    def test_named_group_syntax(self):
        """Test (name:pattern) syntax"""
        import ogex
        r = ogex.compile("(name:hello)")
        m = r.search("hello world")
        assert m is not None
        assert m.text() == "hello"
    
    def test_named_backref(self):
        """Test \g{name} backreference"""
        import ogex
        r = ogex.compile(r"(word:\w+) is \g{word}")
        m = r.search("test is test")
        assert m is not None


class TestRelativeBackref:
    """Test relative backreference functionality"""
    
    def test_relative_backref_last(self):
        """Test \g{-1} references last numbered group"""
        import ogex
        r = ogex.compile(r"(a)(b)(c)\g{-1}")
        assert r.is_match("abcc")
        assert not r.is_match("abca")
    
    def test_relative_backref_with_named(self):
        """Test relative backrefs exclude named groups"""
        import ogex
        # Named group excluded from relative counting
        # (name:x)(a)(b) - numbered groups are 2 and 3
        # \g{-1} should reference group 3 (b)
        r = ogex.compile(r"(name:x)(a)(b)\g{-1}")
        assert r.is_match("xabb")


class TestMatchObject:
    """Test Match object methods"""
    
    def test_match_start(self):
        """Test match start position"""
        import ogex
        m = ogex.search("world", "hello world")
        assert m.start() == 6
    
    def test_match_end(self):
        """Test match end position"""
        import ogex
        m = ogex.search("world", "hello world")
        assert m.end() == 11
    
    def test_match_text(self):
        """Test match text"""
        import ogex
        m = ogex.search("world", "hello world")
        assert m.text() == "world"
    
    def test_match_group(self):
        """Test match group access"""
        import ogex
        r = ogex.compile("(a)(b)(c)")
        m = r.search("abc")
        assert m.group(1) == "a"
        assert m.group(2) == "b"
        assert m.group(3) == "c"


class TestIsMatch:
    """Test is_match functionality"""
    
    def test_is_match_true(self):
        """Test is_match returns True for matching string"""
        import ogex
        r = ogex.compile("hello")
        assert r.is_match("hello world") is True
    
    def test_is_match_false(self):
        """Test is_match returns False for non-matching string"""
        import ogex
        r = ogex.compile("xyz")
        assert r.is_match("hello world") is False


class TestSpecialSyntax:
    """Test special regex syntax"""
    
    def test_quantifiers(self):
        """Test quantifiers work"""
        import ogex
        assert ogex.compile("a*").is_match("")
        assert ogex.compile("a+").is_match("aaa")
        assert ogex.compile("a?").is_match("")
    
    def test_anchors(self):
        """Test anchors work"""
        import ogex
        assert ogex.compile("^hello").is_match("hello world")
        assert ogex.compile("world$").is_match("hello world")
    
    def test_character_classes(self):
        """Test character classes work"""
        import ogex
        r = ogex.compile("[abc]+")
        assert r.is_match("abcabc")
        assert not r.is_match("xyz")
    
    def test_alternation(self):
        """Test alternation works"""
        import ogex
        r = ogex.compile("cat|dog")
        assert r.is_match("cat")
        assert r.is_match("dog")
        assert not r.is_match("bird")
    
    def test_non_capturing_groups(self):
        """Test non-capturing groups work"""
        import ogex
        r = ogex.compile("(?:hello)(world)")
        m = r.search("helloworld")
        assert m is not None
        assert m.text() == "helloworld"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
