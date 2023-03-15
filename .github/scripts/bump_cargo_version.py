from os import environ as env
from typing import Any

import tomlkit

with open("./Cargo.toml") as cargo_file:
    doc: Any = tomlkit.load(cargo_file)
    doc["package"]["version"] = doc["package"]["version"] + "+dev." + env["GITHUB_SHA"][:7]
with open("./Cargo.toml", "w") as cargo_file:
    tomlkit.dump(doc, cargo_file)
