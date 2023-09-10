import math
from dataclasses import dataclass
from typing import Callable, Type

from calculator.builtins import BUILTIN_FUNCS
from calculator.parser import BinaryOperation, BinaryOperator, Expression, UnaryOperation, UnaryOperator, Variable
from calculator.value import BinaryOperationImpl, BuiltinFunc, Float, UnaryOperationImpl, Value


@dataclass
class CalcRuntimeError(Exception):
    errmsg: str


def evaluate(expressions: list[Expression], variables: dict[str, Value]) -> list[Value]:
    results: list[Value] = []
    for expression in expressions:
        results.append(evaluate_expression(expression, variables))
    return results


def evaluate_expression(expression: Expression, variables: dict[str, Value]) -> Value:
    if isinstance(expression, Value):
        return expression
    elif isinstance(expression, Variable):
        if expression.name in variables:
            return variables[expression.name]
        elif expression.name in BUILTIN_FUNCS:
            return BUILTIN_FUNCS[expression.name]
        else:
            raise RuntimeError(f"Reference to non-eixstend variable {expression.name}")
    elif isinstance(expression, BinaryOperation):
        right_res = evaluate_expression(expression.right, variables)
        if expression.operator == BinaryOperator.ASSIGN:
            if isinstance(expression.left, Variable):
                variables[expression.left.name] = right_res
                return right_res
            else:
                raise RuntimeError("Assigning only works for variables")
        left_res = evaluate_expression(expression.left, variables)
        if expression.operator == BinaryOperator.ADD:
            return eval_binary_operation(table=add_impls, a=left_res, b=right_res, op_name="Addition")
        elif expression.operator == BinaryOperator.SUB:
            return eval_binary_operation(table=sub_impls, a=left_res, b=right_res, op_name="Subtraction")
        elif expression.operator == BinaryOperator.MUL:
            return eval_binary_operation(table=mul_impls, a=left_res, b=right_res, op_name="Multiplication")
        elif expression.operator == BinaryOperator.DIV:
            return eval_binary_operation(table=div_impls, a=left_res, b=right_res, op_name="Division")
        elif expression.operator == BinaryOperator.POW:
            return eval_binary_operation(table=pow_impls, a=left_res, b=right_res, op_name="Power")
        elif expression.operator == BinaryOperator.FEED_TO_FUNC:
            return eval_binary_operation(table=feed_to_func_impls, a=left_res, b=right_res, op_name="Feed to func")
        else:
            raise RuntimeError(f"Unexpected binary operator: {expression.operator}")
    elif isinstance(expression, UnaryOperation):
        operand = evaluate_expression(expression.operand, variables)
        if expression.operator is UnaryOperator.NEG:
            return eval_unary_operation(table=neg_impls, operand=operand, op_name="Negation")
        else:
            raise RuntimeError(f"Unexpected unary operator: {expression.operator}")
    else:
        raise RuntimeError(f"Unexpected expression type: {expression}")


BinaryOperationImplTable = list[tuple[tuple[Type[Value], Type[Value]], BinaryOperationImpl]]


def eval_binary_operation(table: BinaryOperationImplTable, a: Value, b: Value, op_name: str) -> Value:
    for (type_a, type_b), impl in table:
        if isinstance(a, type_a) and isinstance(b, type_b):
            return impl(a, b)
    else:
        raise CalcRuntimeError(f"{op_name} is not defined for {a.type_name()} and {b.type_name()}")


add_impls: BinaryOperationImplTable = [((Float, Float), lambda a, b: Float(a.v + b.v))]  # type: ignore
sub_impls: BinaryOperationImplTable = [((Float, Float), lambda a, b: Float(a.v - b.v))]  # type: ignore
mul_impls: BinaryOperationImplTable = [((Float, Float), lambda a, b: Float(a.v * b.v))]  # type: ignore
div_impls: BinaryOperationImplTable = [((Float, Float), lambda a, b: Float(a.v / b.v))]  # type: ignore
pow_impls: BinaryOperationImplTable = [((Float, Float), lambda a, b: Float(a.v**b.v))]  # type: ignore
feed_to_func_impls: BinaryOperationImplTable = [((Value, BuiltinFunc), lambda a, b: b.fn(a))]  # type: ignore

UnaryOperationImplTable = list[tuple[Type[Value], UnaryOperationImpl]]


def eval_unary_operation(table: UnaryOperationImplTable, operand: Value, op_name: str) -> Value:
    for operand_type, impl in table:
        if isinstance(operand, operand_type):
            return impl(operand)
    else:
        raise CalcRuntimeError(f"{op_name} is not defined for {operand.type_name()}")


neg_impls: UnaryOperationImplTable = [(Float, lambda a: Float(-a.v))]  # type: ignore
