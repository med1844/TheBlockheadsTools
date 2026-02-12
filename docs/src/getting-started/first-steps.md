# First steps

This page covers basic usages.

## Read the world

```python
--8<-- "world_db_io.py:open_save"
```

### Print out world information

```python
--8<-- "world_db_io.py:world_info"
```

Run it, you should see something similar to this:

```
world name: TEST
seed: 1711316399
world width: 512
start portal: (10740.0, 520.0)
```

## Save the world

Say you have made some changes to the world and you are happy about it. You can save it with `WorldDb.save_path` or `WorldDb.save_bytes`.

```python
--8<-- "world_db_io.py:write_save"
```

## Edit blocks

```python

```
