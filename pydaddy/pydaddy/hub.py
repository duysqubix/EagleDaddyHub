from typing import *
import yaml
from pathlib import Path

from digi.xbee.devices import DigiMeshDevice, XBeeNetwork

from pydaddy.db import DB, CONFIG


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
        global DB, CONFIG

        baud: int = CONFIG['baud']
        port: str = CONFIG['port']
        self.hub: DigiMeshDevice = DigiMeshDevice(port=port, baud_rate=baud)
        self.xnet: XBeeNetwork = self.hub.get_network()
        self.db = DB
        self.config = CONFIG

    def open(self):
        self.hub.open()

    def close(self):
        self.hub.close()

    def discover_all_devices(self, *args, **kwargs):
        self.xnet.start_discovery_process(**kwargs)