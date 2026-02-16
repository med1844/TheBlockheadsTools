class Snippet:
    def __init__(self, name: str):
        self.name = name

    def __enter__(self):
        print(f"# --8<-- [start:{self.name}]")

    def __exit__(self, _exc_type, _exc_value, _traceback):
        print(f"# --8<-- [end:{self.name}]")
