from typing import BinaryIO

class Metadata:
    def __init__(
        self,
        *,
        title: str | None = None,
        author: str | None = None,
        subject: str | None = None,
        keywords: list[str] | None = None,
        creation_date: str | None = None,
    ) -> None: ...

class CSS:
    def __init__(
        self,
        *,
        string: str | None = None,
        filename: str | None = None,
        file_obj: object | None = None,
    ) -> None: ...

class PDF:
    @property
    def page_count(self) -> int: ...
    def as_bytes(self) -> bytes: ...
    def __bytes__(self) -> bytes: ...
    def __len__(self) -> int: ...

class HTML:
    def __init__(
        self,
        *,
        string: str | None = None,
        url: str | None = None,
        filename: str | None = None,
        file_obj: object | None = None,
        base_url: str | None = None,
    ) -> None: ...
    def add_stylesheet(self, css: CSS | str) -> None: ...
    def add_font(self, *, filename: str | None = None, font_bytes: bytes | None = None) -> None: ...
    def set_metadata(self, meta: Metadata) -> None: ...
    def write_pdf(
        self,
        target: str | BinaryIO | None = None,
        *,
        stylesheets: list[CSS | str] | None = None,
    ) -> PDF: ...

def convert(html: str, css: str | None = None) -> bytes: ...
