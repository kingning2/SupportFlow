"""Convert a local file to Markdown on stdout (used by Rust `knowledge::markitdown`)."""
from __future__ import annotations

import sys
from pathlib import Path


def main() -> int:
    if len(sys.argv) < 2:
        print("usage: markitdown_convert.py <path>", file=sys.stderr)
        return 1
    path = Path(sys.argv[1])
    if not path.is_file():
        print(f"not found: {path}", file=sys.stderr)
        return 1
    try:
        from markitdown import MarkItDown
    except ImportError:
        return 2
    md = MarkItDown(enable_plugins=False)
    convert = getattr(md, "convert_local", None) or md.convert
    result = convert(str(path))
    text = getattr(result, "text_content", None) or ""
    sys.stdout.write(text)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
