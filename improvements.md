use constants instead of hard-coded values
some opcodes don't check if access to v is out of bounds, but this isn't C, so the program panics
