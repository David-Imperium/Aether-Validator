# Python Contract Template

```python
"""
Custom Synward validation contract for Python

Contract name: my_python_contract
Description: Validates custom Python patterns
"""

from typing import List
from synward_contracts import Contract, ValidationError, ASTNode


class MyPythonContract(Contract):
    @property
    def name(self) -> str:
        return "my_python_contract"

    @property
    def description(self) -> str:
        return "Validates custom Python patterns"

    def validate(self, code: str, ast: ASTNode) -> List[ValidationError]:
        errors = []

        # Example: Check for print statements in production
        if "print(" in code:
            errors.append(ValidationError(
                id="print_in_prod",
                message="print() should use logging in production",
                line=None,
                column=None,
                layer="style",
                is_new=True,
            ))

        # Example: Check for bare except
        if "except:" in code:
            errors.append(ValidationError(
                id="bare_except",
                message="Bare except: is not recommended, use except Exception:",
                line=None,
                column=None,
                layer="best_practice",
                is_new=True,
            ))

        return errors
```

## Usage

1. Save as `.synward/contracts/my_python_contract.py`
2. Run: `synward validate main.py --contracts my_python_contract`
