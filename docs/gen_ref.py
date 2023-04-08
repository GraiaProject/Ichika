"""Generate the code reference pages and navigation."""
import sys
from pathlib import Path

from mkdocs_gen_files.editor import FilesEditor
from mkdocs_gen_files.nav import Nav

nav = Nav()

fe = FilesEditor.current()

root = Path(__file__).parent.parent
docs_dir = root / "docs"

src = (root / "python").resolve()
sys.path.append(src.as_posix())

for path in sorted(Path(src, "ichika").glob("**/*.py")):
    module_path = path.relative_to(src).with_suffix("")
    full_doc_path = path.relative_to(src / "ichika").with_suffix(".md")

    parts = list(module_path.parts)
    if parts[-1] == "__init__":
        parts = parts[:-1]
        full_doc_path = full_doc_path.with_name("index.md")
    elif parts[-1] == "__main__" or parts[-1].startswith("_"):
        continue
    full_doc_path = ("api" / full_doc_path).as_posix()
    nav[tuple(parts)] = full_doc_path

    with fe.open(full_doc_path, "w") as f:
        print(f"::: {'.'.join(parts)}", file=f)

    fe.set_edit_path(full_doc_path, path.as_posix())

with fe.open("INDEX.nav", "w") as nav_file:
    nav_file.write(Path(docs_dir, "./INDEX.nav.template").read_text("utf-8"))
    nav_file.writelines(nav.build_literate_nav(indentation=4))
