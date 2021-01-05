"""

"""
from pydaddy.db.models import RemoteModule
from digi.xbee.devices import RemoteDigiMeshDevice
from pydaddy.db import Model


class _Proxy:
    __table_mapping__ = None

    def __init__(self, module: RemoteDigiMeshDevice) -> None:
        self.module: RemoteDigiMeshDevice = module
        self.parent = self.module.get_local_xbee_device()

    @property
    def model(self):
        return self.__table_mapping__

    def to_model(self) -> Model:
        pass


class ProxyDigiMeshDevice(_Proxy):
    __table_mapping__ = RemoteModule

    def to_model(self) -> Model:

        params = dict(address64=self.module.get_64bit_addr().address,
                      node_id=self.module.get_node_id(),
                      operating_mode=self.module.get_parameter("AP"),
                      network_id=self.module.get_parameter("ID"),
                      parent_device=self.parent.get_64bit_addr().address)

        return self.model(**params)

    @classmethod
    def from_record(cls, record):
        pass
