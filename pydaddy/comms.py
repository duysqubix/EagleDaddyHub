import struct

from digi.xbee.models.message import XBeeMessage


class _ModuleCmds:
    RQST_PING = b"\x1d"

    def decode(data: bytes):
        return data


class DummyCmds(_ModuleCmds):
    RQST_TOGGLE = b"\x2b"
    RQST_INT = b"\x3c"
    RQST_FLOAT = b"\x4a"

    def decode(cmd, data: bytes):
        if cmd == DummyCmds.RQST_TOGGLE:
            return int.from_bytes(data, 'big')

        elif cmd == DummyCmds.RQST_INT:
            return int.from_bytes(data, 'big')

        elif cmd == DummyCmds.RQST_FLOAT:
            return struct.unpack('f', data)

        else:
            raise ValueError("not a valid cmdset command")


class ResponseMessage:
    def __init__(self, msg: XBeeMessage):
        self.__xmsg = msg
        self.data = msg.data[1:]

        # this requires incoming msg.data to have the first byte indicate the command that was originally
        # used to generate this reponse message
        self.orig_cmd = bytes([msg.data[0]])

    def decode(self, cmdset):
        return cmdset.decode(self.orig_cmd, self.data)
