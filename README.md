# theBlockheadsUtils
tools for manipulating save files of the iOS video game 'the blockheads' 

## How to use

### Read a world

```python
>>> from gameSave import GameSave
>>> s = GameSave.load("./test_data/saves/c8185...a9229/")
```

### Modify blocks

For example, if you want to change block at 12384, 372:

```python
>>> b = s.get_block(12384, 372)
```

And you would like to change it to time crystal:

```python
>>> from blockType import BlockType
>>> b.set(BlockType.TIME_CRYSTAL.value)
```

### Save the modified world

```python
>>> s.save("./test_data/saves/out/")
```
