from pydaddy import make_iter


class Payload:
    """
    Representation of payload to remote devices
    """
    def __init__(self, data):
        self.data = make_iter(data)

    def __str__(self):
        return f"<PAYLOAD: {bytes(self.data)}>"

    def __repr__(self) -> str:
        return str(self)

    @property
    def encoded(self):
        return bytes(self.data)