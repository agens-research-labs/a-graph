"""A utility of a-graph: reads {"path": "<file.pdf>"} on the standard input and answers
{"text": "<the file as Markdown>"} on the standard output. Needs `pip install pymupdf4llm`."""
import json
import sys

import pymupdf4llm

request = json.load(sys.stdin)
print(json.dumps({"text": pymupdf4llm.to_markdown(request["path"])}))
