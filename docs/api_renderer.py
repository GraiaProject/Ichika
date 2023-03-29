import typing as t

from autodoc2.render.myst_ import MystRenderer as BaseRenderer
from autodoc2.utils import ItemData


class APIRenderer(BaseRenderer):
    def render_package(self, item: ItemData) -> t.Iterable[str]:
        """Create the content for a package."""
        if self.standalone and self.is_hidden(item):
            yield from ["---", "orphan: true", "---", ""]

        full_name = item["full_name"]

        yield f"# {{py:mod}}`{full_name}`"
        yield ""

        yield f"```{{py:module}} {full_name}"
        if self.no_index(item):
            yield ":noindex:"
        if self.is_module_deprecated(item):
            yield ":deprecated:"
        yield from ["```", ""]

        if self.show_docstring(item):
            yield f"```{{autodoc2-docstring}} {item['full_name']}"
            if parser_name := self.get_doc_parser(item["full_name"]):
                yield f":parser: {parser_name}"
            yield ":allowtitles:"
            yield "```"
            yield ""

        visible_subpackages = [i["full_name"] for i in self.get_children(item, {"package"})]
        if visible_subpackages:
            yield from [
                "## 子包",
                "",
                "```{toctree}",
                ":titlesonly:",
                ":maxdepth: 3",
                "",
            ]
            yield from visible_subpackages
            yield "```"
            yield ""

        visible_submodules = [i["full_name"] for i in self.get_children(item, {"module"})]
        if visible_submodules:
            yield from [
                "## 子模块",
                "",
                "```{toctree}",
                ":titlesonly:",
                ":maxdepth: 1",
                "",
            ]
            yield from visible_submodules
            yield "```"
            yield ""

        visible_children = [i["full_name"] for i in self.get_children(item) if i["type"] not in ("package", "module")]
        if not visible_children:
            return

        yield from ["## API", ""]
        for name in visible_children:
            yield from self.render_item(name)

        if self.show_module_summary(item):
            for heading, types in [
                ("类", {"class"}),
                ("函数", {"function"}),
                ("数据", {"data"}),
                ("外部类型", {"external"}),
            ]:
                visible_items = list(self.get_children(item, types))
                if visible_items:
                    yield from [f"## {heading}", ""]
                    yield from self.generate_summary(
                        visible_items,
                        alias={i["full_name"]: i["full_name"].split(".")[-1] for i in visible_items},
                    )
                    yield ""
