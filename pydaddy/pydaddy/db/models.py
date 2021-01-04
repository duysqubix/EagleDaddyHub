from sqlalchemy import Integer, String
from sqlalchemy.sql.schema import Column
from pydaddy.db import Model, engine


class RemoteModule(Model):
    """
    Stores information about remote modules
    """
    __tablename__ = "remote_modules"

    id = Column(Integer, primary_key=True, unique=True, nullable=False)
    address64 = Column(Integer, nullable=False)
    node_id = Column(String(64), nullable=False)
    operating_mode = Column(Integer, nullable=False)
    cluster_id = Column(Integer, nullable=False)
    profile_id = Column(Integer, nullable=False)


Model.metadata.create_all(engine)