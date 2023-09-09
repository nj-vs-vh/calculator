from calculator.parser import BinaryOperation, BinaryOperator, Expression, UnaryOperation, UnaryOperator, Variable


def evaluate(expressions: list[Expression], variables: dict[str, float]) -> list[float]:
    results: list[float] = []
    for expression in expressions:
        results.append(evaluate_expression(expression, variables))
    return results


def evaluate_expression(expression: Expression, variables: dict[str, float]) -> float:
    if isinstance(expression, float):
        return expression
    elif isinstance(expression, Variable):
        if expression.name in variables:
            return variables[expression.name]
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
            return left_res + right_res
        elif expression.operator == BinaryOperator.SUB:
            return left_res - right_res
        elif expression.operator == BinaryOperator.MUL:
            return left_res * right_res
        elif expression.operator == BinaryOperator.DIV:
            return left_res / right_res
        elif expression.operator == BinaryOperator.POW:
            return left_res**right_res
        else:
            raise RuntimeError(f"Unexpected binary operator: {expression.operator}")
    elif isinstance(expression, UnaryOperation):
        operand = evaluate_expression(expression.operand, variables)
        if expression.operator is UnaryOperator.NEG:
            return -operand
        elif expression.operator is UnaryOperator.POS:
            return operand
        else:
            raise RuntimeError(f"Unexpected unary operator: {expression.operator}")
    else:
        raise RuntimeError(f"Unexpected expression type: {expression}")
