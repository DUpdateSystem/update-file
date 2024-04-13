#!/usr/bin/env python3


## Data class start
from dataclasses import dataclass


@dataclass
class OperationData:
    data_map: dict[str, str]
    full_content: str
    remaining_content: str


@dataclass
class OperationOutput:
    data_map: dict[str, str]
    full_content: str


## Data class end


## Operation class start
class Operation:
    def process(self, data: OperationData) -> OperationOutput:
        pass


## Operation class end


def main(data: OperationData) -> OperationOutput:
    operation = Operation()
    return operation.process(data)


from json import loads, dumps

if __name__ == "__main__":
    from sys import argv

    json_in = loads(argv[1])
    data = OperationData(**json_in)
    output = main(data)
    json = dumps(output, class_name="OperationOutput")
    print(json)
