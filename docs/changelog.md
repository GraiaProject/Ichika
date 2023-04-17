```python exec="true"
from pathlib import Path
import subprocess as proc
import tomlkit

REPLACE_TAG = "<!-- towncrier release notes start -->"

cwd = Path.cwd()
version: str = tomlkit.loads(Path(cwd, "Cargo.toml").read_text("utf-8"))["package"]["version"]
changelog = Path(cwd, "CHANGELOG.md").read_text("utf-8")
rendered = proc.run(
    ["towncrier", "build", "--draft", "--keep", "--name", "ichika", "--version", version],
    stdout=proc.PIPE,
    stderr=proc.DEVNULL,
    text=True
).stdout
print(changelog.replace(REPLACE_TAG, rendered))
```
