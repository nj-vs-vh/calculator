from calculator.parser import ParserError, parse
from calculator.runtime import evaluate
from calculator.tokenizer import TokenizerError, tokenize, untokenize

for code in [
    # "5",
    # "-1",
    # "1 + 1",
    # "-1 + 1",
    # "1 + -1",
    # "4 + 6 * 3",
    # "(4 + 6)",
    # "(4+6) * 3",
    # "80225/+2",
    # "7/6/2000",
    "5^2"
    # "a = 1; b2b = 2; c = a + b2b"
]:
    print("=" * 10)
    print(f"code: {code!r}")
    try:
        tokens = tokenize(code)
    except TokenizerError as e:
        print(e.format(code))
        continue

    print(f"tokens: {' '.join(str(t) for t in tokens)}")

    try:
        expression = parse(tokens)[0]
    except ParserError as e:
        print(e)
        continue

    print(f"ast: {expression}")

    result = evaluate([expression])[0]
    print(f"result: {result}")
