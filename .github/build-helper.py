import os
import sys
from typing import Any

import tomlkit

sha, release = sys.argv[1:]

release = release == "true"
if not release:
    with open("./Cargo.toml") as cargo_file:
        doc: Any = tomlkit.load(cargo_file)

    doc["package"]["version"] = doc["package"]["version"] + "-dev." + sha[:7]

    with open("./Cargo.toml", "w") as cargo_file:
        tomlkit.dump(doc, cargo_file)

with open(os.environ["GITHUB_ENV"], "a") as env:
    env.write("BASE_BUILD='--release --out --dist'" if release else "BASE_BUILD='--out --dist'")
    env.write("\n")
