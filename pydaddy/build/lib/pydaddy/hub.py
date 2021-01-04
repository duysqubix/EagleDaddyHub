from typing import *
import yaml
from pathlib import Path

from digi.xbee.devices import DigiMeshDevice

from pydaddy import BROADCAST_ADDR


class HubModule:
    """
    A singleton instance representing the main
    hub module, the one that interacts with the rest of 
    the network
    """
    __instance = None

    def __new__(cls) -> Any:
        if cls.__instance is None:
            cls.__instance = super(HubModule, cls).__new__(cls)
        return cls.__instance

    def __init__(self) -> None:
        baud: int = self.config['baud']
        port: str = self.config['port']
        self.hub: DigiMeshDevice = DigiMeshDevice(port=port, baud_rate=baud)

    def open(self):
        self.hub.open()

    def close(self):
        self.hub.close()

    def discover(self):
        payload = b"\x0a\xaa"
