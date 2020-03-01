# encoding: utf-8
"""
Base class of all classes that need to be exported to
a blockhead save file.

所有需要导出至BH存档的类所需要继承的基类。
"""

from abc import ABCMeta, abstractmethod


class Exportable:

    __metaclass__ = ABCMeta

    @abstractmethod
    def export(self):
        """
        Return the binary representation of self, which would be later 
        exported to files by `writeFiles.py`.
        Returning bytes object would make it simpler for recursively 
        exporting nested structures.

        返回自身的二进制表示。存储将交由`writeFiles.py`实现。
        通过返回bytes对象，可以递归地实现嵌套结构数据的导出。
        """
        raise NotImplementedError
