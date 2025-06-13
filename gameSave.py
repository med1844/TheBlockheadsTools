# encoding: utf-8
import os
import pathlib
import lmdb
import plistlib
from typing import Dict, Any
from bplist import BPList
from gzipWrapper import GzipWrapper
from bh_chunk import Chunk
from block import Block
from blockhead import Blockhead
from inventory import Inventory
from exportable import Exportable
from dataclasses import dataclass


@dataclass
class SaveSummary:
    world_name: str
    start_portal_pos: tuple[int, int]
    seed: int
    world_width_in_chunks: int
    expert_mode: bool


class GameSave:
    """
    The class describes the save file of a world. This abstracts save file to
    a simple class, and isolated instructions like creating lmdb context,
    manipulating cursors, and loading and saving BPLists, etc. It also
    provides methods to load and save GameSave within one method call. On top
    of these, methods for manipulating chunks, blocks and dynamic objects
    will be offered.
    """

    MAX_DBS = 100

    def __init__(self, folder_path):
        if not folder_path.endswith("/"):
            folder_path += "/"
        self._data: dict[str, Any] = {}
        for sub_dir in ["world_db", "server_db", "lightBlocks"]:
            full_path = folder_path + sub_dir
            if os.path.isdir(full_path):
                self._data[sub_dir] = {}
                self._read_env(full_path, self._data[sub_dir])

        self.chunks: Dict[bytes, GzipWrapper | Chunk] = self._data["world_db"][
            b"blocks"
        ]

    def __repr__(self):
        return repr(self._data)

    def __getitem__(self, key):
        return self._data[key]

    def __setitem__(self, key, value):
        self._data[key] = value

    def _read_env(self, path: str, dict_: dict[str, Any]):
        """Read all databases in LMDB Environment from given path, and write
        key-value pairs into `dict_`."""
        env = lmdb.open(path, readonly=True, max_dbs=self.MAX_DBS)
        with env.begin() as txn:
            for k, _ in txn.cursor():
                sub_db = env.open_db(k, txn=txn, create=False)
                dict_[k] = {}
                self._read_db(txn, sub_db, dict_[k])
        env.close()

    def _read_db(self, txn, db, dict_):
        """
        Write all key-value pairs in db into dict_, given transaction, db and
        dict_.
        输入Transaction, db和要写入的字典，读取db中所有键值对并写入dict_中。
        """
        for k, v in txn.cursor(db):
            dict_[k] = self._parse(v)

    def _parse(self, src):
        """
        Read the input bytes and determine which type of data to convert, and
        return the recursively parsed result.

        Types that would be parsed includes:
        - gzip files
        - base64 encoded data
        - bplist
        - normal string
        - xml plist files
        """
        if isinstance(src, bytes):
            if src.startswith(b"bplist00"):  # bplist
                result = BPList(plistlib.loads(src), src_type="bp")
                return self._parse(result)
            if src.startswith(b"\x1f\x8b"):  # gzip
                result = GzipWrapper(src)
                result._data[0] = self._parse(result._data[0])
                return result
            if src.startswith(b"<?xml"):  # xml plist
                result = BPList(plistlib.loads(src), src_type="xml")
                return self._parse(result)
            return src
        elif isinstance(src, list):
            for i, v in enumerate(src):
                src[i] = self._parse(v)
            return src
        elif isinstance(src, dict):
            for k, v in src.items():
                src[k] = self._parse(v)
            return src
        elif isinstance(src, BPList):
            src._data = self._parse(src._data)
            return src
        return src

    @classmethod
    def load(cls, path):
        """
        Read save files according to the input path, and return a new
        `GameSave` object for furthur operations.
        根据输入的文件夹的路径，读取该存档，并返回一个`GameSave`对象，用于后续操作。

        ### Example
        ```python
        >>> world = GameSave.load("saves/70b...d36")
        ```

        ### Arguments
        - `path`
            The path of the save that you want to load.
            你想读取的存档的路径。

        ### Return
        A new `GameSave` object.
        一个新`GameSave`对象。
        """
        return GameSave(path)

    def _export_db(self, dict_, result_dict):
        for k, v in dict_.items():
            if isinstance(v, Exportable):
                result_dict[k] = v.export()

    def _write_db(self, cursor, dict_):
        for k, v in dict_.items():
            cursor.put(k, v)

    def _write_env(self, path: str, dict_):
        if not os.path.exists(path):
            pathlib.Path(path).mkdir(parents=True, exist_ok=True)
        db_data = {}
        size = 0
        for db in dict_:
            db_data[db] = {}
            self._export_db(dict_[db], db_data[db])
            for k, v in db_data[db].items():
                size += len(k) + len(v)
        env = lmdb.open(path, map_size=size << 3, max_dbs=self.MAX_DBS)
        with env.begin(write=True) as txn:
            for k, v in db_data.items():
                sub_db = env.open_db(k, txn=txn, create=True)
                cursor = txn.cursor(sub_db)
                self._write_db(cursor, db_data[k])
        env.close()

    def save(self, path: str) -> None:
        """Save the world to the given path. Existing files would be overwrite."""
        for env in self._data:
            self._write_env(os.path.join(path, env), self._data[env])

    def world_v2(self) -> Dict[str, Any]:
        return self._data["world_db"][b"main"][b"worldv2"]

    def get_summary(self) -> SaveSummary:
        world_v2 = self.world_v2()
        return SaveSummary(
            world_name=world_v2["worldName"],
            start_portal_pos=(
                world_v2["startPortalPos.x"],
                world_v2["startPortalPos.y"],
            ),
            seed=world_v2["randomSeed"],
            world_width_in_chunks=world_v2["worldWidthMacro"],
            expert_mode=world_v2["expertMode"],
        )

    def world_width(self) -> int:
        return self.world_v2()["worldWidthMacro"]

    def get_chunk(self, x: int, y: int) -> Chunk:
        assert (
            0 <= x < self._data["world_db"][b"main"][b"worldv2"]["worldWidthMacro"]
            and 0 <= y < 32
        )
        name = b"%d_%d" % (x, y)
        if name not in self.chunks:
            self.chunks[name] = Chunk.create()
        if not isinstance(self.chunks[name], Chunk):
            self.chunks[name] = Chunk(self.chunks[name]._data[0])
        return self.chunks[name]

    def set_chunk(self, x: int, y: int, c: Chunk):
        assert (
            0 <= x < self._data["world_db"][b"main"][b"worldv2"]["worldWidthMacro"]
            and 0 <= y < 32
        )
        self.chunks[b"%d_%d" % (x, y)] = c

    def get_chunks(self):
        return [[int(_) for _ in name.split(b"_")] for name in self.chunks]

    def get_block(self, x: int, y: int) -> Block:
        assert (
            0
            <= x
            < (self._data["world_db"][b"main"][b"worldv2"]["worldWidthMacro"] << 5)
            and 0 <= y < 1024
        )
        name = bytes("%d_%d" % (x >> 5, y >> 5), "ascii")
        if name not in self.chunks:
            self.chunks[name] = Chunk.create()
        if not isinstance(self.chunks[name], Chunk):
            self.chunks[name] = Chunk(self.chunks[name]._data[0])
        return self.chunks[name].get_block(x & 31, y & 31)

    def get_blockheads(self) -> list[Blockhead]:
        """
        Return a list containing reference to dictionaries describing
        blockheads.
        """
        return [
            Blockhead(d)
            for d in self["world_db"][b"main"][b"blockheads"]["dynamicObjects"]
        ]

    def get_inventory(self, blockhead: Blockhead):
        return Inventory(
            self["world_db"][b"main"][b"blockhead_%d_inventory" % blockhead.get_uid()]
        )


if __name__ == "__main__":
    from pprint import pprint
    from random import randint
    from blockType import BlockType

    gs = GameSave("./test_data/saves/c8185b81198a1890dac4b621677a9229/")
    info = gs.get_summary()
    start_chunk_pos = [_ >> 5 for _ in info.start_portal_pos]
    start_chunk_pos[1] += 1
    c = gs.get_chunk(*start_chunk_pos)
    for _ in range(128):
        block = c.get_block(randint(0, 31), randint(0, 31))
        block.set_attr("first_layer_id", BlockType.TIME_CRYSTAL.value)
    print("saving...")
    gs.save("./test_data/saves/out/")
