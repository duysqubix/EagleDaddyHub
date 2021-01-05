from sqlalchemy import Binary, String, Integer
from sqlalchemy.sql.schema import Column
from sqlalchemy.sql.sqltypes import Boolean
from pydaddy.db import Model, engine


class RemoteModule(Model):
    """
    Stores information about remote modules
    """
    __tablename__ = "remote_modules"

    id = Column(Integer, primary_key=True, unique=True, nullable=False)
    address64 = Column(Binary, nullable=False)
    node_id = Column(String(64), nullable=False)
    operating_mode = Column(Binary, nullable=False)
    network_id = Column(Binary, nullable=False)
    parent_device = Column(Binary, nullable=False)


Model.metadata.create_all(engine)