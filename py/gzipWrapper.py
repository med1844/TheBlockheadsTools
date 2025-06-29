# encoding: utf-8
from exportable import Exportable
import io
import gzip


class GzipWrapper(Exportable):
    def __init__(self, src_bytes: bytes):
        with io.BytesIO(src_bytes) as src:
            with gzip.GzipFile(fileobj=src, mode="rb") as f:
                self._data = [f.read()]

    def export(self):
        with io.BytesIO() as f:
            with gzip.GzipFile(fileobj=f, mode="wb") as g:
                if isinstance(self._data[0], (str, bytes)):
                    g.write(self._data[0])
                elif isinstance(self._data[0], Exportable):
                    g.write(self._data[0].export())
            result = f.getvalue()
        return result
