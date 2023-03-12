from os import environ as env

from actions_toolkit import core

includes = []
mapping = {
    "macos": ["x64", "aarch64", "universal2"],
    "windows": ["x64", "x86", "aarch64"],
    "linux-musl": ["x64", "x86", "aarch64", "armv7"],
    "linux": ["x64", "x86", "aarch64", "armv7", "s390x", "ppc64", "ppc64le"],
}
core.start_group("Jobs")

for os, targets in mapping.items():
    for target in targets:
        job = {
            "name": f"{os}-{target}",
            "os": ("ubuntu" if "linux" in os else os) + "-latest",
            "target": target,
            "build_cmd": "build",
            "build_args": ["--out", "dist"],
        }

        if os == "windows" and target == "x86":
            job["py_arch"] = "x86"

        if "linux" in os:
            job["manylinux"] = "musllinux_1_2" if "musl" in os else "auto"

        if env["RELEASE"] == "true":
            job["build_args"].append("--release")

        if target == "universal2":
            job["target"] = "aarch64"
            job["build_args"].append("--universal2")

        job["build_args"] = " ".join(job["build_args"])
        includes.append(job)
        core.info(f"Job: {job}")

includes.append(
    {
        "name": "source",
        "os": "ubuntu-latest",
        "build_cmd": "sdist",
        "build_args": "--out dist",
    }
)
core.info(f"Job: {includes[-1]}")

core.end_group()

core.set_output("matrix", {"include": includes})
