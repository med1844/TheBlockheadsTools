# encoding: utf-8
import os
import os.path
import shutil
import lmdb
import plistlib
import biplist
from bplist import BPList
from gzipWrapper import GzipWrapper
from chunk import Chunk


class GameSave:
    """
    The class describes the save file of a world. This abstracts save file to 
    a simple class, and isolated instructions like creating lmdb context,
    manipulating cursors, and loading and saving BPLists, etc. It also 
    provides methods to load and save GameSave within one method call. On top
    of these, methods for manipulating chunks, blocks and dynamic objects 
    will be offered.

    As a homage to my past scripts, there would be bonus methods like:
    - exporting the whole world's map
    - loading design and color mapping, and converts the design into 
        `GameSave` object.

    存档类。用来描述一个世界的存档。将存档抽象成了一个类，封装了为了读写数据而必须的lmdb
    操作，以及解析BPList等工作，并且还提供读取与保存方法。此外，会为修改区块等操作提供方便的
    接口。
    作为对以前自己写的脚本的致敬，提供额外的:
    - 导出整个世界地图的功能
    - 根据设计图和颜色映射表，将设计图转换成存档的功能。
    """

    MAX_DBS = 100

    def __init__(self, folder_path):
        if not folder_path.endswith("/"):
            folder_path += "/"
        self._data = {}
        for sub_dir in ["world_db", "server_db", "lightBlocks"]:
            full_path = folder_path + sub_dir
            if os.path.isdir(full_path):
                self._data[sub_dir] = {}
                self._read_env(full_path, self._data[sub_dir])
        
        print("First round read completed.")
        print("Start parsing blocks...")

        self.chunks = self._data["world_db"]["blocks"]
        for name, gw in self.chunks.items():
            self.chunks[name] = Chunk(gw._data[0])
        
        print("Loaded.")
    
    def __repr__(self):
        return repr(self._data)

    def _read_env(self, path, dict_):
        """
        Read all databases in LMDB Environment from given path, and write 
        key-value pairs into `dict_`.
        从指定路径读取LMDB环境中的所有数据库，并将键值对写入dict_中。
        """
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
        根据输入字节判断应该解析成哪种数据，并递归地解析后返回结果。

        Types that would be parsed includes:
        会被解析的数据包括:
        - gzip files
        - base64 encoded data
        - bplist
        - normal string
        - xml plist files
        """
        if isinstance(src, (str, bytes, biplist.Data)):
            if src.startswith(b"bplist00"):  # bplist
                result = BPList(biplist.readPlistFromString(src),
                                src_type="bp")
                return self._parse(result)
            if src.startswith(b"\x1f\x8b"):  # gzip
                result = GzipWrapper(src)
                result._data[0] = self._parse(result._data[0])
                return result
            if src.startswith(b"<?xml"):  # xml plist
                result = BPList(biplist.readPlistFromString(src),
                                src_type="xml")
                return self._parse(result)
            return src
        elif isinstance(src, (list, tuple)):
            for i, v in enumerate(src):
                src[i] = self._parse(v)
            return src
        elif isinstance(src, (dict)):
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
            result_dict[k] = v.export()

    def _write_db(self, cursor, dict_):
        for k, v in dict_.items():
            cursor.put(k, v)
    
    def _write_env(self, path, dict_):
        db_data = {}
        size = 0
        for db in dict_:
            db_data[db] = {}
            self._export_db(dict_[db], db_data[db])
            for k, v in db_data[db].items():
                size += len(k) + len(v)
        env = lmdb.open(path, map_size=size<<2, max_dbs=self.MAX_DBS)
        with env.begin(write=True) as txn:
            for k, v in db_data.items():
                sub_db = env.open_db(k, txn=txn, create=True)
                cursor = txn.cursor(sub_db)
                self._write_db(cursor, db_data[k])
        env.close()
    
    def save(self, path):
        """
        """
        if not path.endswith("/"):
            path += "/"
        for env in self._data:
            self._write_env(path + env + "/", self._data[env])
            
if __name__ == "__main__":
    from pprint import pprint
    from random import randint
    from blockType import BlockType
    FOLDER = "./test_data/saves/c8185b81198a1890dac4b621677a9229/"
    gs = GameSave(FOLDER)
    for name, chunk in gs.chunks.items():
        print("Modifying chunk %s" % name)
        for _ in range(128):
            block = chunk.get_block(randint(0, 31), randint(0, 31))
            block.set_attr("first_layer_id", BlockType.TIME_CRYSTAL.value)
    print("saving...")
    gs.save("./test_data/saves/out/")