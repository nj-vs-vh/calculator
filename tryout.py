from calculator.parser import ParserError, parse
from calculator.runtime import evaluate
from calculator.tokenizer import TokenizerError, tokenize

for code in [
    "5",
    "-1",
    "1 + 1",
    "-1 + 1",
    "1 + -1",
    "4 + 6 * 3",
    "(4 + 6)",
    "(4+6) * 3",
    "80225/+2",
    "7/6/2000",
    "5^2",
    "a = 1; b= 2; c = a + b",
    "var = (1 + 14 * (54^2))",
    "10 / 5/ 2",
    "a = b = 10",
]:
    print("=" * 10)
    print(f"code: {code!r}")
    try:
        tokens = tokenize(code)
    except TokenizerError as e:
        print(e)
        continue

    print(f"tokens: {' '.join(str(t) for t in tokens)}")

    try:
        expressions = parse(tokens)
    except ParserError as e:
        print(e)
        continue
    expressions_str = "\n".join(f" {i + 1:> 2}: {expr}" for i, expr in enumerate(expressions))
    print(f"ast:\n{expressions_str}")

    variables: dict[str, float] = dict()
    results = evaluate(expressions, variables)
    results_str = "\n".join(f" {i + 1:> 2}: {res}" for i, res in enumerate(results))
    print(f"expression results:\n{results_str}")
    print(f"variables: {variables}")
