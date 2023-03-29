import sys
from pathlib import Path
from typing import Any

import tomlkit

CURRENT_DIR = Path(__file__, "..").absolute()
sys.path.append(CURRENT_DIR.as_posix())

cargo: Any = tomlkit.loads(Path(CURRENT_DIR, "./../Cargo.toml").read_text("utf-8"))

# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = "Ichika"
copyright = "2023, BlueGlassBlock"
author = "BlueGlassBlock"
release = str(cargo["package"]["version"])

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = [
    "myst_parser",
    "autodoc2",
    "sphinxext.opengraph",
    "sphinx.ext.intersphinx",
    "sphinx.ext.todo",
    "myst_parser",
    "sphinx_copybutton",
]
templates_path = ["_templates"]
exclude_patterns = []
language = "zh_CN"
source_suffix = {
    ".rst": "restructuredtext",
    ".txt": "markdown",
    ".md": "markdown",
}

intersphinx_mapping = {
    "python": ("https://docs.python.org/zh-cn/3/", None),
}

myst_enable_extensions = [
    "dollarmath",
    "amsmath",
    "deflist",
    "fieldlist",
    "html_admonition",
    "html_image",
    "colon_fence",
    "smartquotes",
    "replacements",
    "strikethrough",
    "substitution",
    "tasklist",
    "attrs_inline",
]

html_theme = "furo"
html_static_path = ["_static"]

html_theme_options = {
    "source_repository": "https://github.com/BlueGlassBlock/Ichika/",
    "source_branch": "master",
    "source_directory": "docs/",
}

autodoc2_packages = [
    "../python/ichika",
]
autodoc2_output_dir = "api"

autodoc2_hidden_objects = {"private", "inherited"}

autodoc2_class_docstring = "both"
autodoc2_index_template = Path(__file__, "..", "./_templates/api-doc-index.rst").read_text("utf-8")
autodoc2_render_plugin = "api_renderer.APIRenderer"

ogp_site_url = "https://hoshino-ichika.netlify.app/"
