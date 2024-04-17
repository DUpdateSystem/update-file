#!/usr/bin/env python3


## Data class start
from dataclasses import dataclass


@dataclass
class OperationData:
    data_map: dict[str, str]
    full_content: str
    content_index: int


@dataclass
class OperationOutput:
    data_map: dict[str, str]
    content_index: int
    new_content: str
    error_message: str


## Data class end

## Init data start
from sys import argv
from json import loads

json_in = loads(argv[1])
data = OperationData(**json_in)
## Init data end

## Init collected data start
# You can use this to store data to help you keep track of the state
# The runner won't touch this data also don't care and will forget it after the execution
data_map: dict[str, str] = data.data_map

# The content you will be working with
content = data.full_content
content_index = data.content_index

# The content of file you will get in the end
new_content = ""

# If you encounter an error, set this variable to a message that describes the error
# And the runner will print it out and stop the execution
error_message = ""

## Init collected data end

## Operation template start

raise NotImplementedError("Please implement the operation block")

## Operation template end

## Collect data start
from json import dumps
from dataclasses import asdict
from os import environ

output = OperationOutput(
    data_map=data_map,
    content_index=content_index,
    new_content=new_content,
    error_message=error_message,
)
json = dumps(asdict(output))
output_path = environ.get("OUTPUT_FILE", "/dev/stdout")
with open(output_path, "w") as f:
    f.write(json)
## Collect data end
