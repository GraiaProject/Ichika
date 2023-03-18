import json
import sys
from os import environ as env
from pathlib import Path

from actions_toolkit.file_command import issue_file_command

matrix: dict = json.loads(env["MATRIX"])
result = env["RESULT"]

success = """\
## ✅ {name} 构建成功。

<details>
<summary> 安装步骤 (需要 GitHub CLI) </summary>

```
gh run download {run_id} -R {repo} -n {name}
pip install {filename}
```

</details>
"""

fail = """\
## ❎ {name} 构建失败。
"""

if result.lower() == "success":
    filename = "<ERROR>"
    for res in Path("./dist").iterdir():
        if res.suffix in (".whl", ".tar.gz"):
            filename = res.name
    issue_file_command(
        "STEP_SUMMARY",
        success.format(
            name=matrix["name"], run_id=env["GITHUB_RUN_ID"], repo=env["GITHUB_REPOSITORY"], filename=filename
        ),
    )
else:
    issue_file_command("STEP_SUMMARY", fail.format(name=matrix["name"]))
    sys.exit(1)
