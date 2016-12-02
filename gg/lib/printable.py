from typing import List

from plumbum import colors

class Line:
    printers = None # stringables

    def __init__(self, printers):
        self.printers = printers

    def __str__(self) -> str:
        result = ""
        for printer in self.printers:
            if printer is None:
                continue
            result += str(printer) + '\t'
        return result


# FB as in "with fallback"
class FB:
    base = "" # stringable
    fallback = "" # stringable

    def __init__(self, base, fallback):
        self.base = base
        self.fallback = fallback

    def __str__(self) -> str:
        if not str(self.base) or self.base is None:
            return str(self.fallback)
        return str(self.base)

class Color:
    color = "" # from plumbum
    value = "" # stringable

    def __init__(self, value, color):
        self.value = value
        self.color = color

    def __str__(self) -> str:
        if not str(self.value) or self.value is None:
            return ""
        return (self.color | str(self.value))

# RED
class R(Color):
    def __init__(self, value):
        super().__init__(value, colors.red)

# WHITE
class W(Color):
    def __init__(self, value):
        super().__init__(value, colors.white)

# GREEN
class G(Color):
    def __init__(self, value):
        super().__init__(value, colors.green)

# BLUE
class B(Color):
    def __init__(self, value):
        super().__init__(value, colors.lightblue)
