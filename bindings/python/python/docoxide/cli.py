import argparse
import sys

from docoxide import convert


def main() -> int:
    parser = argparse.ArgumentParser(prog="docoxide", description="Convert HTML to PDF.")
    parser.add_argument("-i", "--input", help="Path to the input HTML file. Reads from stdin if omitted or set to '-'.")
    parser.add_argument("-c", "--css", help="Optional path to a CSS file.")
    parser.add_argument("-o", "--output", required=True, help="Path to the output PDF file.")
    args = parser.parse_args()

    if args.input is None or args.input == "-":
        html = sys.stdin.read()
    else:
        with open(args.input, encoding="utf-8") as f:
            html = f.read()

    css = None
    if args.css:
        with open(args.css, encoding="utf-8") as f:
            css = f.read()

    pdf = convert(html, css)

    with open(args.output, "wb") as f:
        f.write(pdf)

    return 0


if __name__ == "__main__":
    sys.exit(main())
