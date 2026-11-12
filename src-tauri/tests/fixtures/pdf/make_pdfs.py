"""Write the small PDFs the importer tests read. Run: python make_pdfs.py
Text is public-domain hymn text (Amazing Grace, John Newton, 1779).
Two files are deliberately malformed: pdf-extract 0.12 panics on them."""
import os

HERE = os.path.dirname(os.path.abspath(__file__))
FONT = b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>"


def pdf(pages, font_name="F1", mediabox="0 0 612 792"):
    """pages: list of page texts ('' = a page with no text)."""
    n = len(pages)
    # 1 catalog, 2 pages, 3 font, then (page, contents) per page.
    kids = " ".join(f"{4 + 2 * i} 0 R" for i in range(n))
    objs = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        f"<< /Type /Pages /Kids [{kids}] /Count {n} >>".encode(),
        FONT,
    ]
    for i, text in enumerate(pages):
        objs.append(
            f"<< /Type /Page /Parent 2 0 R /MediaBox [{mediabox}] /Contents {5 + 2 * i} 0 R "
            f"/Resources << /Font << /F1 3 0 R >> >> >>".encode()
        )
        body = f"BT /{font_name} 18 Tf 72 700 Td ({text}) Tj ET".encode() if text else b""
        objs.append(b"<< /Length %d >>\nstream\n" % len(body) + body + b"\nendstream")
    out = bytearray(b"%PDF-1.4\n")
    offsets = []
    for i, obj in enumerate(objs, 1):
        offsets.append(len(out))
        out += f"{i} 0 obj\n".encode() + obj + b"\nendobj\n"
    xref = len(out)
    out += f"xref\n0 {len(objs) + 1}\n0000000000 65535 f \n".encode()
    for off in offsets:
        out += f"{off:010d} 00000 n \n".encode()
    out += f"trailer\n<< /Size {len(objs) + 1} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n".encode()
    return bytes(out)


LINES = [
    "Amazing grace how sweet the sound",
    "Through many dangers toils and snares",
    "The Lord has promised good to me",
]
FILES = {
    "three-pages.pdf": pdf(LINES),
    "blank-middle.pdf": pdf([LINES[0], "", LINES[2]]),
    # The content uses /F9; the page resources define only /F1.
    "unknown-font.pdf": pdf(LINES[:1], font_name="F9"),
    # A MediaBox with three numbers instead of four.
    "mediabox-3.pdf": pdf(LINES[:1], mediabox="0 0 612"),
}
for name, data in FILES.items():
    with open(os.path.join(HERE, name), "wb") as f:
        f.write(data)
print(sorted(FILES))
