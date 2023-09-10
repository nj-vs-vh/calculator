import math

from calculator.value import BuiltinFunc, Float, UnaryOperationImpl, Value

BUILTIN_FUNCS: dict[str, BuiltinFunc] = dict()


def register_builtin_func(name: str):
    def decorator(fn: UnaryOperationImpl) -> UnaryOperationImpl:
        BUILTIN_FUNCS[name] = BuiltinFunc(fn)

        def decorated(arg: Value) -> Value:
            return fn(arg)

        return decorated

    return decorator


@register_builtin_func("log")
def log(arg: Value) -> Value:
    if isinstance(arg, Float):
        return Float(math.log(arg.v))
    else:
        raise TypeError(f"Log is not defined for {arg.type_name()}")
