"""Convert a local file to Markdown on stdout (used by Rust `knowledge::markitdown`)."""
from __future__ import annotations

import sys
from pathlib import Path
import os


def _extract_pdf_text_with_pypdf(path: Path) -> str:
    """Fallback extractor for PDF when MarkItDown returns empty text."""
    try:
        from pypdf import PdfReader
    except ImportError:
        return ""

    try:
        reader = PdfReader(str(path))
    except Exception:
        return ""

    parts: list[str] = []
    for page in reader.pages:
        try:
            text = page.extract_text() or ""
        except Exception:
            text = ""
        if text.strip():
            parts.append(text)
    return "\n\n".join(parts).strip()


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
    # Enable MarkItDown plugins for legacy Word formats that often need
    # add-on parsers (e.g. .doc). Keep it opt-out for safety.
    # - You can override explicitly via env `MARKITDOWN_ENABLE_PLUGINS=0/1`.
    suffix = path.suffix.lower()
    env_override = os.getenv("MARKITDOWN_ENABLE_PLUGINS")
    if env_override is not None:
        enable_plugins = env_override.strip().lower() not in ("0", "false", "no")
    else:
        enable_plugins = suffix in (".doc", ".docx")

    md = MarkItDown(enable_plugins=enable_plugins)
    convert = getattr(md, "convert_local", None) or md.convert
    result = convert(str(path))
    text = (getattr(result, "text_content", None) or "").strip()

    # Some PDFs are readable by pure Python PDF parsers even when MarkItDown
    # returns empty (for example due to parser edge cases / plugin differences).
    # Keep this as a lightweight fallback before Rust legacy parser.
    if not text and suffix == ".pdf":
        text = _extract_pdf_text_with_pypdf(path)

    sys.stdout.write(text)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
