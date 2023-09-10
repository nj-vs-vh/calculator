import abc
from dataclasses import dataclass
from typing import Callable


class Value(abc.ABC):
    @classmethod
    @abc.abstractmethod
    def type_name(cls) -> str:
        ...


UnaryOperationImpl = Callable[[Value], Value]
BinaryOperationImpl = Callable[[Value, Value], Value]


@dataclass
class Float(Value):
    v: float

    @classmethod
    def type_name(cls) -> str:
        return "Float"


@dataclass
class BuiltinFunc(Value):
    fn: Callable[[Value], Value]

    @classmethod
    def type_name(cls) -> str:
        return "Built-in function"
