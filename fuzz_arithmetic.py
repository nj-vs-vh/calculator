import math
import random
import re
import string
import warnings

from calculator.parser import parse
from calculator.runtime import evaluate
from calculator.tokenizer import tokenize

warnings.filterwarnings("ignore")


def eval_py(code: str) -> float | str:
    try:
        return eval(code)
    except Exception as e:
        return str(e)


def eval_my(code: str) -> float | str:
    try:
        return evaluate(parse(tokenize(code)))[0]
    except Exception as e:
        return str(e)


if __name__ == "__main__":
    alphabet = string.digits + ".()+-*/ "

    def generate(length: int) -> str:
        return "".join(random.choices(alphabet, k=length))

    while True:
        code = generate(10)

        if re.findall(r"\*\s*\*", code):
            continue  # avoid generating powers (10**4)

        if re.findall(r"/\s*/", code):
            continue  # avoid generating int devision (10 // 3)

        res_py = eval_py(code)
        res_my = eval_my(code)
        if isinstance(res_py, float) and isinstance(res_my, float) and math.isclose(res_my, res_py):
            continue
        if isinstance(res_py, int) and isinstance(res_my, float) and math.isclose(float(res_py), res_my):
            continue
        if isinstance(res_py, str) and isinstance(res_my, str):
            continue
        if isinstance(res_py, str) and res_py.startswith("leading zeros in decimal integer literals are not permitted"):
            continue
        print(f"{code!r}\npy: {res_py}\nmy: {res_my}\n\n")
