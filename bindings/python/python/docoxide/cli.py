import argparse
import sys

from docoxide import CSS, HTML, Metadata


def main() -> int:
    parser = argparse.ArgumentParser(prog="docoxide", description="Fast, browser-free HTML to PDF converter.")
    parser.add_argument(
        "-i",
        "--input",
        help="HTML input: file path, URL (http/https/file), or - for stdin. Reads from stdin if omitted.",
    )
    parser.add_argument(
        "-s",
        "--stylesheet",
        action="append",
        default=[],
        help="Additional CSS stylesheet file. Can be repeated.",
    )
    parser.add_argument("-b", "--base-url", help="Base URL for resolving relative links and images.")
    parser.add_argument(
        "-m",
        "--metadata",
        action="append",
        default=[],
        help="PDF metadata as key=value. Can be repeated. Keys: title, author, subject, keywords, creation_date.",
    )
    parser.add_argument("--font", action="append", default=[], help="Custom font file. Can be repeated.")
    parser.add_argument("-o", "--output", help="Output PDF file. Writes to stdout if omitted.")
    parser.add_argument("-q", "--quiet", action="store_true", help="Suppress progress output.")
    args = parser.parse_args()

    source = args.input
    if source is None or source == "-":
        html = HTML(string=sys.stdin.read(), base_url=args.base_url)
    elif source.startswith(("http://", "https://", "file://")):
        html = HTML(url=source, base_url=args.base_url)
    else:
        html = HTML(filename=source, base_url=args.base_url)

    for css_path in args.stylesheet:
        html.add_stylesheet(CSS(filename=css_path))

    for font_path in args.font:
        html.add_font(filename=font_path)

    meta = parse_metadata(args.metadata)
    if meta:
        html.set_metadata(meta)

    pdf = html.write_pdf()

    if args.output:
        with open(args.output, "wb") as f:
            f.write(bytes(pdf))
        if not args.quiet:
            print(f"docoxide: written {pdf.page_count} page(s) to {args.output}", file=sys.stderr)
    else:
        sys.stdout.buffer.write(bytes(pdf))

    return 0


def parse_metadata(entries: list[str]) -> Metadata | None:
    if not entries:
        return None
    kwargs: dict = {}
    for entry in entries:
        if "=" not in entry:
            print(f"docoxide: error: metadata must be key=value, got '{entry}'", file=sys.stderr)
            sys.exit(1)
        key, value = entry.split("=", 1)
        key = key.strip()
        if key in ("title", "author", "subject", "creation_date"):
            kwargs[key] = value
        elif key == "keywords":
            kwargs.setdefault("keywords", []).append(value)
        else:
            print(f"docoxide: error: unknown metadata key '{key}'", file=sys.stderr)
            sys.exit(1)
    return Metadata(**kwargs) if kwargs else None


if __name__ == "__main__":
    sys.exit(main())
