from calculator.parser import BinaryOperation, BinaryOperator, Expression, UnaryOperation, UnaryOperator


def evaluate(expressions: list[Expression]) -> list[float]:
    results: list[float] = []
    for expression in expressions:
        results.append(evaluate_expression(expression))
    return results


def evaluate_expression(expression: Expression) -> float:
    if isinstance(expression, float):
        return expression
    elif isinstance(expression, BinaryOperation):
        left_res = evaluate_expression(expression.left)
        right_res = evaluate_expression(expression.right)
        if expression.operator == BinaryOperator.ADD:
            return left_res + right_res
        elif expression.operator == BinaryOperator.SUB:
            return left_res - right_res
        elif expression.operator == BinaryOperator.MUL:
            return left_res * right_res
        elif expression.operator == BinaryOperator.DIV:
            return left_res / right_res
        else:
            raise RuntimeError(f"Unexpected binary operator: {expression.operator}")
    elif isinstance(expression, UnaryOperation):
        operand = evaluate_expression(expression.operand)
        if expression.operator is UnaryOperator.NEG:
            return -operand
        elif expression.operator is UnaryOperator.POS:
            return operand
        else:
            raise RuntimeError(f"Unexpected unary operator: {expression.operator}")
    else:
        raise RuntimeError(f"Unexpected expression type: {expression}")
