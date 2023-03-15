from typing import Any

import tomlkit
from actions_toolkit import core

with open("./Cargo.toml") as cargo_file:
    doc: Any = tomlkit.load(cargo_file)
    core.set_output("version", str(doc["package"]["version"]))
