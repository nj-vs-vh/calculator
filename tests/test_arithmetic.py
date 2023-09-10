import pytest

from calculator.parser import parse
from calculator.runtime import evaluate
from calculator.tokenizer import tokenize
from calculator.value import Float, Value


@pytest.mark.parametrize(
    "code, expected_ret_val",
    [
        pytest.param("1", Float(1.0)),
        pytest.param("-1", Float(-1.0)),
        pytest.param("1+2", Float(3.0)),
        pytest.param("(1+2)", Float(3.0)),
        pytest.param("-(1+2)", Float(-3.0)),
        pytest.param("(((1)))", Float(1.0)),
        pytest.param("1 * 4 + 5", Float(9.0)),
        pytest.param("1 + 4 * 5", Float(21.0)),
        pytest.param("10 / 5 / 2 / 2", Float(0.5)),
        pytest.param("10 + 2 * (5 + 3 - 1)", Float(24.0)),
        # variables
        pytest.param("a = 1; a", Float(1.0)),
        pytest.param("a = 1; b = 2; a + b", Float(3.0)),
        pytest.param("a = 1; b = 2; c = a + b", Float(3.0)),
        # funcs
        pytest.param("1 > log", Float(0.0)),
        pytest.param("1 > exp > log", Float(1.0)),
    ],
)
def test_eval_arithmetic(code: str, expected_ret_val: Value) -> None:
    tokens = tokenize(code)
    ast = parse(tokens)
    results = evaluate(ast, variables={})
    assert results[-1] == expected_ret_val
