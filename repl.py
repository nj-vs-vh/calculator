from calculator.parser import ParserError, parse
from calculator.runtime import evaluate
from calculator.tokenizer import TokenizerError, tokenize


if __name__ == "__main__":
    variables: dict[str, float] = dict()

    while True:
        code = input("> ")

        try:
            tokens = tokenize(code)
        except TokenizerError as e:
            print(e)
            continue

        try:
            expressions = parse(tokens)
        except ParserError as e:
            print(e)
            continue

        try:
            results = evaluate(expressions, variables)
        except Exception as e:
            print(e)
            continue

        print(results[-1])
