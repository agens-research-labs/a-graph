"""Makes contract.pdf (run it once with `pip install pymupdf`; the PDF is kept in the folder)."""
import pymupdf

doc = pymupdf.open()
page = doc.new_page()
page.insert_text((72, 90), "Lease agreement", fontsize=22)
page.insert_text((72, 140), "Article 1. The lease lasts 12 months and renews every year.", fontsize=12)
page.insert_text((72, 165), "Article 2. The monthly rent is 800 euro, paid by the 5th.", fontsize=12)
page.insert_text((72, 190), "Article 3. The tenant gives notice by registered mail.", fontsize=12)
doc.save("contract.pdf")
