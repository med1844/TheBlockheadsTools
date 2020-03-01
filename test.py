import lmdb
import gzip
import ccl_bplist
import io
import pprint


OUT_FOLDER = "OUTPUT/"
FOLDER = "../4d8b32b0548e1c203ff3684870c41876/4d8b32b0548e1c203ff3684870c41876/"


def match(b1, b2):
    for i, b in enumerate(b2):
        if b1[i] != b:
            return False
    return True


def process(v):
    if isinstance(v, list):
        for i, _ in enumerate(v):
            v[i] = process(_)
        return v
    elif isinstance(v, dict):
        for a, b in v.items():
            v[a] = process(b)
        return v
    elif isinstance(v, bytes):
        f = io.BytesIO(v)
        result = None
        if match(v, b"bplist00"):
            result = ccl_bplist.load(f)
            result = process(result)
        elif match(v, b"\x1f\x8b"):
            with gzip.open(f, "rb") as f2:
                content = f2.read()
            result = process(content)
        else:
            result = v
        return result
    else:
        return v


env = lmdb.open(FOLDER + "world_db", readonly=True, max_dbs=100)
with env.begin() as txn:
    cursor = txn.cursor()
    for k, v in cursor:
        sub_db = env.open_db(k, txn=txn, create=False)
        for k2, v2 in txn.cursor(sub_db):
            filename = FOLDER + OUT_FOLDER + "%s_%s" \
                       % (k.decode(), k2.decode().replace("/", "_"))
            result = process(v2)
            if isinstance(result, bytes):
                with open(filename, "wb") as f:
                    f.write(result)
            else:
                with open(filename, "w") as f:
                    f.write(pprint.pformat(result))
env.close()