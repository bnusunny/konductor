# Synthetic Test Project: Python CLI Calculator

## Description
Build a simple command-line calculator in Python that takes two numbers and an operator as arguments and prints the result.

## Usage
```
python calc.py 2 + 3
# Output: 5

python calc.py 10 - 4
# Output: 6

python calc.py 6 x 7
# Output: 42

python calc.py 15 / 3
# Output: 5.0
```

## Requirements
- REQ-01: The calculator accepts three command-line arguments: number1, operator, number2
- REQ-02: Supported operators: + (add), - (subtract), x (multiply), / (divide)
- REQ-03: Division by zero prints an error message instead of crashing
- REQ-04: Invalid input (non-numeric, unknown operator) prints a usage message

## Success Criteria
- `python calc.py 2 + 3` outputs `5`
- `python calc.py 10 / 0` prints an error message
- `python calc.py` (no args) prints usage
- All tests pass

## Tech Stack
- Python 3 (no external dependencies)
- Single file: calc.py
- Test file: test_calc.py
