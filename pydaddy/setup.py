import os
from setuptools import setup, find_packages
from pathlib import Path

VERSION_PATH = Path(__file__).parent / "pydaddy/VERSION.txt"


def get_version():
    return open(VERSION_PATH).read().strip()


def get_requirements():
    with open("requirements.txt", 'r') as f:
        req_lines = f.readlines()

    reqs = []
    for line in req_lines:
        line = line.split("#")[0].strip()
        if line:
            reqs.append(line)

    return reqs


def get_scripts():
    file_set = []

    for f in Path("bin").glob("*"):
        file_name = str(f.absolute())
        file_set.append(file_name)

    return file_set


def get_package_data():
    file_set = []
    for root, dirs, files in os.walk("pydaddy"):
        for f in files:
            if ".git" in f.split(os.path.normpath(os.path.join(root, f))):
                continue
            file_name = os.path.relpath(os.path.join(root, f), 'pydaddy')
            file_set.append(file_name)
    return file_set


setup(name="pydaddy",
      author="duan uys",
      version=get_version(),
      maintainer="duan uys",
      description="agriculture asset management system",
      packages=find_packages(),
      scripts=get_scripts(),
      install_requires=get_requirements(),
      package_data={"": get_package_data()},
      zip_safe=False,
      python_requires=">=3.7")
