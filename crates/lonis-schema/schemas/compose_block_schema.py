import json, collections

KINDS = ["message", "question", "answer", "decision", "action", "assumption", "summary",
         "evidence", "definition", "capability", "intent", "plan", "result", "outcome"]

defs = {}
for k in KINDS:
    doc = json.load(open(f"{k}-v1.json"), object_pairs_hook=collections.OrderedDict)
    for meta in ("$schema", "$id", "x-lonis-protocol-version", "title"):
        doc.pop(meta, None)
    defs[k] = doc

# Hoist per-kind nested $defs (e.g. plan_step, evidence) into the envelope's
# $defs so their #/$defs/... refs resolve against this document.
for k in KINDS:
    nested = defs[k].pop("$defs", {})
    for name, sub in nested.items():
        if name in defs and defs[name] != sub:
            raise SystemExit(f"$defs collision on {name}")
        defs[name] = sub

def arr(items_schema, max_items):
    return {"type": "array", "maxItems": max_items, "items": items_schema}

def s(max_len):
    return {"type": "string", "maxLength": max_len}

provenance = {
    "type": "object", "additionalProperties": False,
    "properties": collections.OrderedDict([
        ("tool_version", s(256)),
        ("compatibility", {
            "type": "object", "additionalProperties": False,
            "required": ["status"],
            "properties": collections.OrderedDict([
                ("status", s(256)),
                ("reasons", arr(s(4096), 256)),
            ]),
        }),
        ("replay", {
            "type": "object", "additionalProperties": False,
            "required": ["replayable"],
            "properties": collections.OrderedDict([
                ("replayable", {"type": "boolean"}),
                ("required_hashes", arr(s(256), 64)),
                ("reasons", arr(s(4096), 256)),
            ]),
        }),
        ("project_hash", s(128)),
        ("input_hash", s(128)),
        ("plan_hash", s(128)),
        ("result_hash", s(128)),
        ("seed", {"type": "integer", "minimum": 0}),
    ]),
}

attribution = {
    "type": "object", "additionalProperties": False,
    "required": ["identity", "provenance"],
    "properties": collections.OrderedDict([
        ("identity", s(1024)),
        ("viewpoint", s(1024)),
        ("provenance", {
            "type": "object", "additionalProperties": False,
            "required": ["when", "producer"],
            "properties": collections.OrderedDict([
                ("when", s(64)),
                ("where", s(4096)),
                ("producer", s(1024)),
            ]),
        }),
    ]),
}

bounds = {
    "type": "object", "additionalProperties": False,
    "properties": collections.OrderedDict([
        ("max_items", {"type": "integer", "minimum": 0}),
        ("max_bytes", {"type": "integer", "minimum": 0}),
        ("max_length", {"type": "integer", "minimum": 0}),
        ("timeout_millis", {"type": "integer", "minimum": 0}),
    ]),
}

block = collections.OrderedDict([
    ("$schema", "https://json-schema.org/draft/2020-12/schema"),
    ("$id", "https://industrialalgebra.com/schemas/lonis.block/v1"),
    ("x-lonis-protocol-version", "lonis.block/v1"),
    ("title", "lonis.block envelope (all 14 seed kinds)"),
    ("type", "object"),
    ("additionalProperties", False),
    ("required", ["schema_version", "attribution", "payload"]),
    ("properties", collections.OrderedDict([
        ("schema_version", {"const": "lonis.block/v1"}),
        ("provenance", provenance),
        ("warnings", arr(s(4096), 256)),
        ("attribution", attribution),
        ("bounds", bounds),
        ("payload", {"oneOf": [{"$ref": f"#/$defs/{k}"} for k in KINDS]}),
    ])),
    ("$defs", defs),
])

with open("block-v1.json", "w") as f:
    f.write(json.dumps(block, indent=2) + "\n")
print("wrote block-v1.json with", len(defs), "$defs")
