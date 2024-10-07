
"""
    test1.py: a test file to test context generation.
    
    Token   Possible Contexts (or None)
    =====   ===========================
    ":"     "annotation", "dict", "complex-slice", "simple-slice"
    "="     "annotation", "initializer"
    "*"     "arg"
    "**"    "arg"
    "."     "import"
"""

class TestClass:

    def test_ops(self, arg1, *args, **kwargs):
        a = 2
        print(a)
