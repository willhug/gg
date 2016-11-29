from plumbum import colors

class Logger():
    def error(self, msg: str) -> None:
        """print an error message (red)"""
        print(colors.red | msg)

    @classmethod
    def info(self, msg: str) -> None:
        """print an info message (normal)"""
        print(msg)

logger = Logger()
