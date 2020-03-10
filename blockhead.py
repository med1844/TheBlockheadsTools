# encoding: utf-8
class Blockhead:

    def __init__(self, data):
        self._data = data

    def __repr__(self):
        return repr(self._data)
    
    def get_uid(self):
        return self._data["uniqueID"]
    
    def set_pos(self, x, y):
        self._data['pos_x'] = x
        self._data['pos_y'] = y
        self._data['float_pos'] = [x + .5, y]