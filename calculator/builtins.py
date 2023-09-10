import math
from typing import Callable, Optional

from calculator.value import BuiltinFunc, Float, UnaryOperationImpl, Value

BUILTIN_FUNCS: dict[str, BuiltinFunc] = dict()


def register_builtin_func(name: str):
    def decorator(fn: Callable[[Value], Optional[Value]]) -> UnaryOperationImpl:
        def decorated(arg: Value) -> Value:
            maybe_res = fn(arg)
            if maybe_res is None:
                raise TypeError(f"{name!r} is not defined for argument of type {arg.type_name()}")
            else:
                return maybe_res

        BUILTIN_FUNCS[name] = BuiltinFunc(name=name, fn=decorated)
        return decorated

    return decorator


@register_builtin_func("log")
def log_(arg: Value) -> Value | None:
    if isinstance(arg, Float):
        return Float(math.log(arg.v))
    else:
        return None


@register_builtin_func("exp")
def exp_(arg: Value) -> Value | None:
    if isinstance(arg, Float):
        return Float(math.exp(arg.v))
    else:
        return None


@register_builtin_func("print")
def print_(arg: Value) -> Value:
    if isinstance(arg, Float):
        print(arg.v)
    elif isinstance(arg, BuiltinFunc):
        print(f"Built-in func {arg.name!r}")
    else:
        print(arg)
    return Float(0.0)
