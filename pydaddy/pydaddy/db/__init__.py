import yaml
from pathlib import Path
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker
from sqlalchemy.ext.declarative import declarative_base

__db_path = str((Path(__file__).parent.parent / "../db.db3").absolute())
Model = declarative_base()
engine = create_engine(f"sqlite:///{__db_path}")

DB = sessionmaker(bind=engine)()
CONFIG = yaml.load(open(Path(__file__).parent.parent / "../config.yaml"),
                   Loader=yaml.Loader)
